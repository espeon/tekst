use std::time::Duration;

use crate::structs::LyricLine;

use super::LyricsSource;
use reqwest;

pub struct XmLyrSource {}

impl LyricsSource for XmLyrSource {
    fn get(metadata: crate::structs::Meta) -> crate::structs::Lyrics {
        let tekst_domain: &std::string::String = &std::env::var("JLF_DOMAIN").expect("JLF_DOMAIN not defined in environment");
        let lyrics = reqwest::blocking::get(format!("{}/{}", tekst_domain, metadata.spotify_uri.unwrap())).unwrap().json::<crate::structs::LRCLyrics>().unwrap();

        let mut lines: Vec<LyricLine> = vec![];

        for l in lyrics.lyrics_synced {
            lines.push(LyricLine {
                line: l.text,
                start: Duration::from_secs_f64(l.time),
                end: None,
            })
        }

        crate::structs::Lyrics {
            lines,
            metadata: crate::structs::LyricsMetadata {
                title: lyrics.meta.title,
                artist: lyrics.meta.artist,
            },
        }
    }
}
