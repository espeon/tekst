use std::{
    env,
    fs,
    ops::{Deref},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use crate::structs::Meta;

use self::spotify::SpotifyClient;

use rspotify::{
    clients::{mutex::Mutex, BaseClient, OAuthClient},
    model::{AdditionalType, Country, Market},
    scopes, AuthCodeSpotify, Config, Credentials, OAuth,
};
pub mod spotify;

pub struct PlaybackInfo {
    pub position: Option<Duration>,
    pub playing: bool,
}

pub trait Client {
    fn init() -> Self;
    fn get_pos(&self) -> Option<PlaybackInfo>;
    fn get_metadata(&self) -> Option<Meta>;
}

fn cache_path(path: &str) -> PathBuf {
    let project_dir_path = env::current_dir().unwrap();
    let mut cache_path = project_dir_path;
    cache_path.push(path);
    if !cache_path.exists() {
        let mut path = cache_path.clone();
        path.pop();
        fs::create_dir_all(path).unwrap();
    }
    cache_path
}

impl Client for SpotifyClient {
    fn init() -> Self {
        let _ = dotenv::dotenv();

        let config = Config {
            token_cached: true,
            cache_path: cache_path(&"spotify.auth"),
            ..Default::default()
        };

        let creds = Credentials::new(
            &std::env::var("SPOTIFY_KEY").expect("SPOTIFY_KEY not defined in environment"),
            &std::env::var("SPOTIFY_SECRET").expect("SPOTIFY_SECRET not defined in environment"),
        );
        let oauth = OAuth::from_env(scopes!(
            "user-read-currently-playing",
            "user-read-playback-state"
        ))
        .unwrap();
        let mut spotify = AuthCodeSpotify::with_config(creds, oauth, config);

        if cache_path(&"spotify.auth").exists() {
            let token = spotify.read_token_cache(true).unwrap();
            spotify.token = Arc::new(Mutex::new(token));
            spotify.refresh_token().unwrap();
        } else {
            let url = spotify.get_authorize_url(true).unwrap();
            println!("{url}");

            let mut buffer = String::new();
            let stdin = std::io::stdin();
            stdin.read_line(&mut buffer).unwrap();
            spotify
                .request_token(&spotify.parse_response_code(&buffer).unwrap())
                .unwrap();

            let token;

            match &spotify.get_token().lock().unwrap().deref() {
                Some(e) => token = e.clone(),
                None => todo!(),
            };

            token.write_cache("spotify.auth").unwrap();
        }

        SpotifyClient {
            client: Arc::new(spotify.clone()),
        }
    }

    fn get_pos(&self) -> Option<PlaybackInfo> {
        let market = Market::Country(Country::UnitedStates);
        let additional_types = [AdditionalType::Episode];

        let pb = self
            .client
            .current_playback(Some(&market), Some(&additional_types))
            .unwrap();

        return match pb {
            Some(pos) => Some(PlaybackInfo {
                position: pos.progress,
                playing: pos.is_playing
            }),
            None => None,
        }
    }

    fn get_metadata(&self) -> Option<Meta> {
        let pb = match self
            .client
            .current_playing(None, Some([&AdditionalType::Episode])) {
                Ok(e) => e,
                Err(_) => return None,
            };

        match pb {
            Some(pb) => match pb.item.to_owned() {
                Some(rspotify::model::PlayableItem::Track(e)) => Some(Meta {
                    title: Some(e.name),
                    artist: Some((&e.album.artists[0].name).to_owned()),
                    spotify_uri: Some(e.id.unwrap().to_string())
                }),
                Some(rspotify::model::PlayableItem::Episode(_)) => None,
                None => None,
            },
            _ => None,
        }
    }
}
