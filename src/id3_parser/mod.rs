pub mod imp;

use std::path::Path;

/// Represents the metadata extracted from an ID3 tag
#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    // Basic information
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub track: Option<u32>,
    pub disc: Option<u32>, // Disc number
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub comment: Option<String>,
    pub duration: Option<u32>, // in seconds
    
    // Additional metadata
    pub publisher: Option<String>, // Record label/Publisher
    pub encoder: Option<String>, // Software/hardware used to encode
    pub language: Option<String>,
    pub copyright: Option<String>,
    pub original_artist: Option<String>,
    pub original_album: Option<String>,
    pub original_year: Option<u32>,
    pub bpm: Option<u32>, // Beats per minute
    pub isrc: Option<String>, // International Standard Recording Code
    pub lyrics: Option<String>,
    pub conductor: Option<String>,
    pub remixer: Option<String>,
    pub producer: Option<String>,
    pub grouping: Option<String>, // Content group description
    pub subtitle: Option<String>, // Subtitle/Description refinement
    pub date: Option<String>, // Recording date
    
    // Cover art
    pub cover_art: Option<Vec<u8>>, // Album cover image data
    pub cover_art_format: Option<String>, // Cover art format (e.g., "image/jpeg", "image/png")
    
    // Custom/Extended fields (stored as key-value pairs)
    pub custom_fields: Vec<(String, String)>,
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