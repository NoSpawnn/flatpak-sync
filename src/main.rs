use std::{error::Error, path::PathBuf, process::Command};

#[derive(Debug)]
struct Flatpak {
    name: String,
    install_type: InstallType,
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

fn main() -> Result<(), Box<dyn Error>> {
    let flatpaks = get_installed_flatpaks()?;

    for f in flatpaks {
        dbg!(&f);
    }

    Ok(())
}

fn get_installed_flatpaks() -> Result<Vec<Flatpak>, Box<dyn Error>> {
    let cmd = Command::new("flatpak")
        .arg("list")
        .arg("--app")
        .arg("--columns=application,options")
        .output()?;

    let flatpaks = String::from_utf8(cmd.stdout)?
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
