pub mod imp;

use std::path::Path;

/// Represents the metadata extracted from an ID3 tag
#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub track: Option<u32>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub comment: Option<String>,
    pub duration: Option<u32>, // in seconds
}

/// Errors that can occur during ID3 parsing
#[clippy::allow(unused_self)]
#[derive(Debug, Clone)]
pub enum ParseError {
    #[allow(unused)]
    FileNotFound,
    #[allow(unused)]
    InvalidFormat,
    #[allow(unused)]
    NoId3Tag,
    #[allow(unused)]
    IoError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::FileNotFound => write!(f, "File not found"),
            ParseError::InvalidFormat => write!(f, "Invalid audio format"),
            ParseError::NoId3Tag => write!(f, "No ID3 tag found in file"),
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse ID3 tags from an audio file
pub fn parse_id3<P: AsRef<Path>>(path: P) -> Result<AudioMetadata, ParseError> {
    imp::parse_id3_impl(path)
}