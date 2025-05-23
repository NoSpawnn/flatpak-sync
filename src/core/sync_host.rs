use super::flatpak::Flatpak;
use crate::cli::SshOpts;
use std::{io, net::TcpStream, path::PathBuf, process::Command};

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

        let tcp = match TcpStream::connect(format!("{}:{}", &self.hostname, self.ssh_port)) {
            Ok(stream) => stream,
            Err(e) => {
                return Err(Error::SshConnectionError {
                    tcp_error: Some(e),
                    ssh_error: None,
                });
            }
        };

        let mut session = match ssh2::Session::new() {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::SshConnectionError {
                    tcp_error: None,
                    ssh_error: Some(e),
                });
            }
        };

        session.set_tcp_stream(tcp);

        if let Err(e) = session.handshake() {
            return Err(Error::SshConnectionError {
                tcp_error: None,
                ssh_error: Some(e),
            });
        }

        if let Err(e) = session.userauth_pubkey_file(
            &self.ssh_username,
            None,
            &self.sync_key_file.as_ref().unwrap(),
            None,
        ) {
            return Err(Error::SshConnectionError {
                tcp_error: None,
                ssh_error: Some(e),
            });
        }

        self.ssh_session = Some(session);
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), Error> {
        if let Some(session) = self.ssh_session.as_mut() {
            match session.disconnect(
                Some(ssh2::DisconnectCode::ByApplication),
                "Other sync host requested to close the connection",
                None,
            ) {
                Ok(_) => {
                    self.ssh_session = None;
                    Ok(())
                }
                Err(e) => Err(Error::SshConnectionError {
                    tcp_error: None,
                    ssh_error: Some(e),
                }),
            }
        } else {
            Ok(())
        }
    }

    fn generate_sync_keypair(&mut self, force: bool) -> Result<(), Error> {
        if !force && self.sync_key_file.is_some() {
            return Err(Error::SyncKeyPairExists);
        }

        let hostname = String::from_utf8(
            Command::new("hostname")
                .output()
                .map_err(Error::IoError)?
                .stdout,
        );
        let keypair_location = format!("{SSH_KEY_DIR}/{}_sync-key", self.hostname);

        log::info!(
            "Generating new SSH keys for syncing with remote host `{}` (at {})",
            self.hostname,
            std::fs::canonicalize(&keypair_location)
                .unwrap()
                .to_string_lossy()
        );

        let output = match Command::new("ssh-keygen")
            .args([
                "-t",
                "rsa",
                "-N",
                "",
                "-f",
                &keypair_location,
                "-C",
                &format!("flatpak-sync@{}", self.hostname),
            ])
            .output()
        {
            Ok(output) => output,
            Err(e) => return Err(Error::SshKeyGenError(e)),
        };

        log::info!("Copying SSH identity to remote host `{}`", self.hostname);
        match Command::new("ssh-copy-id")
            .args([
                "-i",
                &keypair_location,
                "-p",
                &self.ssh_port.to_string(),
                &format!("{}@{}", self.ssh_username, self.hostname),
            ])
            .status()
        {
            Err(e) => return Err(Error::SshKeyCopyError(e)),
            _ => {}
        };

        self.sync_key_file = Some(PathBuf::from(&keypair_location));

        Ok(())
    }

    pub fn install_flatpaks(&self, flatpaks: &[Flatpak]) -> Result<(), Error> {
        for f in flatpaks {
            println!("Installing {}", &f.name);

            let output = match Command::new("flatpak")
                .arg("install")
                .arg(f.install_type.flag_string())
                .arg(&f.name)
                .output()
            {
                Ok(output) => output,
                Err(e) => return Err(Error::FlatpakInstallError(e)),
            };
        }

        Ok(())
    }
}

impl From<SshOpts> for SyncHost {
    fn from(opts: SshOpts) -> Self {
        SyncHost {
            ssh_username: opts.username,
            hostname: opts.remote_host,
            sync_key_file: None,
            ssh_port: opts.port,
            ssh_session: None,
        }
    }
}
