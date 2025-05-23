mod cli;
mod core;

use clap::Parser;
use cli::SshOpts;
use core::{Error, flatpak::Flatpak, sync_host::SyncHost};

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let args = SshOpts::parse();
    let mut host = SyncHost::try_from(args)?;
    let flatpaks = Flatpak::get_local_installed()?;

    host.connect().await?;
    host.install_flatpaks(&flatpaks).await?;
    host.disconnect().await?;

    Ok(())
}
