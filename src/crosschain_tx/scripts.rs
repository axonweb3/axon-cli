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
        .dep_type(DepType::Code.into())
        .out_point(
            OutPointBuilder::default()
                .tx_hash(
                    h256!("0xc0851fda8f09c23f9df6624cd4ed906dc5bd1e313ddb031b04bb7558e6a351e3")
                        .pack()
                )
                .index(0u32.pack())
                .build(),
        )
        .build();
}
// cell_dep = '''
// {
//     "dep_type": "dep_group",
//     "out_point": {
//         "index": "0x0",
//         "tx_hash":
// "0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37"     }
// }
// '''
//
// [[builtin_scripts]]
// script_name = "axon_selection"
// script = '''
// {
//     "args": "0x",
//     "code_hash":
// "0xd799aa64646feae5a1df6379a40514797c628e0cfe2af1362d817e1ff823c1c6",
//     "hash_type": "data"
// }
// '''
// cell_dep = '''
// {
//     "dep_type": "code",
//     "out_point": {
//         "index": "0x0",
//         "tx_hash":
// "0xc0851fda8f09c23f9df6624cd4ed906dc5bd1e313ddb031b04bb7558e6a351e3"     }
// }
// '''
//
//
// [[builtin_scripts]]
// script_name = "axon_checkpoint"
// script = '''
// {
//     "args": "0x",
//     "code_hash":
// "0xd224d5e5d26d67c0b0ffb9b32cb7de3d32e8270a921c96dd70ee4cdb1418278a",
//     "hash_type": "data"
// }
// '''
// cell_dep = '''
// {
//     "dep_type": "code",
//     "out_point": {
//         "index": "0x0",
//         "tx_hash":
// "0xf9b46fcf598799aefa627571829712b87c00e12e6c41caf4ded697629a86505a"     }
// }
// '''
//
//
// [[builtin_scripts]]
// script_name = "axon_stake"
// script = '''
// {
//     "args": "0x",
//     "code_hash":
// "0x951a3594d46e087eb118448075864da309b88f1eefb0c173ef8480a46027b9d5",
//     "hash_type": "data"
// }
// '''
// cell_dep = '''
// {
//     "dep_type": "code",
//     "out_point": {
//         "index": "0x0",
//         "tx_hash":
// "0x4a16ba9a9e1f8a567484b96c030da06be0a2132e78bf0208e1af4bc18aa23fc3"     }
// }
// '''
//
//
// [[builtin_scripts]]
// script_name = "axon_withdraw"
// script = '''
// {
//     "args": "0x",
//     "code_hash":
// "0x42dff59b19b2d964c402a66c9af7b4b1f5012aa7ba259bf12cc1bd1bd959f9f6",
//     "hash_type": "data"
// }
// '''
// cell_dep = '''
// {
//     "dep_type": "code",
//     "out_point": {
//         "index": "0x0",
//         "tx_hash":
// "0xa7ac8cc61ef6848a3f56477afa8f1f67ca84bd48b594e2575d920b8f419f84ee"     }
// }
// '''
//
//
// [[builtin_scripts]]
// script_name = "omni"
// script = '''
// {
//     "args": "0x",
//     "code_hash":
// "0x79f90bb5e892d80dd213439eeab551120eb417678824f282b4ffb5f21bad2e1e",
//     "hash_type": "type"
// }
// '''
// cell_dep = '''
// {
//     "dep_type": "code",
//     "out_point": {
//         "index": "0x0",
//         "tx_hash":
// "0x9154df4f7336402114d04495175b37390ce86a4906d2d4001cf02c3e6d97f39c"     }
// }
// '''
