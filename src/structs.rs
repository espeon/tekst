use std::time::Duration;

use serde::{Serialize, Deserialize};

pub struct Lyrics {
    pub lines: Vec<LyricLine>,
    pub metadata: LyricsMetadata
}

pub struct LyricLine {
    pub line: String,
    pub start: Duration,
    pub end: Option<Duration>
}

pub struct LyricsMetadata {
    pub title: Option<String>,
    pub artist: Option<String>
}

// API response (lyr api)
#[derive(Serialize, Deserialize)]
pub struct LRCLyrics {
    pub lyrics_synced: Vec<LyricsSynced>,
    pub meta: Meta,
}

#[derive(Serialize, Deserialize)]
pub struct LyricsSynced {
    pub time: f64,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub spotify_uri: Option<String>
}