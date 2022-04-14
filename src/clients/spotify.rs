use std::sync::Arc;

use rspotify::AuthCodeSpotify;

#[derive(Clone)]
pub struct SpotifyClient {
    pub client: Arc<AuthCodeSpotify>,
}