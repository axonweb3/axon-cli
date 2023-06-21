use std::collections::HashMap;

use clap::{crate_version, Command};
use colored::*;
use log::error;
use rustyline::{error::ReadlineError, Editor};

use crate::{apm::Apm, axon_nodes::AxonNodes, benchmark::Benchmark, sub_command::SubCommand};

const HISTORY_FILE: &str = "history.txt";

#[derive(Default)]
pub struct Interactive {
    sub_cmds: HashMap<&'static str, Box<dyn SubCommand>>,
}

impl Interactive {
    pub fn new() -> Self {
        let mut sub_cmds = HashMap::default();
        sub_cmds.insert(
            "axon",
            Box::<AxonNodes>::default() as Box<dyn SubCommand>,
        );

        sub_cmds.insert("apm", Box::<Apm>::default() as Box<dyn SubCommand>);

        sub_cmds.insert(
            "benchmark",
            Box::<Benchmark>::default() as Box<dyn SubCommand>,
        );
        Interactive { sub_cmds }
    }

    pub fn build_interactive(&self) -> Command {
        let subcmds: Vec<Command> = self
            .sub_cmds
            .values()
            .map(|cmd| cmd.get_command())
            .collect();

        Command::new("axon-cli")
            .version(crate_version!())
            .subcommands(subcmds)
    }

    pub async fn start(&mut self) {
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
                                let sub_cmd = self.sub_cmds.get_mut(name).unwrap();
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
