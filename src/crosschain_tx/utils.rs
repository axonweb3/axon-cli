use axon_protocol::types::Hex;
use ckb_sdk::{Address, AddressPayload, NetworkType};
use ckb_types::H160;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

use crate::types::Result;

pub struct CkbKeyInfo {
    pub private_key: SecretKey,
    pub public_key:  PublicKey,
    pub address:     Address,
    pub lock_args:   H160,
}

pub fn parse_private_key(private_key: String, network: NetworkType) -> Result<CkbKeyInfo> {
    let private_key = SecretKey::from_slice(Hex::from_string(private_key)?.as_bytes().as_ref())?;
    let secp256k1_context = Secp256k1::new();
    let public_key = PublicKey::from_secret_key(&secp256k1_context, &private_key);
    let address_payload = AddressPayload::from_pubkey(&public_key);
    let lock_args = H160::from_slice(address_payload.args().as_ref())?;
    let address = Address::new(network, address_payload, true);

    Ok(CkbKeyInfo {
        private_key,
        public_key,
        address,
        lock_args,
    })
}
