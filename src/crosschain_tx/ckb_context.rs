use std::{
    collections::HashMap,
    ops::DerefMut,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use ckb_jsonrpc_types::{OutputsValidator, Transaction};
use ckb_sdk::{
    traits::{
        CellCollector, CellDepResolver, DefaultCellCollector, DefaultHeaderDepResolver,
        DefaultTransactionDependencyProvider, HeaderDepResolver, OffchainCellDepResolver,
        SecpCkbRawKeySigner, TransactionDependencyProvider,
    },
    tx_builder::{balance_tx_capacity, unlock_tx, CapacityBalancer, CapacityProvider, SinceSource},
    unlock::{ScriptUnlocker, SecpSighashScriptSigner, SecpSighashUnlocker},
    CkbRpcClient, ScriptId,
};
use ckb_types::{
    core::{FeeRate, TransactionView},
    packed::CellDep,
    prelude::*,
    H160, H256,
};
use secp256k1::SecretKey;
use tokio::task::JoinError;

use super::{
    cells::{get_type_script_builder, SECP256K1_BLAKE160_CODE_HASH},
    constants::{SECP256K1_BLAKE160, SECP256K1_BLAKE160_DEP},
};
use crate::types::Result;

#[async_trait]
pub trait CkbContext {
    async fn balance_tx_capacity_by_pubkey_hash(
        &self,
        args: BalanceTxByPubKeyHashArgs,
    ) -> Result<TransactionView>;

    async fn unlock_tx_by_secp256k1_private_key(
        &self,
        tx_view: TransactionView,
        private_key: SecretKey,
    ) -> Result<TransactionView>;

    async fn send_transaction(&self, tx: Transaction) -> Result<H256>;

    fn reset(&self) -> Result<()>;
}

pub struct SdkCkbContext<
    CellCollectorType: CellCollector + Send + Sync + 'static,
    TransactionDependencyProviderType: TransactionDependencyProvider + Send + Sync + 'static,
    HeaderDepResolverType: HeaderDepResolver + Send + Sync + 'static,
    CellDepResolverType: CellDepResolver + Send + Sync + 'static,
> {
    pub cell_collector:      Arc<Mutex<CellCollectorType>>,
    pub tx_dep_provider:     Arc<TransactionDependencyProviderType>,
    pub header_dep_resolver: Arc<HeaderDepResolverType>,
    pub cell_dep_resolver:   Arc<CellDepResolverType>,
    pub ckb_rpc_client:      Arc<Mutex<CkbRpcClient>>,
}

pub type DefaultSdkCkbContext = SdkCkbContext<
    DefaultCellCollector,
    DefaultTransactionDependencyProvider,
    DefaultHeaderDepResolver,
    OffchainCellDepResolver,
>;

impl DefaultSdkCkbContext {
    pub async fn new(
        ckb_uri: impl AsRef<str> + Send + 'static,
    ) -> std::result::Result<Self, JoinError> {
        let (cell_collector, tx_dep_provider, header_dep_resolver, ckb_rpc_client) =
            tokio::task::spawn_blocking(move || {
                let ckb_uri = ckb_uri.as_ref();
                (
                    Mutex::new(DefaultCellCollector::new(ckb_uri)),
                    DefaultTransactionDependencyProvider::new(ckb_uri, 0),
                    DefaultHeaderDepResolver::new(ckb_uri),
                    Mutex::new(CkbRpcClient::new(ckb_uri)),
                )
            })
            .await?;

        let mut cell_deps: HashMap<ScriptId, (CellDep, String)> = HashMap::new();
        cell_deps.insert(
            SECP256K1_BLAKE160.clone(),
            (SECP256K1_BLAKE160_DEP.clone(), "secp256k1".to_string()),
        );

        let cell_dep_resolver = OffchainCellDepResolver { items: cell_deps };

        Ok(DefaultSdkCkbContext {
            cell_collector:      Arc::new(cell_collector),
            tx_dep_provider:     Arc::new(tx_dep_provider),
            header_dep_resolver: Arc::new(header_dep_resolver),
            cell_dep_resolver:   Arc::new(cell_dep_resolver),
            ckb_rpc_client:      Arc::new(ckb_rpc_client),
        })
    }
}

pub struct BalanceTxByPubKeyHashArgs {
    pub tx_view:     TransactionView,
    pub pubkey_hash: H160,
    pub fee_rate:    FeeRate,
}

#[async_trait]
impl<
        CellCollectorType: CellCollector + Send + Sync + 'static,
        TransactionDependencyProviderType: TransactionDependencyProvider + Send + Sync + 'static,
        HeaderDepResolverType: HeaderDepResolver + Send + Sync + 'static,
        CellDepResolverType: CellDepResolver + Send + Sync + 'static,
    > CkbContext
    for SdkCkbContext<
        CellCollectorType,
        TransactionDependencyProviderType,
        HeaderDepResolverType,
        CellDepResolverType,
    >
{
    async fn balance_tx_capacity_by_pubkey_hash(
        &self,
        args: BalanceTxByPubKeyHashArgs,
    ) -> Result<TransactionView> {
        let BalanceTxByPubKeyHashArgs {
            tx_view,
            pubkey_hash,
            fee_rate,
        } = args;

        let lock_script = get_type_script_builder(&SECP256K1_BLAKE160_CODE_HASH)
            .args(pubkey_hash.as_bytes().pack())
            .build();
        let scripts = vec![(
            lock_script.clone(),
            Default::default(),
            SinceSource::Value(0),
        )];

        let cell_collector = self.cell_collector.clone();
        let tx_dep_provider = self.tx_dep_provider.clone();
        let header_dep_resolver = self.header_dep_resolver.clone();
        let cell_dep_resolver = self.cell_dep_resolver.clone();
        Ok(
            tokio::task::spawn_blocking(move || -> std::result::Result<TransactionView, String> {
                balance_tx_capacity(
                    &tx_view,
                    &CapacityBalancer {
                        fee_rate,
                        capacity_provider: CapacityProvider::new(scripts),
                        change_lock_script: Some(lock_script),
                        force_small_change_as_fee: None,
                    },
                    cell_collector
                        .lock()
                        .map_err(|err| err.to_string())?
                        .deref_mut(),
                    tx_dep_provider.as_ref(),
                    cell_dep_resolver.as_ref(),
                    header_dep_resolver.as_ref(),
                )
                .map_err(|err| err.to_string())
            })
            .await??,
        )
    }

    async fn unlock_tx_by_secp256k1_private_key(
        &self,
        tx_view: TransactionView,
        private_key: SecretKey,
    ) -> Result<TransactionView> {
        let tx_dep_provider = self.tx_dep_provider.clone();
        let (tx_view, not_unlocked) = tokio::task::spawn_blocking(move || {
            let secp_raw_signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![private_key]);
            let secp_signer = SecpSighashScriptSigner::new(Box::new(secp_raw_signer));
            let secp_unlocker = SecpSighashUnlocker::new(secp_signer);

            let mut unlockers: HashMap<ScriptId, Box<dyn ScriptUnlocker>> = HashMap::new();
            unlockers.insert(SECP256K1_BLAKE160.clone(), Box::new(secp_unlocker));

            unlock_tx(tx_view, tx_dep_provider.as_ref(), &unlockers)
        })
        .await??;

        if not_unlocked.is_empty() {
            Ok(tx_view)
        } else {
            Err(format!("Unable to unlock scripts: {not_unlocked:#?}").into())
        }
    }

    async fn send_transaction(&self, tx: Transaction) -> Result<H256> {
        let ckb_rpc_client = self.ckb_rpc_client.clone();
        let hash = tokio::task::spawn_blocking(move || -> std::result::Result<H256, String> {
            ckb_rpc_client
                .lock()
                .map_err(|err| err.to_string())?
                .deref_mut()
                .send_transaction(tx, Some(OutputsValidator::Passthrough))
                .map_err(|err| err.to_string())
        })
        .await??;
        Ok(hash)
    }

    fn reset(&self) -> Result<()> {
        self.cell_collector
            .lock()
            .map_err(|err| err.to_string())?
            .deref_mut()
            .reset();

        Ok(())
    }
}
