pub mod flatpak;
pub mod sync_host;

#[derive(Debug)]
pub enum Error {
    SyncHostError(sync_host::Error),
    FlatpakError(flatpak::Error),
    IcedError(iced::Error),
}

impl From<sync_host::Error> for Error {
    fn from(err: sync_host::Error) -> Self {
        Self::SyncHostError(err)
    }
}

impl From<flatpak::Error> for Error {
    fn from(err: flatpak::Error) -> Self {
        Self::FlatpakError(err)
    }
}

impl From<iced::Error> for Error {
    fn from(err: iced::Error) -> Self {
        Self::IcedError(err)
    }
}
