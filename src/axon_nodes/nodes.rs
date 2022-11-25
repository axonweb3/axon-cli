use clap::Args;
use log::error;

use crate::{
    constants::{DEFAULT_AXON_DATA_VOLUME, DEFAULT_AXON_NETWORK_NAME, DEFAULT_AXON_PATH},
    docker::{DockerApi, StartAxonArgs},
    types::{DockerArgs, Result},
};

#[derive(Args, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct StartNodesArgs {
    /// number of axon nodes
    #[clap(short, long, default_value = "1")]
    number: u32,

    /// the working path of Axon
    #[clap(short='P', long, default_value=*DEFAULT_AXON_PATH)]
    path: String,

    /// the volume of Axon's data
    #[clap(short='D', long="data", default_value=DEFAULT_AXON_DATA_VOLUME)]
    data_volume: String,

    /// the network name of Axon
    #[clap(short='N', long, default_value=DEFAULT_AXON_NETWORK_NAME)]
    network: String,

    /// the starting of axon nodes' http ports
    #[clap(short, long, default_value = "8000")]
    port: u32,

    /// the starting of axon nodes's collecting ports
    #[clap(short, long, default_value = "8900")]
    collecting_port: u32,

    /// the starting of axon nodes' p2p ports
    #[clap(short = '2', long, default_value = "10000")]
    p2p_port: u32,

    #[clap(flatten)]
    docker_args: DockerArgs,
}

#[derive(Args, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct OperateNodeContainersArgs {
    /// number of axon nodes
    #[clap(short, long, default_value = "1")]
    number: u32,

    #[clap(flatten)]
    docker_args: DockerArgs,
}

#[derive(Args, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct RmNodeContainersArgs {
    /// number of axon nodes
    #[clap(short, long, default_value = "1")]
    number: u32,

    /// force the removal of containers
    #[clap(short, long)]
    force: bool,

    #[clap(flatten)]
    docker_args: DockerArgs,
}

pub async fn start_nodes(args: StartNodesArgs) -> Result<()> {
    let StartNodesArgs {
        network,
        path,
        number: num,
        port,
        collecting_port,
        p2p_port,
        docker_args: DockerArgs { docker_uri },
        data_volume,
    } = args;

    let docker_api = DockerApi::new(docker_uri)?;

    docker_api.ensure_network(&network).await?;

    if !(0..num).all(|i| {
        let path: &std::path::Path = path.as_ref();
        path.join("nodes")
            .join(format!("config_{}.toml", i + 1))
            .exists()
    }) {
        error!("Not enough config files to start {num} nodes, see \"axon keygen\" and \"axon config-gen\" to generate config files");
        return Ok(());
    }

    Ok(futures::future::join_all((0..num).map(|i| {
        docker_api.start_axon(StartAxonArgs {
            name:            format!("axon{}", i + 1),
            config_path:     format!("config_{}.toml", i + 1),
            genesis_path:    "genesis.json",
            data_volume:     &data_volume,
            path:            &path,
            port:            port + i,
            collecting_port: collecting_port + i,
            p2p_port:        p2p_port + i,
            network:         &network,
        })
    }))
    .await
    .into_iter()
    .collect::<std::result::Result<(), _>>()?)
}

pub async fn rm_nodes(args: RmNodeContainersArgs) -> Result<()> {
    let RmNodeContainersArgs {
        number,
        force,
        docker_args: DockerArgs { docker_uri },
    } = args;

    Ok(DockerApi::new(docker_uri)?
        .remove_containers((1..number + 1).map(|i| format!("axon{i}")), force)
        .await?)
}

pub async fn stop_nodes(args: OperateNodeContainersArgs) -> Result<()> {
    let OperateNodeContainersArgs {
        number,
        docker_args: DockerArgs { docker_uri },
    } = args;

    Ok(DockerApi::new(docker_uri)?
        .stop_containers((1..number + 1).map(|i| format!("axon{i}")))
        .await?)
}

pub async fn ps_nodes(args: OperateNodeContainersArgs) -> Result<()> {
    let OperateNodeContainersArgs {
        number,
        docker_args: DockerArgs { docker_uri },
    } = args;

    Ok(DockerApi::new(docker_uri)?
        .inspect_containers((1..number + 1).map(|i| format!("axon{i}")))
        .await?)
}
