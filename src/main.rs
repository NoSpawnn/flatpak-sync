use std::process::Command;

#[derive(Debug)]
struct Flatpak {
    name: String,
    install_type: InstallType,
}

#[derive(Debug)]
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
        Flatpak {
            name: String::from(name),
            install_type: InstallType::from_flatpak_options(options),
        }
    }
}

fn main() {
    let c = Command::new("flatpak")
        .arg("list")
        .arg("--columns=application,options")
        .output()
        .expect("Failed to retrieve installed flatpaks");

    let list: Vec<_> = String::from_utf8(c.stdout)
        .unwrap()
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();

    let flatpaks = list.chunks(2).flat_map(|ss| {
        if let [s1, s2] = ss {
            Some(Flatpak::new(s1, s2))
        } else {
            None
        }
    });

    for f in flatpaks {
        dbg!(&f);
    }
}
