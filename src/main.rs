mod apm;
mod axon_nodes;
mod benchmark;
mod crosschain_tx;

mod constants;
mod docker;
mod interactive;
mod sub_command;
mod types;
mod utils;

use interactive::Interactive;
use simplelog::{ConfigBuilder, TermLogger, TerminalMode};

#[tokio::main]
async fn main() {
    log::set_max_level(log::LevelFilter::Info);
    log::set_boxed_logger(Box::new(TermLogger::new(
        log::LevelFilter::Trace,
        ConfigBuilder::new()
            .set_time_level(log::LevelFilter::Debug)
            .build(),
        TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )))
    .expect("unable to set logger");
    let inter = Interactive::new();
    inter.start().await;
}
