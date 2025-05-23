mod cli;
mod core;

use clap::Parser;
use cli::SshOpts;
use core::{Error, sync_host::SyncHost};

fn main() -> Result<(), Error> {
    env_logger::init();

    let args = SshOpts::parse();
    let mut sh = SyncHost::from(args);
    sh.connect()?;

    Ok(())
}
