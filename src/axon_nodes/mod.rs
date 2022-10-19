mod config;
mod nodes;

use std::error::Error as StdErr;

use async_trait::async_trait;
use clap::{ArgMatches, Command, FromArgMatches, Subcommand};

use self::{
    config::{generate_configs, generate_key_pairs, log_key_pairs, ConfigGenArgs, KeygenArgs},
    nodes::{ps_nodes, rm_nodes, start_nodes, stop_nodes, StartNodesArgs},
};
use crate::{
    constants::{DEFAULT_AXON_DATA_VOLUME, DEFAULT_NODE_KEY_PAIRS_PATH},
    docker::DockerApi,
    sub_command::SubCommand,
    types::{DockerArgs, RmContainerArgs},
};

#[derive(Default)]
pub struct AxonNodes {}

#[derive(Subcommand, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum AxonNodesActions {
    /// Start Axon node
    Start(StartNodesArgs),

    /// Stop axon nodes
    Stop(DockerArgs),

    /// Remove containers of Axon
    Rm(RmContainerArgs),

    /// Inspect containers of Axon
    Ps(DockerArgs),

    /// Clean chain data
    Clean {
        /// the volume of Axon's data
        #[clap(short='D', long="data", default_value=DEFAULT_AXON_DATA_VOLUME)]
        data_volume: String,

        #[clap(flatten)]
        docker_args: DockerArgs,
    },

    /// Generate key pairs for Axon nodes
    Keygen(KeygenArgs),

    /// Inspect key pairs for Axon nodes
    Keys {
        /// the path of key pairs file
        #[clap(short, long, default_value=*DEFAULT_NODE_KEY_PAIRS_PATH)]
        path: String,
    },

    /// Generate config files for Axon nodes
    ConfigGen(ConfigGenArgs),
}

#[async_trait]
impl SubCommand for AxonNodes {
    fn get_command(&self) -> Command {
        AxonNodesActions::augment_subcommands(Command::new("axon")).about("Manage Axon nodes")
    }

    async fn exec_command(&self, matches: &ArgMatches) -> Result<(), Box<dyn StdErr>> {
        match AxonNodesActions::from_arg_matches(matches)? {
            AxonNodesActions::Start(args) => {
                start_nodes(args).await?;
            }
            AxonNodesActions::Stop(args) => {
                stop_nodes(args).await?;
            }
            AxonNodesActions::Rm(args) => {
                rm_nodes(args).await?;
            }
            AxonNodesActions::Ps(args) => {
                ps_nodes(args).await?;
            }
            AxonNodesActions::Clean {
                data_volume,
                docker_args: DockerArgs { docker_uri },
            } => {
                DockerApi::new(docker_uri)?
                    .remove_one_volume(data_volume)
                    .await?;
            }
            AxonNodesActions::Keygen(args) => {
                generate_key_pairs(&args)?;
            }
            AxonNodesActions::Keys { path } => {
                log_key_pairs(&path)?;
            }
            AxonNodesActions::ConfigGen(args) => {
                generate_configs(&args)?;
            }
        }

        Ok(())
    }
}
