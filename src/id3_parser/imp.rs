use id3::TagLike;

use super::{AudioMetadata, ParseError};
use std::path::Path;

/// Internal implementation of ID3 parsing
pub fn parse_id3_impl<P: AsRef<Path>>(path: P) -> Result<AudioMetadata, ParseError> {
    let path_ref = path.as_ref();
    
    log::debug!("Parsing ID3 tags from: {:?}", path_ref);
    
    // Check if file exists
    if !path_ref.exists() {
        log::error!("File not found: {:?}", path_ref);
        return Err(ParseError::FileNotFound);
    }

    // Try to read ID3 tags
    let tag = match id3::Tag::read_from_path(path_ref) {
        Ok(tag) => {
            log::debug!("ID3 tag read successfully");
            tag
        }
        Err(id3::Error { kind: id3::ErrorKind::NoTag, .. }) => {
            log::warn!("No ID3 tag found in file: {:?}", path_ref);
            return Err(ParseError::NoId3Tag);
        }
        Err(e) => {
            log::error!("Error reading ID3 tag from {:?}: {}", path_ref, e);
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

    // Extract disc number
    if let Some(disc) = tag.disc() {
        metadata.disc = Some(disc as u32);
    }

    // Extract publisher/record label (TPUB frame)
    if let Some(publisher) = tag.get("TPUB").and_then(|frame| frame.content().text()) {
        metadata.publisher = Some(publisher.to_string());
    }

    // Extract encoder (TENC frame)
    if let Some(encoder) = tag.get("TENC").and_then(|frame| frame.content().text()) {
        metadata.encoder = Some(encoder.to_string());
    }

    // Extract language (TLAN frame)
    if let Some(language) = tag.get("TLAN").and_then(|frame| frame.content().text()) {
        metadata.language = Some(language.to_string());
    }

    // Extract copyright (TCOP frame)
    if let Some(copyright) = tag.get("TCOP").and_then(|frame| frame.content().text()) {
        metadata.copyright = Some(copyright.to_string());
    }

    // Extract original artist (TOPE frame)
    if let Some(original_artist) = tag.get("TOPE").and_then(|frame| frame.content().text()) {
        metadata.original_artist = Some(original_artist.to_string());
    }

    // Extract original album (TOAL frame)
    if let Some(original_album) = tag.get("TOAL").and_then(|frame| frame.content().text()) {
        metadata.original_album = Some(original_album.to_string());
    }

    // Extract original year (TORY frame)
    if let Some(original_year) = tag.get("TORY").and_then(|frame| frame.content().text()) {
        if let Ok(year) = original_year.parse::<u32>() {
            metadata.original_year = Some(year);
        }
    }

    // Extract BPM (TBPM frame)
    if let Some(bpm) = tag.get("TBPM").and_then(|frame| frame.content().text()) {
        if let Ok(bpm_val) = bpm.parse::<u32>() {
            metadata.bpm = Some(bpm_val);
        }
    }

    // Extract ISRC (TSRC frame)
    if let Some(isrc) = tag.get("TSRC").and_then(|frame| frame.content().text()) {
        metadata.isrc = Some(isrc.to_string());
    }

    // Extract lyrics (USLT frame - Unsynchronized lyrics/text transcription)
    if let Some(lyrics_frame) = tag.get("USLT") {
        if let Some(lyrics_text) = lyrics_frame.content().text() {
            metadata.lyrics = Some(lyrics_text.to_string());
        }
    }

    // Extract conductor (TPE3 frame)
    if let Some(conductor) = tag.get("TPE3").and_then(|frame| frame.content().text()) {
        metadata.conductor = Some(conductor.to_string());
    }

    // Extract remixer (TPE4 frame)
    if let Some(remixer) = tag.get("TPE4").and_then(|frame| frame.content().text()) {
        metadata.remixer = Some(remixer.to_string());
    }

    // Extract producer (TPRO frame - not standard, but sometimes used)
    if let Some(producer) = tag.get("TPRO").and_then(|frame| frame.content().text()) {
        metadata.producer = Some(producer.to_string());
    }

    // Extract performer (TPE2 is album artist, but TPE1 is main artist, TPE4 is remixer)
    // TPE2 is already extracted as album_artist, so we'll use TPE1 for main performer
    // (already extracted as artist)

    // Extract grouping (TIT1 frame - Content group description)
    if let Some(grouping) = tag.get("TIT1").and_then(|frame| frame.content().text()) {
        metadata.grouping = Some(grouping.to_string());
    }

    // Extract subtitle (TIT3 frame - Subtitle/Description refinement)
    if let Some(subtitle) = tag.get("TIT3").and_then(|frame| frame.content().text()) {
        metadata.subtitle = Some(subtitle.to_string());
    }

    // Extract date (TDAT frame - Date)
    if let Some(date) = tag.get("TDAT").and_then(|frame| frame.content().text()) {
        metadata.date = Some(date.to_string());
    }

    // Extract encoded by (TENC frame - already extracted as encoder)
    // This is the same as encoder

    // Extract cover art (APIC frame)
    if let Some(picture) = tag.pictures().next() {
        log::debug!("Found cover art (format: {})", picture.mime_type);
        metadata.cover_art = Some(picture.data.clone());
        // Store the mime type for proper file extension
        metadata.cover_art_format = Some(picture.mime_type.clone());
    } else {
        log::debug!("No cover art found in ID3 tag");
    }

    // Extract all other text frames as custom fields
    for frame in tag.frames() {
        let frame_id = frame.id();
        // Skip frames we've already extracted
        if !matches!(frame_id, "TIT2" | "TPE1" | "TALB" | "TYER" | "TDRC" | "TCON" | "TRCK" | "TPE2" | "TCOM" | "COMM" | "TPOS" | "TPUB" | "TENC" | "TLAN" | "TCOP" | "TOPE" | "TOAL" | "TORY" | "TBPM" | "TSRC" | "USLT" | "TPE3" | "TPE4" | "TPRO" | "TIT1" | "TIT3" | "TDAT" | "APIC") {
            if let Some(text) = frame.content().text() {
                metadata.custom_fields.push((frame_id.to_string(), text.to_string()));
            }
        }
    }

    log::debug!("Successfully extracted metadata: title={:?}, artist={:?}, album={:?}, {} custom fields", 
        metadata.title, metadata.artist, metadata.album, metadata.custom_fields.len());
    
    Ok(metadata)
}