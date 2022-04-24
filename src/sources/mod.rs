use crate::structs::{Meta, Lyrics};

pub mod xmlyr;

pub trait LyricsSource {
    fn get(metadata: Meta) -> Option<Lyrics>;
}