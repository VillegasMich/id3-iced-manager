mod app;
mod config;
mod id3_parser;

use app::{State, update, view};
use env_logger::{Builder, Env};

fn main() -> iced::Result {
    // Initialize logger with default level INFO if RUST_LOG is not set
    Builder::from_env(Env::default().default_filter_or("warn"))
        .format_timestamp_secs()
        .init();

    log::info!("Starting ID3 Tag Manager application");

    iced::application(State::new, update, view)
        .theme(|state: &State| state.theme())
        .run()
}
