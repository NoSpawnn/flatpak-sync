mod components;

use std::collections::HashMap;

use crate::core::flatpak::Flatpak;
use iced::{
    Alignment, Length, Subscription, Task,
    widget::{Column, container, text},
    window,
};

#[derive(Default)]
pub struct GUI {
    flatpaks: HashMap<String, Flatpak>,
}

#[derive(Debug)]
enum Message {
    SetFlatpakShouldSync(String, bool),
    CloseApplication,
}

impl GUI {
    pub fn run(flatpaks: Vec<Flatpak>) -> iced::Result {
        iced::application("Flatpak Sync", Self::update, Self::view)
            .theme(|_| Self::theme())
            .centered()
            .run_with(|| (Self::with_flatpak_list(flatpaks), Task::none()))
    }

    fn with_flatpak_list(flatpaks: Vec<Flatpak>) -> Self {
        Self {
            flatpaks: flatpaks.into_iter().map(|f| (f.name.clone(), f)).collect(),
        }
    }

    fn view(&self) -> iced::Element<Message> {
        container(components::flatpaks_list(self.flatpaks.values()))
            .align_y(Alignment::Center)
            .align_x(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SetFlatpakShouldSync(flatpak_name, state) => {
                if let Some(f) = self.flatpaks.get_mut(&flatpak_name) {
                    f.should_sync = state;
                    log::info!("Set `should_sync` of {flatpak_name} to {state}");
                }
            }
            Message::CloseApplication => {
                // Cleanup host connections
                todo!()
            }
        }
    }

    fn theme() -> iced::Theme {
        iced::Theme::Dark
    }
}
