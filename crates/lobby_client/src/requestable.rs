use std::borrow::Cow;

use reqwest::Request;
use serde::de::DeserializeOwned;
use url::Url;

pub(super) trait Requestable
where
    Self: Sync + Send + 'static,
{
    type Response: DeserializeOwned + Sync + Send + 'static;

    fn path(&self) -> Cow<str>;

    fn create(&self, url: Url) -> Request;
}
