use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Theme options for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppTheme {
    Dark,
    Light,
}

impl Default for AppTheme {
    fn default() -> Self {
        AppTheme::Dark
    }
}

impl AppTheme {
    /// Convert to iced::Theme
    pub fn to_iced_theme(self) -> iced::Theme {
        match self {
            AppTheme::Dark => iced::Theme::Dark,
            AppTheme::Light => iced::Theme::Light,
        }
    }
}

/// Application configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub theme: AppTheme,
    pub zoom: f32,     // Zoom level (1.0 = 100%, 1.5 = 150%, etc.)
    #[serde(default)]
    pub recent_files: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: AppTheme::default(),
            zoom: 1.0,
            recent_files: Vec::new(),
        }
    }
}

/// Get the path to the config directory
fn get_config_dir() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let app_config_dir = config_dir.join("id3-iced-manager");
    // Create directory if it doesn't exist
    let _ = fs::create_dir_all(&app_config_dir);
    Some(app_config_dir)
}

/// Get the path to the main config file
fn get_config_path() -> Option<PathBuf> {
    Some(get_config_dir()?.join("config.json"))
}

/// Get the path to the recent files config file (for backward compatibility)
fn get_recent_files_path() -> Option<PathBuf> {
    Some(get_config_dir()?.join("recent_files.json"))
}

/// Load application configuration from disk
pub fn load_config() -> AppConfig {
    let config_path = match get_config_path() {
        Some(path) => path,
        None => {
            log::warn!("Could not determine config path, using default config");
            return AppConfig::default();
        }
    };

    log::debug!("Loading config from: {:?}", config_path);

    // Try to load from new config file
    if let Ok(content) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
            log::info!("Config loaded successfully");
            // Migrate recent files from old format if needed
            if config.recent_files.is_empty() {
                log::debug!("Migrating recent files from legacy format");
                config.recent_files = load_recent_files_legacy()
                    .into_iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
            }
            // Filter out non-existent files
            let initial_count = config.recent_files.len();
            config.recent_files.retain(|path| PathBuf::from(path).exists());
            if config.recent_files.len() < initial_count {
                log::debug!("Filtered out {} non-existent recent files", initial_count - config.recent_files.len());
            }
            return config;
        } else {
            log::warn!("Failed to parse config file, using default config");
        }
    } else {
        log::debug!("Config file does not exist, using default config");
    }

    // Try to migrate from old recent_files.json
    let recent_files = load_recent_files_legacy()
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    AppConfig {
        recent_files,
        ..AppConfig::default()
    }
}

/// Save application configuration to disk
pub fn save_config(config: &AppConfig) {
    let config_path = match get_config_path() {
        Some(path) => path,
        None => {
            log::error!("Could not determine config path, cannot save config");
            return;
        }
    };

    log::debug!("Saving config to: {:?}", config_path);

    // Serialize to JSON
    let json = match serde_json::to_string_pretty(config) {
        Ok(json) => json,
        Err(e) => {
            log::error!("Failed to serialize config: {}", e);
            return;
        }
    };

    // Write to file
    match fs::write(&config_path, json) {
        Ok(_) => log::debug!("Config saved successfully"),
        Err(e) => log::error!("Failed to write config file: {}", e),
    }
}

/// Load recent files from disk (legacy format for migration)
fn load_recent_files_legacy() -> Vec<PathBuf> {
    let config_path = match get_recent_files_path() {
        Some(path) => path,
        None => return Vec::new(),
    };

    // Read the file
    let content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(_) => return Vec::new(), // File doesn't exist or can't be read
    };

    // Deserialize JSON
    let paths: Vec<String> = match serde_json::from_str(&content) {
        Ok(paths) => paths,
        Err(_) => return Vec::new(), // Invalid JSON
    };

    // Convert to PathBuf and filter out files that don't exist
    paths
        .into_iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect()
}

/// Load recent files from disk (using new config format)
pub fn load_recent_files() -> Vec<PathBuf> {
    let config = load_config();
    config
        .recent_files
        .into_iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect()
}

/// Save recent files to disk (using new config format)
pub fn save_recent_files(recent_files: &[PathBuf]) {
    let mut config = load_config();
    
    // Convert PathBuf to String for serialization
    config.recent_files = recent_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    save_config(&config);
}
