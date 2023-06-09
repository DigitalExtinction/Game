use std::borrow::Cow;

use reqwest::Request;
use serde::de::DeserializeOwned;
use url::Url;

pub trait LobbyRequest: Sync + Send + 'static {
    type Response: DeserializeOwned + Sync + Send + 'static;
}

pub(super) trait LobbyRequestCreator: LobbyRequest {
    fn path(&self) -> Cow<str>;

    fn create(&self, url: Url) -> Request;
}
