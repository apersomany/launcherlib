use std::collections::HashMap;
use serde_derive::Deserialize;

const OS_NAME: &str =
    if cfg!(target_os = "macos") { "osx" }
    else { std::env::consts::OS };

const OS_ARCH: &str = 
    if cfg!(target_arch = "x86_64") { "x86" }
    else { std::env::consts::ARCH };

#[derive(Debug, Deserialize)]
pub struct Rule {
    action: String,
    #[serde(default)]
    os: HashMap<String, String>,
    #[serde(default)]
    features: HashMap<String, bool>
}

impl Rule {
    pub fn calc(&self) -> bool {
        let result = self.action == "allow";
        if self.features.len() > 0 { return !result }
        if self.os.contains_key("name") {
            if !(self.os.get("name").unwrap() == OS_NAME) {
                return !result
            }
        }
        if self.os.contains_key("arch") {
            if !(self.os.get("arch").unwrap() == OS_ARCH) {
                return !result
            }
        }
        result
    }
}