use async_trait::async_trait;
use clap::{ArgMatches, Command};

use crate::types::Result;

#[async_trait]
pub trait SubCommand {
    fn get_command(&self) -> Command;
    async fn exec_command(&self, matches: &ArgMatches) -> Result<()>;
}
