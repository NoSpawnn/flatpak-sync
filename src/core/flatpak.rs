use std::{io, process::Command, string};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ListParseError(string::FromUtf8Error),
}

#[derive(Debug)]
pub struct Flatpak {
    pub name: String,
    pub install_type: InstallType,
    pub should_sync: bool,
}

impl Flatpak {
    pub fn new(name: &str, options: &str) -> Self {
        let install_type = InstallType::from_flatpak_options(options);
        Flatpak {
            name: String::from(name),
            install_type: install_type,
            should_sync: true,
        }
    }

    pub fn get_local_installed() -> Result<Vec<Self>, Error> {
        let output = Command::new("flatpak")
            .args(["list", "--app", "--columns=application,options"])
            .output()
            .map_err(Error::IoError)?
            .stdout;

        let flatpaks = String::from_utf8(output)
            .map_err(Error::ListParseError)?
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
}

#[derive(Debug, Clone, Copy)]
pub enum InstallType {
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
