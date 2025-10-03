use crate::logger::Logger;
mod cert;
mod cli;
mod config;
mod logger;
mod models;
mod server;
mod socket;
mod storage;

fn main() -> anyhow::Result<()> {
    Logger::new();
    cli::run_cli()?;
    Ok(())
}
