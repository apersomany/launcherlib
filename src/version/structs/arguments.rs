use serde_derive::Deserialize;

use super::rule::Rule;

#[derive(Debug, Deserialize)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
    pub jvm: Vec<Argument>
}

impl std::default::Default for Arguments {
    fn default() -> Self {
        Self {
            game: Vec::new(),
            jvm: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Raw(Value),
    WithRules {
        rules: Vec<Rule>,
        value: Value
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Single(String),
    Multiple(Vec<String>)
}