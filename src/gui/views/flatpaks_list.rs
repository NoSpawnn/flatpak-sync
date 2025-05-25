use crate::{core::flatpak::Flatpak, gui::Message};
use iced::{
    Alignment,
    widget::{Column, container, text},
};

pub fn flatpaks_list(flatpaks: &[Flatpak]) -> iced::Element<Message> {
    container(Column::with_children(flatpaks.iter().map(flatpak_entry)).align_x(Alignment::Center))
        .into()
}

fn flatpak_entry(flatpak: &Flatpak) -> iced::Element<Message> {
    container(text!("{}", flatpak.name)).into()
}
