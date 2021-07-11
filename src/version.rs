pub mod forge;
mod structs;
pub mod vanilla;

use anyhow::Result;
use futures::{stream, StreamExt};
use maplit::hashmap;
use serde_derive::*;
use std::{collections::HashMap, io::Cursor};
use tokio::process::Command;
use zip::ZipArchive;

use crate::{context::Context, format};

#[derive(Debug, Deserialize, Serialize)]
pub struct Object {
    pub hash: String,
    pub size: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Assets {
    #[serde(default)]
    pub id: String,
    pub objects: HashMap<String, Object>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    pub url: String,
    pub path: String,
    pub size: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Version {
    pub id: String,
    pub assets: Assets,
    pub game_args: Vec<String>,
    pub jvm_args: Vec<String>,
    pub libraries: Vec<File>,
    pub natives: Vec<String>,
    pub main_class: String,
}

impl Version {
    pub fn from_slice(slice: &[u8]) -> Result<Version> {
        Ok(serde_json::from_slice(slice)?)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn classpath(&self, ctx: &Context) -> String {
        let libs: Vec<String> = self
            .libraries
            .iter()
            .map(|f| ctx.path(&format!("libraries/{}", f.path)))
            .collect();
        libs.join(";")
    }

    pub async fn ensure_libraries(&self, ctx: &Context) {
        stream::iter(&self.libraries)
            .for_each_concurrent(ctx.max_concurrent as usize, |f| async move {
                let path = format!("libraries/{}", f.path);
                match ctx.check_file_size(&path, f.size as u64).await {
                    Ok(_) => {}
                    Err(_) => {
                        let bytes = ctx.get(&f.url).await.unwrap().bytes().await.unwrap();
                        ctx.write_file(&path, &bytes).await.unwrap();
                    }
                }
            })
            .await;
    }

    pub async fn ensure_natives(&self, ctx: &Context) {
        stream::iter(&self.natives)
            .for_each_concurrent(ctx.max_concurrent as usize, |n| async move {
                let jar = ctx.read_file(&format!("libraries/{}", n)).await.unwrap();
                ZipArchive::new(Cursor::new(jar))
                    .unwrap()
                    .extract(&ctx.path("natives"))
                    .unwrap();
            })
            .await;
    }

    pub async fn ensure_assets(&self, ctx: &Context) {
        stream::iter(&self.assets.objects)
            .for_each_concurrent(16, |(_, o)| async move {
                let path1 = format!("{}/{}", &o.hash[0..2], o.hash);
                let path2 = format!("assets/objects/{}", path1);
                match ctx.check_file_size(&path2, o.size as u64).await {
                    Ok(_) => {}
                    Err(_) => {
                        let bytes = ctx
                            .get(&format!(
                                "http://resources.download.minecraft.net/{}",
                                path1
                            ))
                            .await
                            .unwrap()
                            .bytes()
                            .await
                            .unwrap();
                        ctx.write_file(&path2, &bytes).await.unwrap();
                    }
                };
            })
            .await;
        ctx.write_file(
            &format!("assets/indexes/{}.json", self.assets.id),
            &serde_json::to_vec(&self.assets).unwrap(),
        )
        .await
        .unwrap();
    }

    pub async fn ensure_all(&self, ctx: &Context) {
        self.ensure_libraries(ctx).await;
        self.ensure_natives(ctx).await;
        self.ensure_assets(ctx).await;
    }

    pub async fn launch(&self, ctx: &Context, args: Vec<String>, vars: HashMap<&str, &str>) {
        let mut variables = hashmap! {
            "natives_directory" => ctx.path("natives"),
            "assets_root" => ctx.path("assets"),
            "assets_index_name" => self.assets.id.clone(),
            "classpath" => self.classpath(ctx)
        };
        vars.into_iter().for_each(|(k, v)| {
            variables.insert(k, v.to_string());
        });
        let jvm_args: Vec<String> = self
            .jvm_args
            .iter()
            .map(|arg| format(arg, &variables))
            .collect();
        let game_args: Vec<String> = self
            .game_args
            .iter()
            .map(|arg| format(arg, &variables))
            .collect();
        let output = Command::new("java")
            .args(jvm_args)
            .arg(&self.main_class)
            .args(args)
            .args(game_args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
    }
}
