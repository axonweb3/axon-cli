use blake2b_rs::Blake2bBuilder;
use ckb_sdk::constants::TYPE_ID_CODE_HASH;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, ScriptHashType},
    h256,
    packed::{self, CellOutput, CellOutputBuilder, Script, ScriptBuilder},
    prelude::*,
    H256,
};
use lazy_static::lazy_static;

use super::schema::{
    CheckpointLockArgsBuilder, CheckpointLockCellDataBuilder, OmniDataBuilder, OmniLockArgsBuilder,
    SelectionLockArgsBuilder, StakeLockArgsBuilder, StakeLockCellDataBuilder,
};
use crate::types::Result;

pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

lazy_static! {
    pub static ref OMNI_CODE_HASH: H256 =
        h256!("0x79f90bb5e892d80dd213439eeab551120eb417678824f282b4ffb5f21bad2e1e");
    pub static ref CHECKPOINT_CODE_HASH: H256 =
        h256!("0xd224d5e5d26d67c0b0ffb9b32cb7de3d32e8270a921c96dd70ee4cdb1418278a");
    pub static ref STAKE_CODE_HASH: H256 =
        h256!("0x951a3594d46e087eb118448075864da309b88f1eefb0c173ef8480a46027b9d5");
    pub static ref SELECTION_CODE_HASH: H256 =
        h256!("0xd799aa64646feae5a1df6379a40514797c628e0cfe2af1362d817e1ff823c1c6");
    pub static ref SECP256K1_BLAKE160_CODE_HASH: H256 =
        h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8");
    pub static ref SUDT_CODE_HASH: H256 =
        h256!("0xc5e5dcf215925f7ef4dfaf5f4b4f105bc321c02776d6e7d52a1db3fcd9d011a4");
}

pub fn get_type_script_builder(code_hash: &H256) -> ScriptBuilder {
    ScriptBuilder::default()
        .hash_type(ScriptHashType::Type.into())
        .code_hash(code_hash.pack())
}

pub fn build_sudt_script(args: packed::Bytes) -> Script {
    get_type_script_builder(&SUDT_CODE_HASH).args(args).build()
}

pub fn build_type_id_script(first_input_out_point: &packed::CellInput, cell_index: u64) -> Script {
    let mut args = [0u8; 32];
    let mut blake2b = Blake2bBuilder::new(32)
        .personal(CKB_HASH_PERSONALIZATION)
        .build();

    blake2b.update(first_input_out_point.as_slice());
    blake2b.update(&cell_index.to_le_bytes());
    blake2b.finalize(&mut args);

    get_type_script_builder(&TYPE_ID_CODE_HASH)
        .args(args.as_ref().pack())
        .build()
}

#[derive(Default)]
pub struct OutputCellBuilder<LockArgsBuilder, DataBuilder> {
    pub lock_builder:      ScriptBuilder,
    pub lock_args_builder: Option<LockArgsBuilder>,
    pub type_:             Option<Script>,
    pub data_builder:      Option<DataBuilder>,
}

impl<LockArgsBuilder: Builder, DataBuilder: Builder>
    OutputCellBuilder<LockArgsBuilder, DataBuilder>
{
    pub fn build_lock(mut self) -> Result<(Self, Script)> {
        let lock = if let Some(lock_args_builder) = &self.lock_args_builder {
            let lock_args = lock_args_builder.build().as_bytes();
            self.lock_builder = self.lock_builder.args(lock_args.pack());
            self.lock_builder.build()
        } else {
            self.lock_builder.build()
        };

        Ok((self, lock))
    }

    pub fn build(self) -> Result<(Self, CellOutput, Option<Bytes>)> {
        let (new_self, lock) = self.build_lock()?;

        let output_builder = CellOutputBuilder::default()
            .lock(lock)
            .type_(new_self.type_.pack());

        let (output, data) = if let Some(data_builder) = &new_self.data_builder {
            let data = data_builder.build().as_bytes();
            (
                output_builder.build_exact_capacity(Capacity::bytes(data.len())?)?,
                Some(data),
            )
        } else {
            (output_builder.build_exact_capacity(Capacity::zero())?, None)
        };

        Ok((new_self, output, data))
    }
}

impl OutputCellBuilder<(), ()> {
    pub fn get_selection_cell_builder(
    ) -> OutputCellBuilder<SelectionLockArgsBuilder, SelectionLockArgsBuilder> {
        OutputCellBuilder {
            type_:             None,
            lock_builder:      get_type_script_builder(&SELECTION_CODE_HASH),
            lock_args_builder: Some(SelectionLockArgsBuilder::default()),
            data_builder:      None,
        }
    }

    pub fn get_omni_cell_builder() -> OutputCellBuilder<OmniLockArgsBuilder, OmniDataBuilder> {
        OutputCellBuilder {
            type_:             Some(build_type_id_script(&Default::default(), 0)),
            lock_builder:      get_type_script_builder(&OMNI_CODE_HASH),
            lock_args_builder: Some(OmniLockArgsBuilder::default()),
            data_builder:      Some(OmniDataBuilder::default()),
        }
    }

    pub fn get_checkpoint_cell_builder(
    ) -> OutputCellBuilder<CheckpointLockArgsBuilder, CheckpointLockCellDataBuilder> {
        OutputCellBuilder {
            type_:             Some(build_type_id_script(&Default::default(), 0)),
            lock_builder:      get_type_script_builder(&CHECKPOINT_CODE_HASH),
            lock_args_builder: Some(CheckpointLockArgsBuilder::default()),
            data_builder:      Some(CheckpointLockCellDataBuilder::default()),
        }
    }

    pub fn get_stake_cell_builder(
    ) -> OutputCellBuilder<StakeLockArgsBuilder, StakeLockCellDataBuilder> {
        OutputCellBuilder {
            type_:             Some(build_type_id_script(&Default::default(), 0)),
            lock_builder:      get_type_script_builder(&STAKE_CODE_HASH),
            lock_args_builder: Some(StakeLockArgsBuilder::default()),
            data_builder:      Some(StakeLockCellDataBuilder::default()),
        }
    }
}
