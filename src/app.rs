use crate::config::{load_recent_files, save_recent_files};
use crate::id3_parser::{parse_id3, AudioMetadata, ParseError};
use iced::{
    Element, Length, Padding, Task, alignment::{Horizontal, Vertical}, widget::{
        Column, Space, button, column, container, row, scrollable, text
    }
};
use iced::widget::button as button_widget;
use std::path::PathBuf;

/// Application state
#[derive(Debug, Clone)]
pub struct State {
    file_path: Option<PathBuf>,
    metadata: Option<AudioMetadata>,
    error: Option<String>,
    recent_files: Vec<PathBuf>, // Max 5 most recent files
}

impl State {
    pub fn new() -> Self {
        let recent_files = load_recent_files();
        Self {
            file_path: None,
            metadata: None,
            error: None,
            recent_files,
        }
    }

    /// Add a file to recent files list (max 5) and save to disk
    fn add_to_recent_files(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_files.retain(|p| p != &path);
        // Add to front
        self.recent_files.insert(0, path);
        // Keep only 5 most recent
        if self.recent_files.len() > 5 {
            self.recent_files.truncate(5);
        }
        // Save to disk
        save_recent_files(&self.recent_files);
    }

    /// Remove a file from recent files and save to disk
    fn remove_from_recent_files(&mut self, path: &PathBuf) {
        self.recent_files.retain(|p| p != path);
        save_recent_files(&self.recent_files);
    }
}

/// Messages that the application can handle
#[derive(Debug, Clone)]
pub enum Message {
    OpenFileDialog,
    FileSelected(Option<PathBuf>),
    SelectRecentFile(PathBuf),
    MetadataParsed(Result<AudioMetadata, ParseError>),
}

/// Update function that handles messages and modifies state
pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::OpenFileDialog => {
            Task::perform(open_file_dialog(), Message::FileSelected)
        }
        Message::FileSelected(path) => {
            if let Some(path) = path {
                state.file_path = Some(path.clone());
                state.add_to_recent_files(path.clone());
                state.error = None;
                // Automatically parse when file is selected
                return Task::perform(parse_file_async(path), Message::MetadataParsed);
            }
            Task::none()
        }
        Message::SelectRecentFile(path) => {
            if path.exists() {
                state.file_path = Some(path.clone());
                state.add_to_recent_files(path.clone());
                state.error = None;
                return Task::perform(parse_file_async(path), Message::MetadataParsed);
            } else {
                state.error = Some("File no longer exists".to_string());
                // Remove from recent files if it doesn't exist
                state.remove_from_recent_files(&path);
            }
            Task::none()
        }
        Message::MetadataParsed(result) => {
            match result {
                Ok(metadata) => {
                    state.metadata = Some(metadata);
                    state.error = None;
                }
                Err(e) => {
                    state.error = Some(e.to_string());
                    state.metadata = None;
                }
            }
            Task::none()
        }
    }
}

/// Async function to open file dialog
async fn open_file_dialog() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("Audio Files", &["mp3", "flac", "m4a", "aac", "ogg", "wav"])
        .pick_file()
        .await
        .map(|file| file.path().to_path_buf())
}

/// Async function to parse ID3 tags
async fn parse_file_async(path: PathBuf) -> Result<AudioMetadata, ParseError> {
    parse_id3(path)
}

/// View function that builds the UI
pub fn view(state: &State) -> Element<'_, Message> {
    let file_picker = button("Select Audio File")
        .on_press(Message::OpenFileDialog)
        .padding(10);

    // Display current file path (read-only)
    let file_path_display = if let Some(ref path) = state.file_path {
        container(
            text(path.to_string_lossy())
                .style(|_theme| {
                    iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.6, 0.8, 1.0)),
                    }
                })
        )
        .padding(10)
        .style(container::rounded_box)
        .width(Length::Fill)
    } else {
        container(
            text("No file selected")
                .style(|_theme| {
                    iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                    }
                })
        )
        .padding(10)
        .style(container::rounded_box)
        .width(Length::Fill)
    };

    let mut content = column![
        text("ID3 Tag Manager")
            .size(32)
            .align_x(Horizontal::Center),
        Space::new().height(20.0),
        file_picker,
        Space::new().height(10.0),
        file_path_display,
    ]
    .spacing(10)
    .padding(20)
    .width(Length::Fill);

    // Add recent files section
    if !state.recent_files.is_empty() {
        content = content.push(Space::new().height(20.0));
        content = content.push(build_recent_files_view(&state.recent_files, &state.file_path));
    }

    // Show error if any
    if let Some(ref error) = state.error {
        content = content.push(
            container(text(error).style(|_theme| {
                iced::widget::text::Style {
                    color: Some(iced::Color::from_rgb(1.0, 0.3, 0.3)),
                }
            }))
            .padding(10)
            .style(container::rounded_box),
        );
    }

    // Show metadata if available
    if let Some(ref metadata) = state.metadata {
        content = content.push(Space::new().height(20));
        content = content.push(build_metadata_view(metadata));
    } else {
        content = content.push(
            container(
                text("No metadata loaded. Select an audio file to view its ID3 tags.").align_x(Horizontal::Center),
            )
            .padding(20)
            .width(Length::Fill),
        );
    }

    container(
        scrollable(
            content
                .width(Length::Fill)
                .align_x(Horizontal::Center)
        )
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(Padding::new(20.0))
    .into()
}

/// Build the recent files view
fn build_recent_files_view<'a>(recent_files: &'a [PathBuf], current_file: &'a Option<PathBuf>) -> Element<'a, Message> {
    let mut files_column = Column::new()
        .spacing(5)
        .width(Length::Fill);

    for path in recent_files {
        // Extract just the filename from the path
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let path_clone = path.clone();
        
        // Check if this is the currently selected file
        let is_selected = current_file.as_ref().map_or(false, |current| current == path);
        
        // Apply different styling for selected file
        let file_button = if is_selected {
            button(
                text(filename)
                    .width(Length::Fill)
                    .align_x(Horizontal::Left)
            )
            .on_press(Message::SelectRecentFile(path_clone))
            .padding(8)
            .width(Length::Fill)
            .style(|theme: &iced::Theme, status: button_widget::Status| {
                // Use a stronger, more vibrant color for selected file
                let selected_color = iced::Color::from_rgb(0.2, 0.6, 1.0); // Bright blue
                button_widget::Style {
                    background: Some(selected_color.into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        width: 1.0,
                        color: iced::Color::from_rgb(0.1, 0.5, 0.9),
                        radius: 4.0.into(),
                    },
                    ..button_widget::primary(theme, status)
                }
            })
        } else {
            button(
                text(filename)
                    .width(Length::Fill)
                    .align_x(Horizontal::Left)
            )
            .on_press(Message::SelectRecentFile(path_clone))
            .padding(8)
            .width(Length::Fill)
        };
        
        files_column = files_column.push(file_button);
    }

    container(
        column![
            text(format!("Recent Files ({})", recent_files.len()))
                .size(20)
                .align_x(Horizontal::Center),
            Space::new().height(10.0),
            files_column,
        ]
        .spacing(10)
        .width(Length::Fill),
    )
    .padding(15)
    .style(container::rounded_box)
    .width(Length::Fill)
    .into()
}

/// Build the metadata display view
fn build_metadata_view(metadata: &AudioMetadata) -> Element<'_, Message> {
    let mut metadata_rows = Column::new()
        .spacing(10)
        .width(Length::Fill);

    // Helper function to create a metadata row element
    fn create_row<'a>(label: &'static str, value: String) -> Element<'a, Message> {
        row![
            text(label)
                .width(Length::Fixed(150.0))
                .style(|_theme| {
                    iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    }
                }),
            text(value).width(Length::Fill),
        ]
        .spacing(10)
        .align_y(Vertical::Center)
        .into()
    }

    // Add rows for each metadata field
    if let Some(ref title) = metadata.title {
        if !title.is_empty() {
            metadata_rows = metadata_rows.push(create_row("Title:", title.clone()));
        }
    }

    if let Some(duration) = metadata.duration {
        metadata_rows = metadata_rows.push(create_row("Duration:", duration.to_string()));
    }

    if let Some(ref artist) = metadata.artist {
        if !artist.is_empty() {
            metadata_rows = metadata_rows.push(create_row("Artist:", artist.clone()));
        }
    }

    if let Some(ref album) = metadata.album {
        if !album.is_empty() {
            metadata_rows = metadata_rows.push(create_row("Album:", album.clone()));
        }
    }

    if let Some(ref album_artist) = metadata.album_artist {
        if !album_artist.is_empty() {
            metadata_rows = metadata_rows.push(create_row("Album Artist:", album_artist.clone()));
        }
    }

    if let Some(ref composer) = metadata.composer {
        if !composer.is_empty() {
            metadata_rows = metadata_rows.push(create_row("Composer:", composer.clone()));
        }
    }

    if let Some(ref genre) = metadata.genre {
        if !genre.is_empty() {
            metadata_rows = metadata_rows.push(create_row("Genre:", genre.clone()));
        }
    }

    if let Some(year) = metadata.year {
        metadata_rows = metadata_rows.push(create_row("Year:", year.to_string()));
    }

    if let Some(track) = metadata.track {
        metadata_rows = metadata_rows.push(create_row("Track:", track.to_string()));
    }

    if let Some(ref comment) = metadata.comment {
        if !comment.is_empty() {
            metadata_rows = metadata_rows.push(create_row("Comment:", comment.clone()));
        }
    }

    container(
        column![
            text("Metadata")
                .size(24)
                .align_x(Horizontal::Center),
            Space::new().height(15.0),
            container(metadata_rows)
                .padding(15)
                .style(container::rounded_box)
                .width(Length::Fill),
        ]
        .spacing(10)
        .width(Length::Fill),
    )
    .padding(20)
    .style(container::rounded_box)
    .width(Length::Fill)
    .into()
}
