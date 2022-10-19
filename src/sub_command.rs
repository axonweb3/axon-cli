use std::error::Error as StdErr;

use async_trait::async_trait;
use clap::{ArgMatches, Command};

#[async_trait]
pub trait SubCommand {
    fn get_command(&self) -> Command;
    async fn exec_command(&self, matches: &ArgMatches) -> Result<(), Box<dyn StdErr>>;
}
