# ID3 Iced Manager

A cross-platform desktop application built with Rust and Iced for viewing and managing ID3 metadata tags in audio files.

## Features

- 📁 **File Selection**: Easy file picker to select audio files
- 🏷️ **ID3 Tag Parsing**: Extract and display comprehensive metadata including:
  - Title
  - Artist
  - Album
  - Album Artist
  - Composer
  - Genre
  - Year
  - Track Number
  - Comments
  - Duration
- 📋 **Recent Files**: Quick access to your 5 most recently opened files
- 💾 **Persistent State**: Recent files are saved and restored between sessions
- 🎨 **Modern UI**: Clean, dark-themed interface built with Iced

## Supported Audio Formats

- MP3
- FLAC
- M4A
- AAC
- OGG
- WAV

## Installation

### Prerequisites

- Rust (latest stable version)
- Cargo

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd id3-iced-manager

# Build the project
cargo build --release

# Run the application
cargo run --release
```

The executable will be located in `target/release/id3-iced-manager`.

## Usage

1. **Select a File**: Click the "Select Audio File" button to open a file dialog
2. **View Metadata**: Once a file is selected, its ID3 tags are automatically parsed and displayed
3. **Quick Access**: Use the "Recent Files" section to quickly reload previously opened files
4. **Current File**: The currently selected file is highlighted in blue in the recent files list

## Project Structure

```
id3-iced-manager/
├── src/
│   ├── main.rs          # Application entry point
│   ├── app.rs           # Application logic and UI
│   ├── config.rs        # Configuration and persistence
│   └── id3_parser/      # ID3 tag parsing module
│       ├── mod.rs       # Public API
│       └── imp.rs       # Implementation
├── Cargo.toml          # Project dependencies
└── README.md           # This file
```

## Dependencies

- **iced** (0.14.0): Cross-platform GUI framework
- **id3** (1.13): ID3 tag parsing library
- **rfd** (0.14): Native file dialogs
- **serde_json** (1.0): JSON serialization for persistence
- **dirs** (5.0): Cross-platform directory access

## Configuration

The application stores its configuration in the standard config directory for your operating system:

- **Linux**: `~/.config/id3-iced-manager/`
- **macOS**: `~/Library/Application Support/id3-iced-manager/`
- **Windows**: `%APPDATA%\id3-iced-manager\`

Recent files are saved in `recent_files.json`.

## Development

### Running in Development Mode

```bash
cargo run
```

### Building for Release

```bash
cargo build --release
```

## TODO

- [ ] **Add ID3 Tags**: Implement functionality to edit and save ID3 tags to audio files
- [ ] **Show Cover if Available**: Display album artwork/cover image when present in ID3 tags
- [ ] **Accessibility Options**: 
  - [ ] Zoom controls for UI scaling
  - [ ] Theme selection (light/dark/custom themes)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Add your license here]

## Acknowledgments

- Built with [Iced](https://github.com/iced-rs/iced) - A cross-platform GUI library for Rust
- ID3 parsing powered by [id3](https://github.com/polyfloyd/id3-rs)

