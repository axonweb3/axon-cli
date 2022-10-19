use std::{error::Error as StdErr, process};

use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};

use crate::{docker::DockerApi, sub_command::SubCommand};

#[derive(Debug, Default)]
pub struct Apm {}

#[async_trait]
impl SubCommand for Apm {
    fn get_command(&self) -> Command {
        Command::new("apm")
            .about("Application Performance Management")
            .subcommand(
                Command::new("start")
                    .arg(
                        Arg::new("path")
                            .short('p')
                            .long("path")
                            .help("path of apm directory")
                            .required(true)
                            .takes_value(true),
                    )
                    .about("Start apm"),
            )
            .subcommand(Command::new("stop").about("Stop apm"))
            .subcommand(
                Command::new("clean")
                    .arg(
                        Arg::new("path")
                            .short('p')
                            .long("path")
                            .help("path of apm directory")
                            .required(true)
                            .takes_value(true),
                    )
                    .about("Clean apm"),
            )
    }

    async fn exec_command(&self, matches: &ArgMatches) -> Result<(), Box<dyn StdErr>> {
        match matches.subcommand() {
            Some(("start", matches)) => {
                let path = matches.value_of("path").unwrap_or("");
                println!("apm path: {}", path);

                let _output = process::Command::new(path.to_owned() + "/apm_start.sh")
                    .output()
                    .expect("Start apm exception!!!");

                println!("Start monitors!!");
                DockerApi::new_default().unwrap().start_monitor(path).await;
                println!("Sleeping 30 seconds!!");
                let thirty_secs = std::time::Duration::from_secs(30);
                std::thread::sleep(thirty_secs);
                println!("\nStart agents!!");
                DockerApi::new_default().unwrap().start_agent(path).await;
                Ok(())
            }
            Some(("stop", _)) => {
                let docker_api = DockerApi::new_default().unwrap();
                docker_api.stop_monitor().await;
                docker_api.stop_agent().await;
                Ok(())
            }
            Some(("clean", matches)) => {
                let path = matches.value_of("path").unwrap_or("");
                println!("apm path: {}", path);
                DockerApi::clean(path).await;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
