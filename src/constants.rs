use axon_protocol::types::{Block, Metadata, MetadataVersion, RichBlock, ValidatorExtend, U256};
use ethers_core::abi::Contract;
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;

use crate::{
    crosschain_tx::types::{CheckpointConfig, CreateSidechainConfigs, OmniConfig},
    types::ContractJson,
};

pub const DEFAULT_AXON_NETWORK_NAME: &str = "axon-net";
pub const DEFAULT_AXON_DATA_VOLUME: &str = "axon-data";

pub fn get_default_docker_uri() -> &'static str {
    match std::env::consts::OS {
        "linux" => "unix:///var/run/docker.sock",
        _ => "tcp://127.0.0.1:2375",
    }
}

pub fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub const BASE_FEE_PER_GAS: u64 = 0x539;

pub const CROSS_CHAIN_CONTRACT_JSON: &str = include_str!("./assets/CrossChain.json");
pub const METADATA_CONTRACT_JSON: &str = include_str!("./assets/MetadataManager.json");
pub const TOKEN_CONTRACT_JSON: &str = include_str!("./assets/MirrorToken.json");
pub const PROXY_CONTRACT_JSON: &str = include_str!("./assets/ERC1967Proxy.json");

pub const CONFIG_TEMPLATE: &str = include_str!("./assets/config_template.toml");
pub const DB_OPTION_TEMPLATE: &str = include_str!("./assets/default.db-options");

pub const APM_CONFIGS: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/assets/apm");
pub const APM_MONITOR_VARS_TEMPLATE: &str = include_str!("./assets/apm_monitor_vars_template.yaml");
pub const APM_MONITOR_PROMETHEUS_TEMPLATE: &str =
    include_str!("./assets/apm_monitor_prometheus_template.yaml");

lazy_static! {
    pub static ref HOME_PATH: &'static str = string_to_static_str(std::env::var("HOME").unwrap());
    pub static ref DEFAULT_AXON_PATH: &'static str =
        string_to_static_str(format!("{}/.config/axon", *HOME_PATH));
    pub static ref DEFAULT_NODES_PATH: &'static str =
        string_to_static_str(format!("{}/nodes", *DEFAULT_AXON_PATH));
    pub static ref DEFAULT_NODE_KEY_PAIRS_PATH: &'static str =
        string_to_static_str(format!("{}/key_pairs.json", *DEFAULT_NODES_PATH));
    pub static ref DEFAULT_BENCHMARK_PATH: &'static str =
        string_to_static_str(format!("{}/benchmark", *DEFAULT_AXON_PATH));
    pub static ref DEFAULT_APM_PATH: &'static str =
        string_to_static_str(format!("{}/apm", *DEFAULT_AXON_PATH));
    pub static ref DEFAULT_APM_MONITOR_PATH: &'static str =
        string_to_static_str(format!("{}/axon-apm-monitor", *DEFAULT_APM_PATH));
    pub static ref DEFAULT_APM_AGENT_PATH: &'static str =
        string_to_static_str(format!("{}/axon-apm-agent", *DEFAULT_APM_PATH));
    pub static ref DEFAULT_SIDECHAIN_CONFIG_PATH: &'static str =
        string_to_static_str(format!("{}/sidechain", *DEFAULT_AXON_PATH));
    pub static ref DEFAULT_CREAET_SIDECHAIN_CONFIG_PATH: &'static str = string_to_static_str(
        format!("{}/create-sidechain.json", *DEFAULT_SIDECHAIN_CONFIG_PATH)
    );
    pub static ref GENESIS_TEMPLATE: RichBlock = {
        let mut block = Block::default();
        block.header.base_fee_per_gas = U256::from(BASE_FEE_PER_GAS);
        block.header.chain_id = 2022;
        block.header.timestamp = 1639459018;

        RichBlock {
            block,
            txs: Vec::new(),
        }
    };
    pub static ref VALIDATOR_TEMPLATE: ValidatorExtend = ValidatorExtend {
        propose_weight: 1,
        vote_weight: 1,
        ..Default::default()
    };
    pub static ref METADATA_TEMPLATE: Metadata = Metadata {
        version: MetadataVersion {
            start: 0,
            end:   99999999,
        },
        gas_limit: 4294967295000,
        gas_price: 1,
        interval: 3000,
        propose_ratio: 15,
        prevote_ratio: 10,
        precommit_ratio: 10,
        brake_ratio: 10,
        tx_num_limit: 20000,
        max_tx_size: 409600000,
        verifier_list: vec![VALIDATOR_TEMPLATE.clone()],
        ..Default::default()
    };
    pub static ref CREATE_SIDECHAIN_CONFIG_TEMPLATE: CreateSidechainConfigs =
        CreateSidechainConfigs {
            checkpoint_config: CheckpointConfig {
                base_reward: "0".to_string(),
                ..Default::default()
            },
            omni_config: OmniConfig {
                max_supply: "0".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
    pub static ref CROSS_CHAIN_CONTRACT: ContractJson<'static> =
        serde_json::from_str(CROSS_CHAIN_CONTRACT_JSON).unwrap();
    pub static ref CROSS_CHAIN_ABI: Contract =
        Contract::load(CROSS_CHAIN_CONTRACT.abi.get().as_bytes()).unwrap();
    pub static ref METADATA_CONTRACT: ContractJson<'static> =
        serde_json::from_str(METADATA_CONTRACT_JSON).unwrap();
    pub static ref METADATA_ABI: Contract =
        Contract::load(METADATA_CONTRACT.abi.get().as_bytes()).unwrap();
    pub static ref TOKEN_CONTRACT: ContractJson<'static> =
        serde_json::from_str(TOKEN_CONTRACT_JSON).unwrap();
    pub static ref TOKEN_ABI: Contract =
        Contract::load(TOKEN_CONTRACT.abi.get().as_bytes()).unwrap();
    pub static ref PROXY_CONTRACT: ContractJson<'static> =
        serde_json::from_str(PROXY_CONTRACT_JSON).unwrap();
    pub static ref PROXY_ABI: Contract =
        Contract::load(PROXY_CONTRACT.abi.get().as_bytes()).unwrap();
}

pub const AXON_IMAGE_NAME: &str = "hanssen0/axon";
pub const AXON_IMAGE_TAG: &str = "a53490b";
pub const BENCHMARK_IMAGE_NAME: &str = "zhengjianhui/axon-benchmark";
pub const BENCHMARK_IMAGE_TAG: &str = "latest";
