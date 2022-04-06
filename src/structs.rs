use std::time::Duration;

use serde::{Serialize, Deserialize};

pub struct Lyrics {
    pub lines: Vec<LyricLine>,
}

pub struct LyricLine {
    pub line: String,
    pub start: Duration,
    pub end: Option<Duration>
}

#[derive(Serialize, Deserialize)]
pub struct LRCLyrics {
    pub lyrics_synced: Vec<LyricsSynced>,
}

#[derive(Serialize, Deserialize)]
pub struct LyricsSynced {
    pub time: f64,
    pub text: String,
}