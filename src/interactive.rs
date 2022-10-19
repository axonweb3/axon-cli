use std::collections::HashMap;

use clap::{crate_version, Command};
use colored::*;
use log::error;
use rustyline::{error::ReadlineError, Editor};

use crate::{
    apm::Apm, axon_nodes::AxonNodes, benchmark::Benchmark, crosschain_tx::CrossChain,
    sub_command::SubCommand,
};

const HISTORY_FILE: &str = "history.txt";
const AXON_CMD: &str = "axon";
const APM_CMD: &str = "apm";
const CS_CMD: &str = "cs";
const BENCHMARK_CMD: &str = "benchmark";

#[derive(Default)]
pub struct Interactive {
    sub_cmds: HashMap<String, Box<dyn SubCommand>>,
}

impl Interactive {
    pub fn new() -> Self {
        let mut sub_cmds = HashMap::default();
        sub_cmds.insert(
            AXON_CMD.to_string(),
            Box::new(AxonNodes::default()) as Box<dyn SubCommand>,
        );

        sub_cmds.insert(
            APM_CMD.to_string(),
            Box::new(Apm::default()) as Box<dyn SubCommand>,
        );

        sub_cmds.insert(
            CS_CMD.to_string(),
            Box::new(CrossChain::default()) as Box<dyn SubCommand>,
        );

        sub_cmds.insert(
            BENCHMARK_CMD.to_string(),
            Box::new(Benchmark::default()) as Box<dyn SubCommand>,
        );
        Interactive { sub_cmds }
    }

    pub fn build_interactive(&self) -> Command {
        let subcmds: Vec<Command> = self
            .sub_cmds
            .values()
            .into_iter()
            .map(|cmd| cmd.get_command())
            .collect();

        Command::new("axon-cli")
            .version(crate_version!())
            .subcommands(subcmds)
    }

    pub async fn start(&self) {
        let mut rl = Editor::<()>::new();
        if rl.load_history(HISTORY_FILE).is_err() {
            println!("No previous history.");
        }

        let parser = self.build_interactive().no_binary_name(true);
        loop {
            let readline = rl.readline(&format!("{}", ">> ".green()));
            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line);
                    let args: Vec<&str> = line.trim().split(' ').collect();
                    let app_m = parser.clone().try_get_matches_from(args);
                    match app_m {
                        Ok(matches) => {
                            if let Some((name, matches)) = matches.subcommand() {
                                // println!("cmd name: {}", name);
                                let sub_cmd = &self.sub_cmds[&name.to_string()];
                                if let Err(err) = sub_cmd.exec_command(matches).await {
                                    error!("{}", err);
                                }
                            } else {
                                println!("cli parse error");
                            }
                        }
                        Err(err) => {
                            err.print().expect("Error writing error");
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        rl.save_history(HISTORY_FILE).unwrap();
    }
}
