use std::collections::HashMap;
use serde_derive::Deserialize;

use super::rule::Rule;

#[derive(Debug, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Downloads,
    #[serde(default)]
    pub extract: Extract,
    #[serde(default)]
    pub natives: HashMap<String, String>,
    #[serde(default)]
    pub rules: Vec<Rule>
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    #[serde(default)]
    pub artifact: Artifact,
    #[serde(default)]
    pub classifiers: HashMap<String, Artifact>
}

#[derive(Debug, Deserialize)]
pub struct Artifact {
    pub path: String,
    pub url: String,
    pub size: u32
}

impl Default for Artifact {
    fn default() -> Self {
        Self {
            path: String::new(),
            url: String::new(),
            size: 0
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Extract {
    #[serde(default)]
    pub default: bool,
    pub exclude: Vec<String>
}

impl Default for Extract {
    fn default() -> Self {
        Self {
            default: true,
            exclude: Vec::new()
        }
    }
}