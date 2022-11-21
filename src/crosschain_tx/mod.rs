mod cells;
mod create_sidechain;
mod schema;
mod schema_helper;
mod scripts;

use async_trait::async_trait;
use clap::{ArgMatches, Command, FromArgMatches, Subcommand};
use create_sidechain::create_sidechain;

use crate::{sub_command::SubCommand, types::Result};

#[derive(Default)]
pub struct Ckb {}

#[derive(Subcommand, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum CKBActions {
    Create,
}

#[async_trait]
impl SubCommand for Ckb {
    fn get_command(&self) -> Command {
        CKBActions::augment_subcommands(Command::new("ckb")).about("Interact with Nervos")
    }

    async fn exec_command(&self, matches: &ArgMatches) -> Result<()> {
        match CKBActions::from_arg_matches(matches)? {
            CKBActions::Create => {
                create_sidechain(Default::default()).await?;
            }
        }

        Ok(())
    }
}
