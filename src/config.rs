use serde::{Deserialize, Serialize};

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default, deny_unknown_fields)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

impl Config {
    pub fn load(c: String) -> Result<Self> {
        Ok(toml::from_str(&c)?)
    }
}
