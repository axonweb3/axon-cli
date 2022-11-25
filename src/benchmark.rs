use async_trait::async_trait;
use clap::{ArgMatches, Args, Command, FromArgMatches, Subcommand};

use crate::{
    constants::{DEFAULT_AXON_NETWORK_NAME, DEFAULT_BENCHMARK_PATH},
    docker::DockerApi,
    sub_command::SubCommand,
    types::{DockerArgs, Result, RmContainerArgs},
};

#[derive(Default)]
pub struct Benchmark {}

#[derive(Subcommand, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum BenchmarkActions {
    /// Start benchmark
    Start(StartBenchmarkArgs),

    /// Stop benchmark
    Stop(DockerArgs),

    /// Remove the container of benchmark
    Rm(RmContainerArgs),

    /// Inspect the container of benchmark
    Ps(DockerArgs),
}

#[derive(Args, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct StartBenchmarkArgs {
    /// the working path of benchmark
    #[clap(short, long, default_value=*DEFAULT_BENCHMARK_PATH)]
    path: String,

    /// the http endpoint of Axon
    #[clap(short = 'H', long, default_value = "http://axon_single:8000")]
    http_endpoint: String,

    /// the network name of Axon
    #[clap(short='N', long, default_value=DEFAULT_AXON_NETWORK_NAME)]
    network: String,

    #[clap(flatten)]
    docker_args: DockerArgs,
}

#[async_trait]
impl SubCommand for Benchmark {
    fn get_command(&self) -> Command<'static> {
        BenchmarkActions::augment_subcommands(Command::new("benchmark")).about("Manage benchmark")
    }

    async fn exec_command(&mut self, matches: &ArgMatches) -> Result<()> {
        match BenchmarkActions::from_arg_matches(matches)? {
            BenchmarkActions::Start(args) => {
                let StartBenchmarkArgs {
                    path,
                    http_endpoint,
                    network,
                    docker_args: DockerArgs { docker_uri },
                } = args;

                DockerApi::new(docker_uri)?
                    .start_benchmark(path, http_endpoint, network)
                    .await?;
            }
            BenchmarkActions::Rm(args) => {
                Benchmark::rm_benchmark(args).await?;
            }
            BenchmarkActions::Stop(args) => {
                Benchmark::stop_benchmark(args).await?;
            }
            BenchmarkActions::Ps(args) => {
                Benchmark::ps_benchmark(args).await?;
            }
        }

        Ok(())
    }
}

impl Benchmark {
    async fn rm_benchmark(args: RmContainerArgs) -> Result<()> {
        let RmContainerArgs {
            force,
            docker_args: DockerArgs { docker_uri },
        } = args;

        Ok(DockerApi::new(docker_uri)?
            .remove_containers(["benchmark"], force)
            .await?)
    }

    async fn stop_benchmark(args: DockerArgs) -> Result<()> {
        let DockerArgs { docker_uri } = args;

        Ok(DockerApi::new(docker_uri)?
            .stop_containers(["benchmark"])
            .await?)
    }

    async fn ps_benchmark(args: DockerArgs) -> Result<()> {
        let DockerArgs { docker_uri } = args;

        Ok(DockerApi::new(docker_uri)?
            .inspect_containers(["benchmark"])
            .await?)
    }
}
