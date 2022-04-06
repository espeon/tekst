use std::{fs, time::Duration};

use crate::structs::{LRCLyrics, LyricLine, Lyrics, LyricsMetadata};

pub fn parse() -> Lyrics {
    let lyr_str = fs::read_to_string("harmony_hall.json").unwrap();

    let lyr: LRCLyrics = serde_json::from_str(&lyr_str).unwrap();

    let mut lines: Vec<LyricLine> = vec![];

    for l in lyr.lyrics_synced {
        lines.push( LyricLine {
            line: l.text,
            start: Duration::from_secs_f64(l.time),
            end: None,
        })
    }

    return Lyrics {
        lines: lines,
        metadata: LyricsMetadata {
            title: lyr.meta.title,
            artist: lyr.meta.artist
        }
    };
}
