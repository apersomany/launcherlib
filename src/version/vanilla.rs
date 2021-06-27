use std::vec;
use anyhow::{Result, anyhow};
use serde_derive::Deserialize;

use crate::{manifest};
use super::{
    Assets, File, Version,
    structs::{
        arguments::{Argument, Arguments, Value},
        asset_index::AssetIndex, downloads::Downloads, library::Library
    }
};

const OS_NAME: &str =
    if cfg!(target_os = "macos") { "osx" }
    else { std::env::consts::OS };

    
const OS_ARCH: &str =
    if cfg!(target_arch = "x86") { "32" }
    else if cfg!(target_arch = "x86_64"){ "64" }
    else { "" };

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Root {
    id: String,
    downloads: Downloads,
    asset_index: AssetIndex,
    #[serde(default)]
    arguments: Arguments,
    #[serde(default)]
    minecraft_arguments: String,
    libraries: Vec<Library>,
    main_class: String,
}

pub async fn parse(url: &str) -> Result<Version> {
    let root: Root = reqwest::get(url).await?.json().await?;
    let mut assets: Assets = reqwest::get(root.asset_index.url).await?.json().await?;
    assets.id = root.asset_index.id;
    let mut game_args = Vec::new();
    let mut jvm_args = Vec::new();
    if root.minecraft_arguments.len() > 0 {
        let split = root.minecraft_arguments.split(" ");
        split.for_each(|arg| { game_args.push(String::from(arg)) });
        jvm_args = vec![
            "-Djava.library.path=${natives_directory}",
            "-Dminecraft.launcher.brand=${launcher_name}",
            "-Dminecraft.launcher.version=${launcher_version}",
            "-cp",
            "${classpath}"
        ].iter().map(|a| a.to_string()).collect();
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
    let mut libraries = vec![
        File {
            path: format!("client/{}.jar", root.id),
            url: root.downloads.client.url,
            size: root.downloads.client.size,
        }
    ];
    let mut natives = Vec::new();
    root.libraries.into_iter().for_each(|mut l| {
        if l.rules.iter().find(|r| !r.calc()).is_none() {
            if l.downloads.artifact.url.len() > 0 {
                libraries.push(File {
                    path: l.downloads.artifact.path,
                    url: l.downloads.artifact.url,
                    size: l.downloads.artifact.size
                });
            }
            if l.natives.len() > 0 {
                let key = l.natives
                    .get(OS_NAME).unwrap()
                    .replace("${arch}", OS_ARCH);
                let artifact = l.downloads.classifiers.remove(&key).unwrap();
                libraries.push(File {
                    path: artifact.path.clone(),
                    url: artifact.url,
                    size: artifact.size
                });
                if !l.extract.default { natives.push(artifact.path) };
            }
        }
    });
    Ok(Version {
        id: root.id, main_class: root.main_class,
        assets, game_args, jvm_args, libraries, natives
    })
}

pub async fn get(id: &str) -> Result<Version> {
    let manifest = manifest::vanilla::get().await?;
    match manifest.get(id) {
        Some(v) => { Ok(parse(&v.url).await?) }
        None => { Err(anyhow!("")) }
    }
}