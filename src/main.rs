use crate::logger::{get_subscriber, init_subscriber};
mod cert;
mod cli;
mod config;
mod logger;
mod models;
mod server;
mod socket;
mod storage;

fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber(
        "shade".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);
    cli::run_cli()?;
    Ok(())
}
