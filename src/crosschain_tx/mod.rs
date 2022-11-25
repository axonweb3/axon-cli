mod cells;
mod ckb_context;
mod constants;
mod create_sidechain;
mod schema;
mod schema_helper;
pub mod types;
mod utils;

use std::{collections::HashMap, io::Write};

use async_trait::async_trait;
use clap::{ArgMatches, Command, FromArgMatches, Subcommand};
use log::info;

use self::{ckb_context::DefaultSdkCkbContext, create_sidechain::CreateSidechainArgs};
use crate::{crosschain_tx::ckb_context::CkbContext, sub_command::SubCommand, types::Result};

#[derive(Default)]
pub struct Ckb {
    contexts: HashMap<String, DefaultSdkCkbContext>,
}

impl Ckb {
    async fn get_context(&mut self, ckb_uri: String) -> Result<&DefaultSdkCkbContext> {
        if !self.contexts.contains_key(&ckb_uri) {
            let context = DefaultSdkCkbContext::new(ckb_uri.clone()).await?;
            self.contexts.insert(ckb_uri.clone(), context);
        }

        Ok(self.contexts.get(&ckb_uri).unwrap())
    }
}

#[derive(Subcommand, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum CKBActions {
    Create(CreateSidechainArgs),
}

#[async_trait]
impl SubCommand for Ckb {
    fn get_command(&self) -> Command<'static> {
        CKBActions::augment_subcommands(Command::new("ckb")).about("Interact with Nervos")
    }

    async fn exec_command(&mut self, matches: &ArgMatches) -> Result<()> {
        let (tx, should_skip_confirm, ckb_uri) = match CKBActions::from_arg_matches(matches)? {
            CKBActions::Create(args) => {
                let ckb_uri = args.ckb_uri.clone();
                let should_skip_confirm = args.should_skip_confirm;

                (
                    self.create_sidechain(args).await?,
                    should_skip_confirm,
                    ckb_uri,
                )
            }
        };

        info!(
            "Prepared transaction: {}",
            serde_json::to_string_pretty(&tx)?
        );
        let context = self.get_context(ckb_uri).await?;

        if !should_skip_confirm {
            print!("Sure to send this transaction? [y/N] ");
            std::io::stdout().flush()?;
            let mut confirmation = String::new();
            std::io::stdin().read_line(&mut confirmation)?;

            match confirmation.as_str().trim() {
                "y" => (),
                _ => {
                    context.reset()?;
                    info!("Aborted");
                    return Ok(());
                }
            }
        }

        let hash = context.send_transaction(tx).await?;
        info!("Transaction hash: 0x{hash}");

        Ok(())
    }
}
