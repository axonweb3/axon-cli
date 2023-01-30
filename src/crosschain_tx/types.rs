use ckb_jsonrpc_types::{Deserialize, Serialize};
use ckb_types::{bytes::Bytes, packed, prelude::*, H160, H256};
use molecule::prelude::*;

use super::schema::{self, Byte97, IdentityBuilder, StakeInfoBuilder, StakeInfoVecBuilder};
use crate::types::Result;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct OmniConfig {
    pub version:    u8,
    pub max_supply: String,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct CheckpointConfig {
    pub version:              u8,
    pub period_interval:      u32,
    pub era_period:           u32,
    pub base_reward:          String,
    pub half_period:          u64,
    pub common_ref:           Bytes,
    pub withdrawal_lock_hash: H256,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct StakeConfig {
    pub version:     u8,
    pub stake_infos: Vec<StakeInfo>,
    pub quoram_size: u8,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct CreateSidechainConfigs {
    pub omni_config:       OmniConfig,
    pub checkpoint_config: CheckpointConfig,
    pub stake_config:      StakeConfig,
    pub admin_identity:    Identity,
}
