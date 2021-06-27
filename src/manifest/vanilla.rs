use std::collections::HashSet;
use anyhow::Result;
use serde_derive::Deserialize;

use super::*;

const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

#[derive(Debug, Deserialize)]
struct Root {
    latest: Latest,
    versions: Vec<Version>
}

#[derive(Debug, Deserialize)]
struct Latest {
    release: String,
    snapshot: String
}

#[derive(Debug, Deserialize)]
struct Version {
    id: String,
    url: String,
    #[serde(rename = "type")]
    _type: String
}

pub async fn parse(url: &str) -> Result<Manifest> {
    let root: Root = reqwest::get(url).await?.json().await?;
    let mut tags = HashSet::new();
    let latest_tag = Tag {
        id: "latest".to_string(),
        name: "Latest".to_string(),
        filter: false
    };
    tags.insert(latest_tag.clone());
    let versions = root.versions.iter().map(|v| {
        let tag = Tag {
            id: v._type.clone(),
            name: match v._type.as_str() {
                "release" => "Release",
                "snapshot" => "Snapshot",
                "old_alpha" => "Alpha",
                "old_beta" => "Beta",
                _ => &v._type
            }.to_string(),
            filter: true
        };
        tags.insert(tag.clone());
        let mut tags = vec![tag];
        #[allow(unused_parens)]
        if (
            v.id == root.latest.release ||
            v.id == root.latest.snapshot
        ) { tags.push(latest_tag.clone()) };
        super::Version {
            id: v.id.clone(),
            url: v.url.clone(),
            tags: tags
        }
    }).collect();
    let tags = tags.into_iter().collect();
    Ok(Manifest { tags, versions })
}

pub async fn get() -> Result<Manifest> {
    parse(MANIFEST_URL).await
}