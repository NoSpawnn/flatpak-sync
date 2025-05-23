use std::{error::Error, process::Command};

use clap::Parser;

#[derive(Debug)]
struct Flatpak {
    name: String,
    install_type: InstallType,
}

impl Flatpak {
    pub fn new(name: &str, options: &str) -> Self {
        let install_type = InstallType::from_flatpak_options(options);
        Flatpak {
            name: String::from(name),
            install_type: install_type,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum InstallType {
    System,
    User,
}

impl InstallType {
    pub fn from_flatpak_options(options: &str) -> Self {
        if options.contains("system") {
            Self::System
        } else {
            Self::User
        }
    }

    pub fn flag_string(&self) -> String {
        match self {
            InstallType::System => String::from("--system"),
            InstallType::User => String::from("--user"),
        }
    }
}

/// Sync flatpaks between your devices!
#[derive(Parser, Debug)]
#[command(version, about)]
struct SshInfo {
    /// SSH username
    #[arg(short, long)]
    user: String,

    /// SSH remote host
    #[arg(short, long)]
    remote: String,

    /// SSH remote port
    #[arg(long, default_value_t = 22)]
    port: u16,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = SshInfo::parse();

    // Generate SSH keypair for new remote host
    let hostname = String::from_utf8(Command::new("hostname").output()?.stdout)?;
    let keypair_name = format!("{}_sync-key", &args.remote);
    let output = Command::new("ssh-keygen")
        .args([
            "-t",
            "rsa",
            "-N",
            "",
            "-f",
            &keypair_name,
            "-C",
            &format!("flatpak-sync@{hostname}"),
        ])
        .output()?;

    // `ssh-copy-id` to that host (which prompts for password)
    let output = Command::new("ssh-copy-id")
        .args([
            "-i",
            &keypair_name,
            "-p",
            &args.port.to_string(),
            &format!("{}@{}", &args.user, &args.remote),
        ])
        .status()?;

    Ok(())
}

fn get_installed_flatpaks() -> Result<Vec<Flatpak>, Box<dyn Error>> {
    let output = Command::new("flatpak")
        .args(["list", "--app", "--columns=application,options"])
        .output()?
        .stdout;

    let flatpaks = String::from_utf8(output)?
        .lines()
        .flat_map(|line| {
            let mut parts = line.split('\t');
            match (parts.next(), parts.next()) {
                (Some(name), Some(options)) => Some(Flatpak::new(name, options)),
                _ => None,
            }
        })
        .collect();

    Ok(flatpaks)
}

fn install_flatpaks_on_host(flatpaks: &[Flatpak], host: String) -> Result<(), Box<dyn Error>> {
    for f in flatpaks {
        println!("Installing {}", &f.name);

        let output = Command::new("flatpak")
            .arg("install")
            .arg(f.install_type.flag_string())
            .arg(&f.name)
            .output()?;

        if output.status.success() {
            println!("{} is installed", &f.name)
        } else {
            panic!(
                "Failed to install {}: {}",
                &f.name,
                String::from_utf8(output.stderr)?
            )
        }
    }

    Ok(())
}
