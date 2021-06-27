use anyhow::{Result};
use scraper::{Html, Selector};

use super::*;

const BASE_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/index_";

pub async fn parse(url: &str) -> Result<Manifest> {
    let text = reqwest::get(url).await?.text().await?;
    let html = Html::parse_document(&text);
    let lsel = Selector::parse(".download-list > tbody:nth-child(3) > *").unwrap();
    let vsel = Selector::parse(".download-version").unwrap();
    let ssel = Selector::parse(".download-version > i").unwrap();
    let dsel = Selector::parse(".download-links > * > a:nth-child(2)").unwrap();
    let mut tags = Vec::new();
    let latest_tag = Tag {
        id: "latest".to_string(),
        name: "Latest".to_string(),
        filter: false
    };
    let recomm_tag = Tag {
        id: "recommended".to_string(),
        name: "Recommended".to_string(),
        filter: false
    };
    tags.push(latest_tag.clone());
    tags.push(recomm_tag.clone());
    let mut versions = Vec::new();
    let list = html.select(&lsel);
    for (i, elem) in list.enumerate() {
        let velm = elem.select(&vsel).next().unwrap();
        let id = velm
            .text()
            .collect::<Vec<&str>>()[0]
            .split_whitespace()
            .collect::<Vec<&str>>()[0];
        let star = velm.select(&ssel).next().is_some();
        let dsel = elem.select(&dsel);
        for elem in dsel {
            let href = elem.value().attr("href");
            if href.is_some() {
                let href = href.unwrap();
                let mut tags = Vec::new();
                if i == 0 { tags.push(latest_tag.clone()) }
                if star { tags.push(recomm_tag.clone()) }
                if href.ends_with("installer.jar") {
                    versions.push(Version {
                        id: id.to_string(),
                        url: href.to_string(),
                        tags
                    })
                }
            }
        }
    }
    Ok(Manifest { tags, versions })
}

pub async fn get(id: &str) -> Result<Manifest> {
    parse(&format!("{}{}.html", BASE_URL, id)).await
}