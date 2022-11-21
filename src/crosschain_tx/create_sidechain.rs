use std::collections::HashMap;

use ckb_sdk::{
    traits::{
        DefaultCellCollector, DefaultHeaderDepResolver, DefaultTransactionDependencyProvider,
        OffchainCellDepResolver,
    },
    tx_builder::{balance_tx_capacity, CapacityBalancer, CapacityProvider, SinceSource},
    ScriptId,
};
use ckb_types::{
    bytes::Bytes,
    core::{FeeRate, TransactionView},
    h160,
    packed::{self, CellDep},
    prelude::*,
    H160, H256,
};
use log::info;
use molecule::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    cells::{get_type_script_builder, OutputCellBuilder, SECP256K1_BLAKE160_CODE_HASH},
    schema::{self, Byte97, IdentityBuilder, StakeInfoBuilder, StakeInfoVecBuilder},
    scripts::{SECP256K1_BLAKE160, SECP256K1_BLAKE160_DEP},
};
use crate::{
    crosschain_tx::{
        cells::{build_sudt_script, build_type_id_script},
        schema::Byte10,
    },
    types::Result,
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct StakeInfo {
    pub identity:         Identity,
    pub l2_address:       H160,
    pub bls_pub_key:      Bytes,
    pub stake_amount:     String,
    pub inauguration_era: u64,
}

impl TryFrom<&StakeInfo> for schema::StakeInfo {
    type Error = crate::types::Error;

    fn try_from(val: &StakeInfo) -> Result<Self> {
        if val.bls_pub_key.len() != 194 {
            return Err("BLS public key length is not 97".into());
        }
        let stake_amount = val.stake_amount.parse::<u128>()?;
        Ok(StakeInfoBuilder::default()
            .identity((&val.identity).into())
            .l2_address((&val.l2_address).into())
            .bls_pub_key(Byte97::new_unchecked(val.bls_pub_key.clone()))
            .stake_amount(stake_amount.into())
            .inauguration_era(val.inauguration_era.into())
            .build())
    }
}

impl TryFrom<&Vec<StakeInfo>> for schema::StakeInfoVec {
    type Error = crate::types::Error;

    fn try_from(val: &Vec<StakeInfo>) -> Result<Self> {
        let items = val
            .iter()
            .map(|i| i.try_into())
            .collect::<Result<Vec<schema::StakeInfo>>>()?;

        Ok(StakeInfoVecBuilder(items).build())
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct OmniConfig {
    pub version:    u8,
    pub max_supply: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CheckpointConfig {
    pub version:              u8,
    pub period_interval:      u32,
    pub era_period:           u32,
    pub base_reward:          String,
    pub half_period:          u64,
    pub common_ref:           Bytes,
    pub withdrawal_lock_hash: H256,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct StakeConfig {
    pub version:     u8,
    pub stake_infos: Vec<StakeInfo>,
    pub quoram_size: u8,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Identity {
    pub flag:    u8,
    pub content: H160,
}

impl From<&Identity> for schema::Identity {
    fn from(id: &Identity) -> Self {
        IdentityBuilder::default()
            .flag(packed::Byte::new(id.flag))
            .content((&id.content).into())
            .build()
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CreateSidechainArgs {
    pub omni_config:       OmniConfig,
    pub checkpoint_config: CheckpointConfig,
    pub stake_config:      StakeConfig,
    pub admin_identity:    Identity,
}

async fn balance_tx_capacity_by_pubkey_hash(
    tx_view: &TransactionView,
    fee_rate: FeeRate,
    pubkey_hash: H160,
) -> Result<TransactionView> {
    let _ckb_indexer_uri = "https://mercury-testnet.ckbapp.dev/indexer";
    let ckb_uri = "https://mercury-testnet.ckbapp.dev/rpc";
    let (mut cell_collector, tx_dep_provider, header_dep_resolver) =
        tokio::task::block_in_place(|| {
            (
                DefaultCellCollector::new(ckb_uri),
                DefaultTransactionDependencyProvider::new(ckb_uri, 0),
                DefaultHeaderDepResolver::new(ckb_uri),
            )
        });

    let mut cell_deps: HashMap<ScriptId, (CellDep, String)> = HashMap::new();
    cell_deps.insert(
        SECP256K1_BLAKE160.clone(),
        (SECP256K1_BLAKE160_DEP.clone(), "secp256k1".to_string()),
    );

    let cell_dep_resolver = OffchainCellDepResolver { items: cell_deps };

    let lock_script = get_type_script_builder(&SECP256K1_BLAKE160_CODE_HASH)
        .args(pubkey_hash.as_bytes().pack())
        .build();
    let scripts = vec![(
        lock_script.clone(),
        Default::default(),
        SinceSource::Value(0),
    )];

    Ok(tokio::task::block_in_place(|| {
        balance_tx_capacity(
            tx_view,
            &CapacityBalancer {
                fee_rate,
                capacity_provider: CapacityProvider::new(scripts),
                change_lock_script: Some(lock_script),
                force_small_change_as_fee: None,
            },
            &mut cell_collector,
            &tx_dep_provider,
            &cell_dep_resolver,
            &header_dep_resolver,
        )
    })?)
}

const DEFAULT_FEE_RATE: u64 = 1000;

pub(crate) async fn create_sidechain(args: CreateSidechainArgs) -> Result<()> {
    let CreateSidechainArgs {
        admin_identity,
        omni_config,
        checkpoint_config,
        stake_config,
    } = args;
    let selection_cell_builder = OutputCellBuilder::get_selection_cell_builder();
    let omni_cell_builder = OutputCellBuilder::get_omni_cell_builder();
    let checkpoint_cell_builder = OutputCellBuilder::get_checkpoint_cell_builder();
    let stake_cell_builder = OutputCellBuilder::get_stake_cell_builder();

    let (mut selection_cell_builder, selection_cell, _) = selection_cell_builder.build()?;
    let (mut omni_cell_builder, omni_cell, _) = omni_cell_builder.build()?;
    let (mut checkpoint_cell_builder, checkpoint_cell, _) = checkpoint_cell_builder.build()?;
    let (mut stake_cell_builder, stake_cell, _) = stake_cell_builder.build()?;

    let tx_view = TransactionView::new_advanced_builder()
        .outputs([selection_cell, omni_cell, checkpoint_cell, stake_cell])
        .outputs_data([
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        ])
        .build();

    info!("Balancing tx...");
    let tx_view = balance_tx_capacity_by_pubkey_hash(
        &tx_view,
        FeeRate(DEFAULT_FEE_RATE),
        h160!("0x414144f1de540e1eab7e8d644c23739c57a4c607"),
    )
    .await?;

    let first_input_cell = tx_view.inputs().get(0).unwrap();
    let omni_type_id = build_type_id_script(&first_input_cell, 1);
    let omni_type_hash = omni_type_id.calc_script_hash();
    omni_cell_builder.type_ = Some(omni_type_id);
    let checkpoint_type_id = build_type_id_script(&first_input_cell, 2);
    let checkpoint_type_hash = checkpoint_type_id.calc_script_hash();
    let stake_type_id = build_type_id_script(&first_input_cell, 3);
    let stake_type_hash = stake_type_id.calc_script_hash();

    omni_cell_builder.lock_args_builder = omni_cell_builder.lock_args_builder.map(|builder| {
        builder
            .flag(8u8.into())
            .identity((&admin_identity).into())
            .omni_type_hash(omni_type_hash.into())
    });
    checkpoint_cell_builder.lock_args_builder =
        checkpoint_cell_builder.lock_args_builder.map(|builder| {
            builder
                .admin_identity((&admin_identity).into())
                .type_id_hash(checkpoint_type_hash.into())
        });
    stake_cell_builder.lock_args_builder = stake_cell_builder.lock_args_builder.map(|builder| {
        builder
            .admin_identity((&admin_identity).into())
            .type_id_hash(stake_type_hash.into())
    });

    let (mut omni_cell_builder, omni_lock) = omni_cell_builder.build_lock()?;
    let (mut checkpoint_cell_builder, checkpoint_lock) = checkpoint_cell_builder.build_lock()?;
    let omni_lock_hash = omni_lock.calc_script_hash();
    let checkpoint_lock_hash = checkpoint_lock.calc_script_hash();
    selection_cell_builder.lock_args_builder =
        selection_cell_builder.lock_args_builder.map(|builder| {
            builder
                .omni_lock_hash(omni_lock_hash.into())
                .checkpoint_lock_hash(checkpoint_lock_hash.into())
        });
    let (selection_cell_builder, selection_lock) = selection_cell_builder.build_lock()?;
    let sudt_args = selection_lock.calc_script_hash();
    let sudt_type_hash = build_sudt_script(sudt_args.as_bytes().pack()).calc_script_hash();

    {
        let max_supply = omni_config.max_supply.parse::<u128>()?;
        omni_cell_builder.data_builder = omni_cell_builder.data_builder.map(|builder| {
            builder
                .version(omni_config.version.into())
                .max_supply(max_supply.into())
                .sudt_type_hash(sudt_type_hash.clone().into())
        });
    }
    {
        let base_reward = checkpoint_config.base_reward.parse::<u128>()?;
        checkpoint_cell_builder.data_builder =
            checkpoint_cell_builder.data_builder.map(|builder| {
                builder
                    .version(checkpoint_config.version.into())
                    .sudt_type_hash(sudt_type_hash.clone().into())
                    .withdrawal_lock_code_hash((&checkpoint_config.withdrawal_lock_hash).into())
                    .common_ref(Byte10::new_unchecked(checkpoint_config.common_ref))
                    .half_period(checkpoint_config.half_period.into())
                    .era_period(checkpoint_config.era_period.into())
                    .period_interval(checkpoint_config.period_interval.into())
                    .base_reward(base_reward.into())
            });
    }
    {
        let stake_infos = (&stake_config.stake_infos).try_into()?;

        stake_cell_builder.data_builder = stake_cell_builder.data_builder.map(|builder| {
            builder
                .version(stake_config.version.into())
                .quorum_size(stake_config.quoram_size.into())
                .stake_infos(stake_infos)
                .sudt_type_hash(sudt_type_hash.into())
        });
    }

    let (_, selection_cell, _) = selection_cell_builder.build()?;
    let (_, omni_cell, omni_data) = omni_cell_builder.build()?;
    let (_, checkpoint_cell, checkpoint_data) = checkpoint_cell_builder.build()?;
    let (_, stake_cell, stake_data) = stake_cell_builder.build()?;
    let omni_data = omni_data.unwrap();
    let checkpoint_data = checkpoint_data.unwrap();
    let stake_data = stake_data.unwrap();

    let tx_view = tx_view
        .as_advanced_builder()
        .outputs(vec![selection_cell, omni_cell, checkpoint_cell, stake_cell])
        .outputs_data([
            Default::default(),
            omni_data.pack(),
            checkpoint_data.pack(),
            stake_data.pack(),
        ])
        .build();

    info!("Balanced tx: {}", tx_view);

    Ok(())
    //     // Update checkpoint cell
    //     let checkpoint_type_script = build_type_id_script(&first_input_cell,
    // 2)?;     let checkpoint_type_hash =
    // checkpoint_type_script.calc_script_hash();
    //     let checkpoint_lock_args = checkpoint_cell.lock().args().raw_data();
    //     let new_args =
    // CheckpointLockArgs::new_unchecked(checkpoint_lock_args)
    //         .as_builder()
    //         .type_id_hash(checkpoint_type_hash.into())
    //         .build();
    //     let checkpoint_lock = checkpoint_cell
    //         .lock()
    //         .as_builder()
    //         .args(new_args.as_bytes().pack())
    //         .build();
    //     let checkpoint_cell = checkpoint_cell
    //         .as_builder()
    //         .lock(checkpoint_lock)
    //         .type_(Some(checkpoint_type_script).pack())
    //         .build();
    //
    //     // Update stake cell
    //     let stake_type_script = build_type_id_script(&first_input_cell, 3)?;
    //     let stake_type_hash = stake_type_script.calc_script_hash();
    //     let stake_lock_args = stake_cell.lock().args().raw_data();
    //     let new_args =
    // generated::StakeLockArgs::new_unchecked(stake_lock_args)
    //         .as_builder()
    //         .type_id_hash(stake_type_hash.clone().into())
    //         .build();
    //     let stake_lock_script = stake_cell
    //         .lock()
    //         .as_builder()
    //         .args(new_args.as_bytes().pack())
    //         .build();
    //     let stake_cell = stake_cell
    //         .as_builder()
    //         .type_(Some(stake_type_script).pack())
    //         .lock(stake_lock_script)
    //         .build();
    //
    //     // Update selection cell
    //     let omni_lock_hash = omni_cell.lock().calc_script_hash();
    //     let checkpoint_lock_hash = checkpoint_cell.lock().calc_script_hash();
    //     let new_args = generated::SelectionLockArgsBuilder::default()
    //         .omni_lock_hash(omni_lock_hash.into())
    //         .checkpoint_lock_hash(checkpoint_lock_hash.into())
    //         .build();
    //     let selection_lock_script = selection_cell
    //         .lock()
    //         .as_builder()
    //         .args(new_args.as_bytes().pack())
    //         .build();
    //     let selection_cell = selection_cell
    //         .as_builder()
    //         .lock(selection_lock_script)
    //         .build();
    //
    //     let sudt_args = selection_cell.lock().calc_script_hash();
    //     let sudt_type_hash =
    // self.build_sudt_script(sudt_args).calc_script_hash();
    //
    //     // Updata omni data
    //     let mut omni_cell_data = omni_cell_data.to_vec();
    //     omni_cell_data[33..].swap_with_slice(&mut
    // sudt_type_hash.raw_data().to_vec());
    //
    //     // Update checkpoint data
    //     let checkpoint_cell_data =
    //         generated::CheckpointLockCellData::new_unchecked(checkpoint_cell_data)
    //             .as_builder()
    //             .sudt_type_hash(sudt_type_hash.clone().into())
    //             .stake_type_hash(stake_type_hash.clone().into())
    //             .build()
    //             .as_bytes();
    //
    //     // Updata stake data
    //     let stake_cell_data =
    // generated::StakeLockCellData::new_unchecked(stake_cell_data)
    //         .as_builder()
    //         .sudt_type_hash(sudt_type_hash.into())
    //         .build()
    //         .as_bytes();
    //
    //
    //     let script_groups = self.get_tx_script_groups(&tx_view)?;
    //     println!("{:?}", script_groups);
    //     Ok(TransactionCompletionResponse::new(
    //         tx_view.into(),
    //         script_groups,
    //     ))
}
