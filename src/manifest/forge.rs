use anyhow::Result;
use serde_derive::Deserialize;

use crate::manifest::{Version, Manifest};

const FORGE_MAVEN_URL: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge";

#[derive(Debug, Deserialize)]
struct Root {
    versioning: Versioning
}

#[derive(Debug, Deserialize)]
struct Versioning {
    versions: Versions
}

#[derive(Debug, Deserialize)]
struct Versions {
    #[serde(rename = "$value")]
    list: Vec<String>
}

pub async fn parse(url: &str) -> Result<Manifest> {
    let text = reqwest::get(url).await?.text().await?;
    let root: Root = serde_xml_rs::from_str(&text)?;
    let versions = root.versioning.versions.list
        .iter()
        .map(|v| {
            Version {
                id: v.split("-").last().unwrap().to_string(),
                tags: Vec::new(),
                url: format!(
                    "{}/{}/forge-{}-installer.jar",
                    FORGE_MAVEN_URL, v, v
                ),
            }
        }).collect();
    Ok(Manifest { tags: Vec::new(), versions })
}

pub async fn get() -> Result<Manifest> {
    parse(&format!("{}/maven-metadata.xml", FORGE_MAVEN_URL)).await
}