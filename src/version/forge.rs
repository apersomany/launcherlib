use std::io::{Cursor, Read};

use anyhow::{Result, anyhow};
use serde_derive::Deserialize;
use zip::ZipArchive;

use crate::{manifest, version::{
    self, File, Version,
    structs:: {
        library::Library,
        arguments::{Argument, Arguments, Value}
    }
}};

const FORGE_MAVEN_URL: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Root {
    id: String,
    main_class: String,
    inherits_from: String,
    #[serde(default)]
    arguments: Arguments,
    #[serde(default)]
    minecraft_arguments: String,
    libraries: Vec<Library>
}

pub async fn parse(url: &str) -> Result<Version> {
    let inst = reqwest::get(url).await?.bytes().await?;
    let mut zip = ZipArchive::new(Cursor::new(inst))?;
    let mut file = Vec::new();
    zip.by_name("version.json")?.read_to_end(&mut file)?;
    let root: Root = serde_json::from_slice(&file)?;
    let parent = version::vanilla::get(&root.inherits_from).await?;
    let mut game_args = Vec::new();
    let mut jvm_args = Vec::new();
    if root.minecraft_arguments.len() > 0 {
        let split = root.minecraft_arguments.split(" ");
        split.for_each(|arg| { game_args.push(String::from(arg)) });
        jvm_args = parent.jvm_args;
    } else {
        root.arguments.game.into_iter().for_each(|arg| {
            match arg {
                Argument::Raw(value) => {
                    match value {
                        Value::Single(str) => { game_args.push(str) }
                        Value::Multiple(vec) => {
                            vec.into_iter().for_each(|str| { game_args.push(str) })
                        }
                    }
                }
                Argument::WithRules { rules, value } => {
                    if rules.iter().find(|r| !r.calc()).is_none() {
                        match value {
                            Value::Single(str) => { game_args.push(str) }
                            Value::Multiple(vec) => {
                                vec.into_iter().for_each(|str| { game_args.push(str) })
                            }
                        }
                    }
                }
            }

        });
        root.arguments.jvm.into_iter().for_each(|arg| {
            match arg {
                Argument::Raw(value) => {
                    match value {
                        Value::Single(str) => { jvm_args.push(str) }
                        Value::Multiple(vec) => {
                            vec.into_iter().for_each(|str| { jvm_args.push(str) })
                        }
                    }
                }
                Argument::WithRules { rules, value } => {
                    if rules.iter().find(|r| !r.calc()).is_none() {
                        match value {
                            Value::Single(str) => { jvm_args.push(str) }
                            Value::Multiple(vec) => {
                                vec.into_iter().for_each(|str| { jvm_args.push(str) })
                            }
                        }
                    }
                }
            }
        });
    }
    let mut libraries = parent.libraries;
    let split = root.id.split("-");
    let forge_id = format!("{}-{}", parent.id, split.last().unwrap());
    root.libraries.into_iter().for_each(|l| {
        if l.downloads.artifact.url.len() > 0 {
            libraries.push(File {
                path: l.downloads.artifact.path,
                url: l.downloads.artifact.url,
                size: l.downloads.artifact.size
            });
        } else {
            libraries.push(File {
                path:  l.downloads.artifact.path,
                url: format!(
                    "{}/{}/forge-{}-universal.jar",
                    FORGE_MAVEN_URL, forge_id, forge_id
                ),
                size: l.downloads.artifact.size
            })
        }
    });
    Ok(
        Version {
            id: root.id,
            natives: parent.natives,
            assets: parent.assets,
            main_class: root.main_class,
            jvm_args, game_args, libraries
        }
    )
}

pub async fn get(id: &str) -> Result<Version> {
    let manifest = manifest::forge::get().await?;
    match manifest.get(id) {
        Some(v) => { Ok(parse(&v.url).await?) }
        None => { Err(anyhow!("")) }
    }
}