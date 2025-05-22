use std::{error::Error, process::Command};

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

fn main() -> Result<(), Box<dyn Error>> {
    let flatpaks = get_installed_flatpaks()?;

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

fn get_installed_flatpaks() -> Result<Vec<Flatpak>, Box<dyn Error>> {
    let output = Command::new("flatpak")
        .arg("list")
        .arg("--app")
        .arg("--columns=application,options")
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
