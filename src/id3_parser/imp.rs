use id3::TagLike;

use super::{AudioMetadata, ParseError};
use std::path::Path;

/// Internal implementation of ID3 parsing
pub fn parse_id3_impl<P: AsRef<Path>>(path: P) -> Result<AudioMetadata, ParseError> {
    let path_ref = path.as_ref();
    
    // Check if file exists
    if !path_ref.exists() {
        return Err(ParseError::FileNotFound);
    }

    // Try to read ID3 tags
    let tag = match id3::Tag::read_from_path(path_ref) {
        Ok(tag) => tag,
        Err(id3::Error { kind: id3::ErrorKind::NoTag, .. }) => {
            return Err(ParseError::NoId3Tag);
        }
        Err(e) => {
            return Err(ParseError::IoError(e.to_string()));
        }
    };

    let mut metadata = AudioMetadata::default();

    // Extract title
    if let Some(title) = tag.title() {
        metadata.title = Some(title.to_string());
    }

    // Extract duration
    if let Some(duration) = tag.duration() {
        metadata.duration = Some(duration as u32);
    }

    // Extract artist
    if let Some(artist) = tag.artist() {
        metadata.artist = Some(artist.to_string());
    }

    // Extract album
    if let Some(album) = tag.album() {
        metadata.album = Some(album.to_string());
    }

    // Extract year
    if let Some(year) = tag.year() {
        metadata.year = Some(year as u32);
    }

    // Extract genre
    if let Some(genre) = tag.genre() {
        metadata.genre = Some(genre.to_string());
    }

    // Extract track number
    if let Some(track) = tag.track() {
        metadata.track = Some(track as u32);
    }

    // Extract album artist
    if let Some(album_artist) = tag.album_artist() {
        metadata.album_artist = Some(album_artist.to_string());
    }

    // Extract composer
    if let Some(composer) = tag.get("TCOM").and_then(|frame| frame.content().text()) {
        metadata.composer = Some(composer.to_string());
    }

    // Extract comment (first comment frame)
    if let Some(comment) = tag.comments().next() {
        metadata.comment = Some(comment.text.to_string());
    }

    Ok(metadata)
}