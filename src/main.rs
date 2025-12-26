mod app;
mod config;
mod id3_parser;

use app::{State, update, view};
use iced::Theme;

fn main() -> iced::Result {
    iced::application(State::new, update, view)
        .theme(|_: &_| Theme::Dark)
        .run()
}
