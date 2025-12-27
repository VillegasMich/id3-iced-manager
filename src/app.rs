use crate::config::{load_config, load_recent_files, save_config, save_recent_files, AppTheme};
use crate::id3_parser::{parse_id3, AudioMetadata, ParseError};
use iced::{
    Element, Length, Padding, Task, alignment::{Horizontal, Vertical}, widget::{
        Column, Space, button, column, container, row, scrollable, text, image
    }
};
use iced::widget::button as button_widget;
use iced::widget::image::Handle;
use std::path::PathBuf;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Application state
#[derive(Debug, Clone)]
pub struct State {
    file_path: Option<PathBuf>,
    metadata: Option<AudioMetadata>,
    error: Option<String>,
    recent_files: Vec<PathBuf>, // Max 5 most recent files
    theme: AppTheme,            // Dark or Light theme
    zoom: f32,                  // Zoom level (1.0 = 100%)
    settings_open: bool,        // Whether settings panel is visible
}

impl State {
    pub fn new() -> Self {
        log::info!("Initializing application state");
        let config = load_config();
        let recent_files = load_recent_files();
        log::info!("Loaded {} recent files, theme: {:?}, zoom: {:.1}", 
            recent_files.len(), config.theme, config.zoom);
        Self {
            file_path: None,
            metadata: None,
            error: None,
            recent_files,
            theme: config.theme,
            zoom: config.zoom,
            settings_open: false,
        }
    }

    /// Get the current theme
    pub fn theme(&self) -> iced::Theme {
        self.theme.to_iced_theme()
    }

    /// Save current settings to disk
    fn save_settings(&self) {
        log::debug!("Saving settings: theme={:?}, zoom={:.1}", self.theme, self.zoom);
        let mut config = load_config();
        config.theme = self.theme.clone();
        config.zoom = self.zoom;
        save_config(&config);
    }

    /// Add a file to recent files list (max 5) and save to disk
    fn add_to_recent_files(&mut self, path: PathBuf) {
        log::debug!("Adding file to recent files: {:?}", path);
        // Remove if already exists
        self.recent_files.retain(|p| p != &path);
        // Add to front
        self.recent_files.insert(0, path);
        // Keep only 5 most recent
        if self.recent_files.len() > 5 {
            log::debug!("Recent files list exceeded 5, truncating");
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
    ToggleSettings,
    ThemeChanged(AppTheme),
    ZoomIncrease,
    ZoomDecrease,
}

/// Update function that handles messages and modifies state
pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::OpenFileDialog => {
            Task::perform(open_file_dialog(), Message::FileSelected)
        }
        Message::FileSelected(path) => {
            if let Some(path) = path {
                log::info!("File selected: {:?}", path);
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
                log::info!("Recent file selected: {:?}", path);
                state.file_path = Some(path.clone());
                state.add_to_recent_files(path.clone());
                state.error = None;
                return Task::perform(parse_file_async(path), Message::MetadataParsed);
            } else {
                log::warn!("Recent file no longer exists: {:?}", path);
                state.error = Some("File no longer exists".to_string());
                // Remove from recent files if it doesn't exist
                state.remove_from_recent_files(&path);
            }
            Task::none()
        }
        Message::MetadataParsed(result) => {
            match result {
                Ok(metadata) => {
                    log::info!("Metadata parsed successfully. Title: {:?}, Artist: {:?}", 
                        metadata.title, metadata.artist);
                    state.metadata = Some(metadata);
                    state.error = None;
                }
                Err(e) => {
                    log::error!("Failed to parse metadata: {}", e);
                    state.error = Some(e.to_string());
                    state.metadata = None;
                }
            }
            Task::none()
        }
        Message::ToggleSettings => {
            state.settings_open = !state.settings_open;
            Task::none()
        }
        Message::ThemeChanged(theme) => {
            log::debug!("Theme changed to: {:?}", theme);
            state.theme = theme;
            state.save_settings();
            Task::none()
        }
        Message::ZoomIncrease => {
            let new_zoom = (state.zoom + 0.1).min(2.0);
            log::debug!("Zoom increased from {:.1} to {:.1}", state.zoom, new_zoom);
            state.zoom = new_zoom;
            state.save_settings();
            Task::none()
        }
        Message::ZoomDecrease => {
            let new_zoom = (state.zoom - 0.1).max(0.5);
            log::debug!("Zoom decreased from {:.1} to {:.1}", state.zoom, new_zoom);
            state.zoom = new_zoom;
            state.save_settings();
            Task::none()
        }
    }
}

/// View function that builds the UI
pub fn view(state: &State) -> Element<'_, Message> {
    // Apply zoom to sizes
    let base_size = 32.0 * state.zoom;
    let base_spacing = 10.0 * state.zoom;
    let base_padding = 20.0 * state.zoom;

    // Settings button in top right
    let settings_button = button("⚙️")
        .on_press(Message::ToggleSettings)
        .padding(8);

    // Header with title and settings button
    let header = row![
        text("ID3 Tag Manager")
            .size(base_size as u32)
            .width(Length::Fill)
            .align_x(Horizontal::Center),
        settings_button,
    ]
    .spacing(10)
    .align_y(Vertical::Center)
    .width(Length::Fill);

    let file_picker = button("Select Audio File")
        .on_press(Message::OpenFileDialog)
        .padding(10);

    // Display current file path (read-only) - apply zoom to text size
    let file_text_size = (14.0 * state.zoom) as u32;
    let file_path_display = if let Some(ref path) = state.file_path {
        container(
            text(path.to_string_lossy())
                .size(file_text_size)
                .style(|_theme| {
                    iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.6, 0.8, 1.0)),
                    }
                })
        )
        .padding(10.0 * state.zoom)
        .style(container::rounded_box)
        .width(Length::Fill)
    } else {
        container(
            text("No file selected")
                .size(file_text_size)
                .style(|_theme| {
                    iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                    }
                })
        )
        .padding(10.0 * state.zoom)
        .style(container::rounded_box)
        .width(Length::Fill)
    };

    let mut content = column![
        header,
        Space::new().height(20.0 * state.zoom),
        file_picker,
        Space::new().height(base_spacing),
        file_path_display,
    ]
    .spacing(base_spacing)
    .padding(base_padding)
    .width(Length::Fill);

    // Add recent files section
    if !state.recent_files.is_empty() {
        content = content.push(Space::new().height(20.0 * state.zoom));
        content = content.push(build_recent_files_view(&state.recent_files, &state.file_path, state.zoom));
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
        content = content.push(Space::new().height(20.0 * state.zoom));
        content = content.push(build_metadata_view(metadata, state.zoom, state.theme));
    } else {
        let no_metadata_text_size = (14.0 * state.zoom) as u32;
        content = content.push(
            container(
                text("No metadata loaded. Select an audio file to view its ID3 tags.")
                    .size(no_metadata_text_size)
                    .align_x(Horizontal::Center),
            )
            .padding(20.0 * state.zoom)
            .width(Length::Fill),
        );
    }

    let main_content = container(
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
    .padding(Padding::new(base_padding));

    // Add settings overlay if open
    if state.settings_open {
        // Create overlay with semi-transparent background and centered settings window
        // Use a container that covers the entire screen
        container(
            row![
                Space::new().width(Length::Fill),
                column![
                    Space::new().height(Length::Fill),
                    build_settings_overlay(state),
                    Space::new().height(Length::Fill),
                ]
                .align_x(Horizontal::Center)
                .width(Length::Shrink),
                Space::new().width(Length::Fill),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
                ..container::rounded_box(theme)
            }
        })
        .into()
    } else {
        main_content.into()
    }
}

/// Build the settings overlay window (fixed size, not affected by zoom)
fn build_settings_overlay<'a>(state: &'a State) -> Element<'a, Message> {
    // Fixed sizes for the overlay (not affected by zoom)
    const TITLE_SIZE: u32 = 20;
    const BUTTON_PADDING: f32 = 10.0;
    const SECTION_PADDING: f32 = 15.0;
    const SPACING: f32 = 10.0;
    const BUTTON_WIDTH: f32 = 50.0;
    
    // Capture theme for use in closures
    let theme = state.theme;

    // Theme selection
    let dark_button = if state.theme == AppTheme::Dark {
        button("Dark")
            .on_press(Message::ThemeChanged(AppTheme::Dark))
            .style(button_widget::primary)
    } else {
        button("Dark")
            .on_press(Message::ThemeChanged(AppTheme::Dark))
            .style(button_widget::secondary)
    };
    
    let light_button = if state.theme == AppTheme::Light {
        button("Light")
            .on_press(Message::ThemeChanged(AppTheme::Light))
            .style(button_widget::primary)
    } else {
        button("Light")
            .on_press(Message::ThemeChanged(AppTheme::Light))
            .style(button_widget::secondary)
    };
    
    let theme_buttons = row![dark_button, light_button].spacing(SPACING);

    // Zoom controls with + and - buttons (centered text)
    let zoom_controls = row![
        button(
            text("-")
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
        )
            .on_press(Message::ZoomDecrease)
            .width(Length::Fixed(BUTTON_WIDTH))
            .height(Length::Fixed(BUTTON_WIDTH))
            .padding(BUTTON_PADDING),
        text(format!("Zoom: {:.0}%", state.zoom * 100.0))
            .size(16) // Slightly larger for bold appearance
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .style(move |_theme| {
                iced::widget::text::Style {
                    // Theme-aware label color
                    color: Some(match theme {
                        AppTheme::Light => iced::Color::from_rgb(0.1, 0.1, 0.1), // Dark for light theme
                        AppTheme::Dark => iced::Color::from_rgb(0.9, 0.9, 0.9),  // Light for dark theme
                    }),
                }
            }),
        button(
            text("+")
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
        )
            .on_press(Message::ZoomIncrease)
            .width(Length::Fixed(BUTTON_WIDTH))
            .height(Length::Fixed(BUTTON_WIDTH))
            .padding(BUTTON_PADDING),
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center);

    // Close button
    let close_button = button(text("X")
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center))
        .on_press(Message::ToggleSettings)
        .padding(5.0)
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0));

    container(
        column![
            // Header row with title and close button
            row![
                text("Accessibility Settings")
                    .size(TITLE_SIZE)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
                close_button,
            ]
            .align_y(Vertical::Center)
            .width(Length::Fill),
            Space::new().height(SPACING),
            row![
                text("Theme:")
                    .size(16) // Slightly larger for bold appearance
                    .width(Length::Fixed(100.0))
                    .style(move |_theme| {
                        iced::widget::text::Style {
                            // Theme-aware label color
                            color: Some(match theme {
                                AppTheme::Light => iced::Color::from_rgb(0.1, 0.1, 0.1), // Dark for light theme
                                AppTheme::Dark => iced::Color::from_rgb(0.9, 0.9, 0.9),  // Light for dark theme
                            }),
                        }
                    }),
                theme_buttons,
            ]
            .spacing(SPACING)
            .align_y(Vertical::Center),
            Space::new().height(SPACING),
            zoom_controls,
        ]
        .spacing(SPACING)
        .width(Length::Fill),
    )
    .padding(SECTION_PADDING)
    .style(container::rounded_box)
    .width(Length::Fixed(350.0))
    .into()
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

/// Build the recent files view
fn build_recent_files_view<'a>(recent_files: &'a [PathBuf], current_file: &'a Option<PathBuf>, zoom: f32) -> Element<'a, Message> {
    let text_size = (16.0 * zoom) as u32;
    let title_size = (20.0 * zoom) as u32;
    let spacing = 10.0 * zoom;
    let padding = 15.0 * zoom;
    
    let mut files_column = Column::new()
        .spacing(5.0 * zoom)
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
                    .size(text_size)
                    .width(Length::Fill)
                    .align_x(Horizontal::Left)
            )
            .on_press(Message::SelectRecentFile(path_clone))
            .padding(8.0 * zoom)
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
                    .size(text_size)
                    .width(Length::Fill)
                    .align_x(Horizontal::Left)
            )
            .on_press(Message::SelectRecentFile(path_clone))
            .padding(8.0 * zoom)
            .width(Length::Fill)
        };
        
        files_column = files_column.push(file_button);
    }

    container(
        column![
            text(format!("Recent Files ({})", recent_files.len()))
                .size(title_size)
                .align_x(Horizontal::Center),
            Space::new().height(spacing),
            files_column,
        ]
        .spacing(spacing)
        .width(Length::Fill),
    )
    .padding(padding)
    .style(container::rounded_box)
    .width(Length::Fill)
    .into()
}

/// Build the metadata display view
fn build_metadata_view(metadata: &AudioMetadata, zoom: f32, theme: AppTheme) -> Element<'_, Message> {
    let title_size = (24.0 * zoom) as u32;
    let spacing = 10.0 * zoom;
    let padding = 15.0 * zoom;
    
    let mut metadata_rows = Column::new()
        .spacing(spacing)
        .width(Length::Fill);

    // Add rows for each metadata field
    metadata_rows = add_string_field(metadata_rows, "Title:", &metadata.title, zoom, theme);
    metadata_rows = add_numeric_field(metadata_rows, "Duration:", metadata.duration, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Artist:", &metadata.artist, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Album:", &metadata.album, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Album Artist:", &metadata.album_artist, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Composer:", &metadata.composer, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Genre:", &metadata.genre, zoom, theme);
    metadata_rows = add_numeric_field(metadata_rows, "Year:", metadata.year, zoom, theme);
    metadata_rows = add_numeric_field(metadata_rows, "Track:", metadata.track, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Comment:", &metadata.comment, zoom, theme);
    metadata_rows = add_numeric_field(metadata_rows, "Disc:", metadata.disc, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Publisher:", &metadata.publisher, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Encoder:", &metadata.encoder, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Language:", &metadata.language, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Copyright:", &metadata.copyright, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Original Artist:", &metadata.original_artist, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Original Album:", &metadata.original_album, zoom, theme);
    metadata_rows = add_numeric_field(metadata_rows, "Original Year:", metadata.original_year, zoom, theme);
    metadata_rows = add_numeric_field(metadata_rows, "BPM:", metadata.bpm, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "ISRC:", &metadata.isrc, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Conductor:", &metadata.conductor, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Remixer:", &metadata.remixer, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Producer:", &metadata.producer, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Grouping:", &metadata.grouping, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Subtitle:", &metadata.subtitle, zoom, theme);
    metadata_rows = add_string_field(metadata_rows, "Date:", &metadata.date, zoom, theme);

    // Handle lyrics with truncation
    if let Some(ref lyrics) = metadata.lyrics {
        if !lyrics.is_empty() {
            let display_lyrics = if lyrics.len() > 200 {
                format!("{}...", &lyrics[..200])
            } else {
                lyrics.clone()
            };
            metadata_rows = metadata_rows.push(create_row("Lyrics:", display_lyrics, zoom, theme));
        }
    }

    // Display custom fields
    for (key, value) in &metadata.custom_fields {
        if !value.is_empty() {
            let label = format!("{}:", key);
            metadata_rows = metadata_rows.push(create_row(label, value.clone(), zoom, theme));
        }
    }

    let mut metadata_content = column![
        text("Metadata")
            .size(title_size)
            .align_x(Horizontal::Center),
        Space::new().height(spacing * 1.5),
    ]
    .spacing(spacing)
    .width(Length::Fill);

    // Add cover art - show default if not available
    let cover_display: Element<'_, Message> = if let Some(ref cover_data) = metadata.cover_art {
        // Generate a unique filename based on track metadata with correct extension
        let extension = determine_image_extension(&metadata.cover_art_format);
        let filename_base = generate_cover_filename(metadata);
        let filename = format!("{}.{}", filename_base, extension);
        let temp_dir = std::env::temp_dir().join("id3_iced_manager");
        let temp_path = temp_dir.join(&filename);
        
        // Create directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&temp_dir) {
            log::error!("Failed to create temp directory {:?}: {}", temp_dir, e);
        }
        
        // Write cover art to file
        use std::fs;
        use std::io::Write;
        
        log::debug!("Saving cover art to: {:?} (format: {:?})", temp_path, metadata.cover_art_format);
        
        // Only write if file doesn't exist (reuse existing file)
        if temp_path.exists() {
            log::debug!("Cover file already exists, reusing: {:?}", temp_path);
            let handle = Handle::from_path(temp_path.clone());
            image(handle)
                .width(Length::Fixed(200.0))
                .height(Length::Fixed(200.0))
                .into()
        } else {
            match fs::File::create(&temp_path) {
                Ok(mut file) => {
                    if file.write_all(cover_data).is_ok() {
                        drop(file); // Close file before reading
                        
                        // Verify file exists and has content
                        if temp_path.exists() {
                            if let Ok(file_metadata) = fs::metadata(&temp_path) {
                                log::debug!("Cover file created successfully: {:?}, size: {} bytes", temp_path, file_metadata.len());
                            }
                            let handle = Handle::from_path(temp_path.clone());
                            image(handle)
                                .width(Length::Fixed(200.0))
                                .height(Length::Fixed(200.0))
                                .into()
                        } else {
                            log::warn!("Cover file does not exist after creation: {:?}", temp_path);
                            create_default_cover()
                        }
                    } else {
                        log::error!("Failed to write cover data to file: {:?}", temp_path);
                        create_default_cover()
                    }
                }
                Err(e) => {
                    log::error!("Failed to create cover file {:?}: {}", temp_path, e);
                    create_default_cover()
                }
            }
        }
    } else {
        create_default_cover()
    };
    
    metadata_content = metadata_content.push(
        container(cover_display)
            .align_x(Horizontal::Center)
            .padding(10.0 * zoom)
    );
    metadata_content = metadata_content.push(Space::new().height(spacing * 1.5));

    metadata_content = metadata_content.push(
        container(metadata_rows)
            .padding(padding)
            .style(container::rounded_box)
            .width(Length::Fill)
    );

    container(metadata_content)
        .padding(20.0 * zoom)
        .style(container::rounded_box)
        .width(Length::Fill)
        .into()
}

/// Create a metadata row element
fn create_row<'a>(label: impl Into<String>, value: String, zoom: f32, theme: AppTheme) -> Element<'a, Message> {
    let label_str = label.into();
    let text_size = (14.0 * zoom) as u32;
    // Make labels appear bold by using a slightly larger size (15px instead of 14px)
    let label_size = ((15.0 * zoom) as u32).max(1);
    
    // Theme-aware label color: dark for light theme, light for dark theme
    let label_color = match theme {
        AppTheme::Light => iced::Color::from_rgb(0.1, 0.1, 0.1), // Dark color for light theme
        AppTheme::Dark => iced::Color::from_rgb(0.9, 0.9, 0.9),  // Light color for dark theme
    };
    
    row![
        text(label_str.clone())
            .size(label_size)
            .width(Length::Fixed(150.0 * zoom))
            .style(move |_theme| {
                iced::widget::text::Style {
                    // Make labels more prominent (appear bold) with theme-aware color
                    color: Some(label_color),
                }
            }),
        text(value)
            .size(text_size)
            .width(Length::Fill),
    ]
    .spacing(10.0 * zoom)
    .align_y(Vertical::Center)
    .into()
}

/// Add a string field if it exists and is not empty
fn add_string_field<'a>(rows: Column<'a, Message>, label: &'a str, value: &Option<String>, zoom: f32, theme: AppTheme) -> Column<'a, Message> {
    if let Some(ref val) = value {
        if !val.is_empty() {
            rows.push(create_row(label, val.clone(), zoom, theme))
        } else {
            rows
        }
    } else {
        rows
    }
}

/// Add a numeric field if it exists
fn add_numeric_field<'a>(rows: Column<'a, Message>, label: &'a str, value: Option<u32>, zoom: f32, theme: AppTheme) -> Column<'a, Message> {
    if let Some(val) = value {
        rows.push(create_row(label, val.to_string(), zoom, theme))
    } else {
        rows
    }
}

/// Determine the file extension based on the image format
fn determine_image_extension(format: &Option<String>) -> &'static str {
    match format.as_deref() {
        Some("image/jpeg") | Some("image/jpg") => "jpg",
        Some("image/png") => "png",
        Some("image/gif") => "gif",
        Some("image/webp") => "webp",
        _ => "jpg", // Default to jpg if format is unknown
    }
}

/// Generate a unique filename for the cover image based on track metadata
fn generate_cover_filename(metadata: &AudioMetadata) -> String {
    // Create a hash from track metadata to ensure uniqueness
    let mut hasher = DefaultHasher::new();
    
    // Use title and artist if available, otherwise use a hash of all metadata
    let identifier = if let (Some(title), Some(artist)) = (&metadata.title, &metadata.artist) {
        format!("{}_{}", sanitize_filename(title), sanitize_filename(artist))
    } else if let Some(title) = &metadata.title {
        sanitize_filename(title)
    } else if let Some(artist) = &metadata.artist {
        sanitize_filename(artist)
    } else {
        // Fallback: hash the cover data or use a timestamp
        format!("cover_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
    };
    
    identifier.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Limit filename length and add hash for uniqueness
    let mut filename = identifier.chars().take(50).collect::<String>();
    filename.push_str(&format!("_{:x}.jpg", hash));
    
    filename
}

/// Sanitize a string to be used as a filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            ' ' => '_',
            _ => '_',
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Create a default cover image placeholder
fn create_default_cover<'a>() -> Element<'a, Message> {
    container(
        column![
            text("🎵")
                .size(80)
                .align_x(Horizontal::Center),
            text("No Cover")
                .size(16)
                .align_x(Horizontal::Center)
                .style(|_theme| {
                    iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                    }
                }),
        ]
        .spacing(10)
        .align_x(Horizontal::Center)
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .width(Length::Fixed(200.0))
    .height(Length::Fixed(200.0))
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|theme| {
        container::Style {
            background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
            border: iced::Border {
                width: 1.0,
                color: iced::Color::from_rgb(0.4, 0.4, 0.4),
                radius: 4.0.into(),
            },
            ..container::rounded_box(theme)
        }
    })
    .into()
}
