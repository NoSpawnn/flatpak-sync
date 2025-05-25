use clap::Parser;

/// Sync flatpaks between your devices!
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct SshOpts {
    /// SSH username
    #[arg(short, long)]
    pub username: String,

    /// SSH remote host
    #[arg(short, long)]
    pub remote_host: String,

    /// SSH remote port
    #[arg(long, default_value_t = 22)]
    pub port: u16,

    /// Flatpaks to exclude from syncing (blacklist)
    #[arg(short, long, value_delimiter = ',', num_args=1..)]
    pub exclude: Vec<String>,
}
