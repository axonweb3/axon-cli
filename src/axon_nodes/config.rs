use std::{
    fs::{create_dir_all, write},
    path::Path,
};

use axon_protocol::{
    codec::hex_decode,
    types::{
        Address, Eip1559Transaction, Hasher, Hex, Metadata, SignedTransaction, TransactionAction,
        UnsignedTransaction, UnverifiedTransaction, ValidatorExtend, H160, U256,
    },
};
use axon_tools::consts::METADATA_CONTRACT_ADDRESS;
use clap::Args;
use ethers_core::abi::Token;
use ophelia::{PrivateKey, PublicKey, Signature, ToBlsPublicKey};
use ophelia_blst::BlsPrivateKey;
use ophelia_secp256k1::Secp256k1RecoverablePrivateKey;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use tentacle_secio::SecioKeyPair;

use crate::{
    constants::{
        CONFIG_TEMPLATE, DB_OPTION_TEMPLATE, DEFAULT_NODES_PATH, DEFAULT_NODE_KEY_PAIRS_PATH,
        GENESIS_TEMPLATE, METADATA_ABI, METADATA_TEMPLATE, VALIDATOR_TEMPLATE,
    },
    types::Result,
    utils::{
        from_json_file, read_or_create_json_template, read_or_create_plain_template, to_json_file,
    },
};

#[derive(Args, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct KeygenArgs {
    /// number of key pairs
    #[clap(short, long, default_value = "1")]
    number:       u32,
    /// the output path for key pairs file
    #[clap(short, long, default_value=*DEFAULT_NODE_KEY_PAIRS_PATH)]
    path:         String,
    /// private keys are seperated by ',', extra keys will be randomly generated
    #[clap(short = 'P', long, value_delimiter = ',')]
    private_keys: Vec<String>,
}

#[derive(Args, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ConfigGenArgs {
    /// the output path of config files
    #[clap(short, long, default_value=*DEFAULT_NODES_PATH)]
    path:           String,
    /// the path of key pairs file
    #[clap(short, long, default_value=*DEFAULT_NODE_KEY_PAIRS_PATH)]
    key_pairs_path: String,
    /// the p2p address of nodes
    #[clap(short, long, value_delimiter = ',')]
    addresses:      Vec<String>,
    /// the epoch length
    #[clap(short, long, default_value = "1000000")]
    epoch_len:      u64,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct KeyPair {
    bls_private_key:      Hex,
    bls_public_key:       Hex,
    secp256k1_public_key: Hex,
    address:              H160,
    peer_id:              String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct KeyPairsList {
    common_ref: String,
    key_pairs:  Vec<KeyPair>,
}

fn get_key_pair_from_private_key(
    common_ref: &<BlsPrivateKey as ToBlsPublicKey>::CommonReference,
    bls_private_key: BlsPrivateKey,
) -> Result<KeyPair> {
    let bls_private_key_raw = bls_private_key.to_bytes();

    let bls_public_key_raw = bls_private_key.pub_key(common_ref).to_bytes();

    let secp256k1_private_key = SecioKeyPair::secp256k1_raw_key(&bls_private_key_raw)?;
    let secp256k1_public_key = secp256k1_private_key.public_key();
    let peer_id = secp256k1_public_key.peer_id().to_base58();
    let secp256k1_public_key_raw = secp256k1_public_key.inner();

    let address = Address::from_pubkey_bytes(&secp256k1_public_key_raw)?;

    Ok(KeyPair {
        bls_private_key: Hex::encode(bls_private_key_raw),
        bls_public_key: Hex::encode(bls_public_key_raw),
        secp256k1_public_key: Hex::encode(secp256k1_public_key_raw),
        address: H160::from_slice(address.as_slice()),
        peer_id,
    })
}

pub fn generate_key_pairs(args: &KeygenArgs) -> Result<()> {
    let KeygenArgs {
        number,
        path: path_str,
        private_keys,
    } = args;
    let provided_len = private_keys.len();
    let path: &Path = path_str.as_ref();

    let common_ref = "0x0".to_string();
    let key_pairs = (0..usize::try_from(*number)?)
        .map(|i| {
            let bls_private_key = if i < provided_len {
                BlsPrivateKey::try_from(hex_decode(&private_keys[i])?.as_slice())?
            } else {
                BlsPrivateKey::generate(&mut OsRng)
            };

            get_key_pair_from_private_key(&common_ref, bls_private_key)
        })
        .collect::<Result<Vec<_>>>()?;

    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    to_json_file(
        &KeyPairsList {
            common_ref,
            key_pairs,
        },
        path,
    )?;

    log::info!("Key pairs generated");

    Ok(())
}

pub fn log_key_pairs(path: impl AsRef<Path>) -> Result<()> {
    let key_pairs_list: KeyPairsList = from_json_file(path)?;

    log::info!("Key pairs logged to stdout (to avoid being recorded)");
    println!("{}", serde_json::to_string_pretty(&key_pairs_list)?);

    Ok(())
}

fn get_tx(
    fee_per_gas: U256,
    nonce: u32,
    action: TransactionAction,
    data: Vec<u8>,
) -> UnsignedTransaction {
    UnsignedTransaction::Eip1559(Eip1559Transaction {
        nonce: nonce.into(),
        max_priority_fee_per_gas: fee_per_gas,
        gas_price: 0.into(),
        gas_limit: 30_000_000.into(),
        action,
        value: 0.into(),
        data: data.into(),
        access_list: Default::default(),
    })
}

fn sign_tx(
    private_key: &Secp256k1RecoverablePrivateKey,
    tx: UnsignedTransaction,
    chain_id: u64,
) -> SignedTransaction {
    let signature = private_key.sign_message(
        &Hasher::digest(tx.encode(Some(chain_id), None))
            .as_bytes()
            .try_into()
            .unwrap(),
    );
    let utx = UnverifiedTransaction {
        unsigned:  tx,
        signature: Some(signature.to_bytes().into()),
        chain_id:  Some(chain_id),
        hash:      Default::default(),
    }
    .calc_hash();

    SignedTransaction::from_unverified(utx, None).unwrap()
}

pub fn generate_configs(args: &ConfigGenArgs) -> Result<()> {
    let ConfigGenArgs {
        key_pairs_path,
        path: path_str,
        addresses,
        epoch_len,
    } = args;

    let path: &Path = path_str.as_ref();

    create_dir_all(path)?;

    let mut metadata =
        read_or_create_json_template(path.join("metadata_template.json"), &*METADATA_TEMPLATE)?;
    let mut genesis =
        read_or_create_json_template(path.join("genesis_template.json"), &*GENESIS_TEMPLATE)?;
    let config = read_or_create_plain_template(path.join("config_template.toml"), CONFIG_TEMPLATE)?;
    read_or_create_plain_template(path.join("default.db-options"), DB_OPTION_TEMPLATE)?;
    let ValidatorExtend {
        propose_weight: propose_weight_ref,
        vote_weight: vote_weight_ref,
        ..
    } = metadata
        .verifier_list
        .first()
        .unwrap_or(&*VALIDATOR_TEMPLATE);

    let propose_weight = *propose_weight_ref;
    let vote_weight = *vote_weight_ref;

    let KeyPairsList { key_pairs, .. } = from_json_file(key_pairs_path)?;

    let first_key_pair = if let Some(key_pair) = key_pairs.first() {
        key_pair
    } else {
        log::error!("No key pair provided, see \"axon keygen\" to generate key pairs");
        return Ok(());
    };

    metadata.to_mut().verifier_list = key_pairs
        .iter()
        .map(
            |KeyPair {
                 bls_public_key,
                 secp256k1_public_key,
                 address,
                 ..
             }| {
                ValidatorExtend {
                    bls_pub_key: bls_public_key.clone(),
                    pub_key: secp256k1_public_key.clone(),
                    address: *address,
                    propose_weight,
                    vote_weight,
                }
            },
        )
        .collect::<Vec<_>>();

    let chain_id = genesis.block.header.chain_id;
    let fee_per_gas = genesis.block.header.base_fee_per_gas;

    let private_key = Secp256k1RecoverablePrivateKey::try_from(
        first_key_pair.bls_private_key.as_bytes().as_ref(),
    )?;
    let address = first_key_pair.address;

    let append_metadata_1 = get_tx(
        fee_per_gas,
        0,
        TransactionAction::Call(METADATA_CONTRACT_ADDRESS),
        build_append_metadata_call(&metadata, 0, epoch_len - 1, 0)?,
    );
    let append_metadata_2 = get_tx(
        fee_per_gas,
        1,
        TransactionAction::Call(METADATA_CONTRACT_ADDRESS),
        build_append_metadata_call(&metadata, *epoch_len, epoch_len * 2 - 1, 1)?,
    );

    genesis.to_mut().txs = [append_metadata_1, append_metadata_2]
        .into_iter()
        .map(|tx| sign_tx(&private_key, tx, chain_id))
        .collect::<Vec<_>>();

    to_json_file(&genesis, path.join("genesis.json"))?;
    log::info!("Genesis file generated");

    let bootstraps = key_pairs.iter().enumerate().map(|(i, key_pair)| {
        let peer_id = &key_pair.peer_id;

        if i < addresses.len() {
            format!("[[network.bootstraps]]\nmulti_address = \"{}/p2p/{peer_id}\"", &addresses[i])
        } else {
            format!("[[network.bootstraps]]\nmulti_address = \"/ip4/172.17.0.1/tcp/{}/p2p/{peer_id}\"", 10000 + i)
        }
    }).reduce(|a, b| format!("{a}\n{b}")).unwrap_or_default();

    key_pairs
        .iter()
        .enumerate()
        .try_for_each(|(index, key_pair)| {
            let index = index + 1;
            let bls_private_key = &key_pair.bls_private_key;
            let config = config
                .replace("{PRIVATE_KEY}", &bls_private_key.as_string())
                .replace("{DATA_PATH}", &format!("data{index}"))
                .replace("{INIT_DISTRIBUTE_ADDRESS}", &format!("0x{address:x}"))
                .replace("{NETWORK_BOOTSTRAPS}", &bootstraps);

            write(path.join(format!("config_{index}.toml")), config.as_bytes())?;

            log::info!("Config file {index} generated");

            Result::Ok(())
        })?;

    Ok(())
}

fn build_append_metadata_call(
    metadata: &Metadata,
    start: u64,
    end: u64,
    epoch: u64,
) -> Result<Vec<u8>> {
    Ok(METADATA_ABI
        .function("appendMetadata")?
        .encode_input(&[Token::Tuple(vec![
            Token::Tuple(vec![Token::Uint(start.into()), Token::Uint(end.into())]),
            Token::Uint(epoch.into()),
            Token::Uint(metadata.gas_limit.into()),
            Token::Uint(metadata.gas_price.into()),
            Token::Uint(metadata.interval.into()),
            Token::Array(
                metadata
                    .verifier_list
                    .iter()
                    .map(|ve| {
                        Token::Tuple(vec![
                            Token::Bytes(ve.bls_pub_key.as_bytes().to_vec()),
                            Token::Bytes(ve.pub_key.as_bytes().to_vec()),
                            Token::Address(ve.address),
                            Token::Uint(ve.propose_weight.into()),
                            Token::Uint(ve.vote_weight.into()),
                        ])
                    })
                    .collect::<Vec<_>>(),
            ),
            Token::Uint(metadata.propose_ratio.into()),
            Token::Uint(metadata.prevote_ratio.into()),
            Token::Uint(metadata.precommit_ratio.into()),
            Token::Uint(metadata.brake_ratio.into()),
            Token::Uint(metadata.tx_num_limit.into()),
            Token::Uint(metadata.max_tx_size.into()),
            Token::Array(vec![]),
        ])])?)
}
