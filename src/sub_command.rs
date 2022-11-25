use async_trait::async_trait;
use clap::{ArgMatches, Command};

use crate::types::Result;

#[async_trait]
pub trait SubCommand {
    fn get_command(&self) -> Command<'static>;
    async fn exec_command(&mut self, matches: &ArgMatches) -> Result<()>;
}
