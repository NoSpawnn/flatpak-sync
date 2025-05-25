mod views;

use crate::core::flatpak::Flatpak;
use iced::{
    Alignment, Length, Task,
    widget::{Column, container, text},
};

#[derive(Default)]
pub struct GUI {
    flatpaks: Vec<Flatpak>,
}

#[derive(Debug)]
enum Message {
    SetFlatpakList(Vec<Flatpak>),
}

impl GUI {
    pub fn run(flatpaks: Vec<Flatpak>) -> iced::Result {
        iced::application("Flatpak Sync", Self::update, Self::view)
            .theme(|_| Self::theme())
            .centered()
            .run_with(|| (Self::with_flatpak_list(flatpaks), Task::none()))
    }

    fn with_flatpak_list(flatpaks: Vec<Flatpak>) -> Self {
        Self { flatpaks }
    }

    fn view(&self) -> iced::Element<Message> {
        container(views::flatpaks_list(&self.flatpaks))
            .align_y(Alignment::Center)
            .align_x(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SetFlatpakList(flatpaks) => self.flatpaks = flatpaks,
        }
    }

    fn theme() -> iced::Theme {
        iced::Theme::Dark
    }
}
