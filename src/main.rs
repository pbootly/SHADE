mod cli;
mod cert;
mod server;
mod config;
mod storage;
mod socket;

fn main() -> anyhow::Result<()> {
    cli::run_cli()?;
    Ok(())
}

