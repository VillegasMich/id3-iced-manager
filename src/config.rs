use std::fs;
use std::path::PathBuf;

/// Get the path to the config file for storing recent files
fn get_config_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let app_config_dir = config_dir.join("id3-iced-manager");
    // Create directory if it doesn't exist
    let _ = fs::create_dir_all(&app_config_dir);
    Some(app_config_dir.join("recent_files.json"))
}

/// Load recent files from disk
pub fn load_recent_files() -> Vec<PathBuf> {
    let config_path = match get_config_path() {
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

/// Save recent files to disk
pub fn save_recent_files(recent_files: &[PathBuf]) {
    let config_path = match get_config_path() {
        Some(path) => path,
        None => return, // Can't determine config path
    };

    // Convert PathBuf to String for serialization
    let paths: Vec<String> = recent_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // Serialize to JSON
    let json = match serde_json::to_string_pretty(&paths) {
        Ok(json) => json,
        Err(_) => return, // Serialization failed
    };

    // Write to file (ignore errors)
    let _ = fs::write(&config_path, json);
}
