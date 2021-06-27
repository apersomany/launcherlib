use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Client {
    pub url: String,
    pub size: u32
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub client: Client
}