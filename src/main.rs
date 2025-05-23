mod cli;
mod core;

use clap::Parser;
use cli::SshOpts;
use core::{Error, flatpak::Flatpak, sync_host::SyncHost};

fn main() -> Result<(), Error> {
    env_logger::init();

    let args = SshOpts::parse();
    let mut sh = SyncHost::try_from(args)?;
    let flatpaks = Flatpak::get_local_installed()?;

    sh.connect()?;
    sh.install_flatpaks(&flatpaks)?;

    Ok(())
}
