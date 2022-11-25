use ckb_sdk::ScriptId;
use ckb_types::{
    core::DepType,
    h256,
    packed::{CellDep, CellDepBuilder, OutPointBuilder},
    prelude::*,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SECP256K1_BLAKE160: ScriptId = ScriptId::new_type(h256!(
        "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"
    ),);
    pub static ref SECP256K1_BLAKE160_DEP: CellDep = CellDepBuilder::default()
        .dep_type(DepType::DepGroup.into())
        .out_point(
            OutPointBuilder::default()
                .tx_hash(
                    h256!("0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37")
                        .pack()
                )
                .index(0u32.pack())
                .build(),
        )
        .build();
}
