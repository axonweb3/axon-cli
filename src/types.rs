use clap::Args;
use serde::Deserialize;

use crate::constants::get_default_docker_uri;

#[derive(Args, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct DockerArgs {
    /// uri of docker service
    #[clap(short, long, default_value=get_default_docker_uri())]
    pub docker_uri: String,
}

#[derive(Args, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct RmContainerArgs {
    /// force the removal of container
    #[clap(short, long)]
    pub force: bool,

    #[clap(flatten)]
    pub docker_args: DockerArgs,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ContractJson<'a> {
    pub bytecode: &'a str,

    #[serde(borrow)]
    pub abi: &'a serde_json::value::RawValue,
}

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;
