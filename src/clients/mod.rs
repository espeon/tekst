use std::{
    env,
    fs,
    ops::{Deref},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use crate::structs::Meta;

use self::{mpris::MPRISClient, spotify::SpotifyClient};

use rspotify::{
    clients::{mutex::Mutex, BaseClient, OAuthClient},
    model::{AdditionalType, Country, Market},
    scopes, AuthCodeSpotify, Config, Credentials, OAuth,
};

pub mod mpris;
pub mod spotify;

pub trait Client {
    fn init() -> Self;
    fn get_pos(&self) -> Option<Duration>;
    fn get_metadata(&self) -> Option<Meta>;
}

impl Client for MPRISClient {
    fn init() -> Self {
        MPRISClient {}
    }

    fn get_pos(&self) -> Option<Duration> {
        Some(Duration::from_millis(696969))
    }

    fn get_metadata(&self) -> Option<Meta> {
        Some(Meta {
            title: Some("bob".to_string()),
            artist: Some("jon".to_string()),
            spotify_uri: None
        })
    }
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

    fn get_pos(&self) -> Option<Duration> {
        let market = Market::Country(Country::UnitedStates);
        let additional_types = [AdditionalType::Episode];

        let pb = self
            .client
            .current_playback(Some(&market), Some(&additional_types))
            .unwrap();

        match pb {
            Some(e) => e.progress,
            None => None,
        }
    }

    fn get_metadata(&self) -> Option<Meta> {
        let pb = self
            .client
            .current_playing(None, Some([&AdditionalType::Episode]))
            .unwrap();

        match pb {
            Some(pb) => match pb.item.to_owned().unwrap() {
                rspotify::model::PlayableItem::Track(e) => Some(Meta {
                    title: Some(e.name),
                    artist: Some((&e.album.artists[0].name).to_owned()),
                    spotify_uri: Some(e.id.unwrap().to_string())
                }),
                rspotify::model::PlayableItem::Episode(_) => todo!(),
            },
            None => None,
        }
    }
}
