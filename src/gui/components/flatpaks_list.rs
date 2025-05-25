use crate::{core::flatpak::Flatpak, gui::Message};
use iced::{
    Alignment,
    widget::{Column, checkbox, container, row, text},
};

pub fn flatpaks_list<'a>(
    flatpaks: impl Iterator<Item = &'a Flatpak> + 'a,
) -> iced::Element<'a, Message> {
    container(Column::with_children(flatpaks.map(flatpak_entry)).align_x(Alignment::Center)).into()
}

fn flatpak_entry(flatpak: &Flatpak) -> iced::Element<Message> {
    container(row![
        checkbox("", flatpak.should_sync)
            .on_toggle(|state| { Message::SetFlatpakShouldSync(flatpak.name.clone(), state) }),
        text!("{}", flatpak.name),
    ])
    .into()
}
