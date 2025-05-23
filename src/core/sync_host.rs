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
        ssh_error: Option<ssh2::Error>,
    },
    NoSyncKey,
    NoSshSession,
}

impl From<ssh2::Error> for Error {
    fn from(err: ssh2::Error) -> Self {
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
    ssh_session: Option<ssh2::Session>,
}

impl SyncHost {
    pub fn connect(&mut self) -> Result<(), Error> {
        if self.sync_key_file.is_none() {
            return Err(Error::NoSyncKey);
        }

        let tcp =
            TcpStream::connect(format!("{}:{}", &self.hostname, self.ssh_port)).map_err(|e| {
                Error::SshConnectionError {
                    tcp_error: Some(e),
                    ssh_error: None,
                }
            })?;

        let mut session = ssh2::Session::new()?;

        session.set_tcp_stream(tcp);
        session.handshake()?;
        session.userauth_pubkey_file(
            &self.ssh_username,
            None,
            &self.sync_key_file.as_ref().unwrap(),
            None,
        )?;

        self.ssh_session = Some(session);
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), Error> {
        if let Some(session) = self.ssh_session.as_mut() {
            session.disconnect(
                Some(ssh2::DisconnectCode::ByApplication),
                &format!("{} requested to close the connection", self.hostname),
                None,
            )?;
            self.ssh_session = None;
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
                &format!("{}@{}", self.ssh_username, self.hostname),
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

    pub fn install_flatpaks(&mut self, flatpaks: &[Flatpak]) -> Result<(), Error> {
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

        let mut channel = self
            .ssh_session
            .as_ref()
            .ok_or(Error::NoSshSession)?
            .channel_session()?;

        // Maybe return a list of what failed to install (if anything)?
        for flatpak in flatpaks {
            log::info!("Installing {} on host `{}`", flatpak.name, self.hostname);

            let cmd_string = format!(
                "flatpak install {} {}",
                flatpak.install_type.flag_string(),
                &flatpak.name,
            );
            let mut cmd_output = String::new();

            channel.exec(&cmd_string)?;
            let _ = channel.read_to_string(&mut cmd_output);
        }

        channel.send_eof()?;
        channel.wait_eof()?;
        channel.close()?;
        channel.wait_close()?;

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
