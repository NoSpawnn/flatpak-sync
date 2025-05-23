use super::flatpak::Flatpak;
use crate::cli::SshOpts;
use std::{
    io::{self, Read},
    net::TcpStream,
    path::PathBuf,
    process::Command,
};

pub(crate) const SSH_KEY_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/sync-keys");

#[derive(Debug)]
pub enum Error {
    SyncKeyPairExists,
    FlatpakInstallError(io::Error),
    IoError(io::Error),
    SshKeyGenError(io::Error),
    SshKeyCopyError(io::Error),
    SshConnectionError {
        tcp_error: Option<io::Error>,
        ssh_error: Option<openssh::Error>,
    },
    NoSyncKey,
    NoSshSession,
}

impl From<openssh::Error> for Error {
    fn from(err: openssh::Error) -> Self {
        Error::SshConnectionError {
            tcp_error: None,
            ssh_error: Some(err),
        }
    }
}

pub struct SyncHost {
    pub ssh_username: String,
    pub hostname: String,
    pub sync_key_file: Option<PathBuf>,
    ssh_port: u16,
    ssh_session: Option<openssh::Session>,
}

impl SyncHost {
    pub async fn connect(&mut self) -> Result<(), Error> {
        if self.sync_key_file.is_none() {
            return Err(Error::NoSyncKey);
        }

        let keyfile = self.sync_key_file.as_ref().unwrap();
        let session = openssh::SessionBuilder::default()
            .keyfile(keyfile)
            .connect(format!(
                "ssh://{}@{}:{}",
                &self.ssh_username, &self.hostname, self.ssh_port
            ))
            .await?;

        self.ssh_session = Some(session);
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), Error> {
        if let Some(session) = self.ssh_session.take() {
            session.close().await?;
        }

        Ok(())
    }

    fn generate_sync_keypair(&mut self, force: bool) -> Result<(), Error> {
        if !force && self.sync_key_file.is_some() {
            return Err(Error::SyncKeyPairExists);
        }

        match std::fs::create_dir(SSH_KEY_DIR) {
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return Err(Error::IoError(e)),
            _ => {}
        };

        let keypair_location = std::fs::canonicalize(SSH_KEY_DIR)
            .unwrap()
            .join(format!("{}_sync-key", self.hostname));

        log::info!(
            "Generating new SSH keys for syncing with remote host `{}` (at {})",
            self.hostname,
            &keypair_location.to_string_lossy()
        );

        let output = Command::new("ssh-keygen")
            .args([
                "-t",
                "rsa",
                "-N",
                "",
                "-f",
                &keypair_location.to_string_lossy(),
                "-C",
                &format!("flatpak-sync@{}", self.hostname),
            ])
            .output()
            .map_err(Error::SshKeyGenError)?;

        log::info!("Copying SSH identity to remote host `{}`", self.hostname);
        Command::new("ssh-copy-id")
            .args([
                "-i",
                &keypair_location.to_string_lossy(),
                "-p",
                &self.ssh_port.to_string(),
                &format!("{}@{}", &self.ssh_username, &self.hostname),
            ])
            .status()
            .map_err(Error::SshKeyCopyError)?;

        self.sync_key_file = Some(PathBuf::from(&keypair_location));

        log::info!(
            "Identity file for {} set to {}",
            self.hostname,
            &keypair_location.to_string_lossy()
        );

        Ok(())
    }

    pub async fn install_flatpaks(&mut self, flatpaks: &[Flatpak]) -> Result<(), Error> {
        if self.ssh_session.is_none() {
            log::warn!("Could not install flatpaks due to no SSH session");
            return Err(Error::NoSshSession);
        }

        log::info!(
            "Attempting to install flatpaks {} on host `{}`",
            flatpaks
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            self.hostname
        );

        // Maybe return a list of what failed to install (if anything)?
        let session = self.ssh_session.as_ref().unwrap();
        for flatpak in flatpaks {
            log::info!("Installing {} on host `{}`", flatpak.name, self.hostname);

            let output = session
                .command("flatpak")
                .args([
                    "install",
                    &flatpak.install_type.flag_string(),
                    &flatpak.name,
                    "-y",
                ])
                .output()
                .await?;

            if output.status.success() {
                log::info!("Installed {} on host `{}`", flatpak.name, self.hostname)
            } else {
                log::warn!(
                    "Failed to install {} on host `{}`",
                    flatpak.name,
                    self.hostname
                )
            }
        }

        Ok(())
    }
}

impl TryFrom<SshOpts> for SyncHost {
    type Error = Error;

    fn try_from(opts: SshOpts) -> Result<Self, Self::Error> {
        let mut sh = SyncHost {
            ssh_username: opts.username,
            hostname: opts.remote_host,
            sync_key_file: None,
            ssh_port: opts.port,
            ssh_session: None,
        };
        sh.generate_sync_keypair(false)?;
        Ok(sh)
    }
}
