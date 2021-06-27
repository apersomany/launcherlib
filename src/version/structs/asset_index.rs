use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub url: String
}