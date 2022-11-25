use std::{fs::create_dir_all, path::Path};

use ckb_jsonrpc_types::Transaction;
use ckb_sdk::NetworkType;
use ckb_types::{
    core::{FeeRate, TransactionView},
    prelude::*,
};
use clap::Args;
use log::info;
use molecule::prelude::*;

use super::{cells::OutputCellBuilder, ckb_context::CkbContext, Ckb};
use crate::{
    constants::{CREATE_SIDECHAIN_CONFIG_TEMPLATE, DEFAULT_CREAET_SIDECHAIN_CONFIG_PATH},
    crosschain_tx::{
        cells::{build_sudt_script, build_type_id_script},
        ckb_context::BalanceTxByPubKeyHashArgs,
        schema::Byte10,
        types::CreateSidechainConfigs,
        utils::{parse_private_key, CkbKeyInfo},
    },
    types::Result,
    utils::read_or_create_json_template,
};

#[derive(Args, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct CreateSidechainArgs {
    /// uri to CKB rpc
    #[clap(short, long, default_value = "https://mercury-testnet.ckbapp.dev/rpc")]
    pub ckb_uri:             String,
    /// private key to sign tx
    #[clap(short = 'P', long)]
    pub private_key:         String,
    /// is network type testnet
    #[clap(short = 't', long)]
    pub is_testnet:          bool,
    /// the path of sidechain config file
    #[clap(short='p', long, default_value=*DEFAULT_CREAET_SIDECHAIN_CONFIG_PATH)]
    pub config_path:         String,
    /// should we skip all confirmations
    #[clap(short = 'y', long)]
    pub should_skip_confirm: bool,
}

const DEFAULT_FEE_RATE: u64 = 1000;

impl Ckb {
    pub async fn create_sidechain(&mut self, args: CreateSidechainArgs) -> Result<Transaction> {
        let CreateSidechainArgs {
            config_path,
            ckb_uri,
            private_key,
            is_testnet,
            ..
        } = args;
        if let Some(parent) = <_ as AsRef<Path>>::as_ref(&config_path).parent() {
            create_dir_all(parent)?;
        }
        let CreateSidechainConfigs {
            omni_config,
            checkpoint_config,
            stake_config,
            admin_identity,
        } = read_or_create_json_template(config_path, &*CREATE_SIDECHAIN_CONFIG_TEMPLATE)?
            .into_owned();
        let CkbKeyInfo {
            private_key,
            address,
            lock_args,
            ..
        } = parse_private_key(
            private_key,
            if is_testnet {
                NetworkType::Testnet
            } else {
                NetworkType::Mainnet
            },
        )?;
        info!("From address {}(0x{lock_args})", address.to_string());

        let context = self.get_context(ckb_uri).await?;
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
        let tx_view = context
            .balance_tx_capacity_by_pubkey_hash(BalanceTxByPubKeyHashArgs {
                tx_view,
                fee_rate: FeeRate(DEFAULT_FEE_RATE),
                pubkey_hash: lock_args,
            })
            .await?;

        let first_input_cell = tx_view.inputs().get(0).unwrap();
        let omni_type_id = build_type_id_script(&first_input_cell, 1);
        let omni_type_hash = omni_type_id.calc_script_hash();
        omni_cell_builder.type_ = Some(omni_type_id);
        let checkpoint_type_id = build_type_id_script(&first_input_cell, 2);
        let checkpoint_type_hash = checkpoint_type_id.calc_script_hash();
        checkpoint_cell_builder.type_ = Some(checkpoint_type_id);
        let stake_type_id = build_type_id_script(&first_input_cell, 3);
        let stake_type_hash = stake_type_id.calc_script_hash();
        stake_cell_builder.type_ = Some(stake_type_id);

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
        stake_cell_builder.lock_args_builder =
            stake_cell_builder.lock_args_builder.map(|builder| {
                builder
                    .admin_identity((&admin_identity).into())
                    .type_id_hash(stake_type_hash.into())
            });

        let (mut omni_cell_builder, omni_lock) = omni_cell_builder.build_lock()?;
        let (mut checkpoint_cell_builder, checkpoint_lock) =
            checkpoint_cell_builder.build_lock()?;
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

        let change_cell_outputs = tx_view
            .outputs_with_data_iter()
            .skip(4)
            .map(|(output, _)| output);
        let change_cell_datas = tx_view
            .outputs_with_data_iter()
            .skip(4)
            .map(|(_, data)| data.pack());
        let tx_view = tx_view
            .as_advanced_builder()
            .set_outputs(
                [selection_cell, omni_cell, checkpoint_cell, stake_cell]
                    .into_iter()
                    .chain(change_cell_outputs)
                    .collect::<Vec<_>>(),
            )
            .set_outputs_data(
                [
                    Default::default(),
                    omni_data.pack(),
                    checkpoint_data.pack(),
                    stake_data.pack(),
                ]
                .into_iter()
                .chain(change_cell_datas)
                .collect::<Vec<_>>(),
            )
            .build();

        let tx = context
            .unlock_tx_by_secp256k1_private_key(tx_view, private_key)
            .await?
            .data()
            .into();

        Ok(tx)
    }
}
