use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::runtime::RuntimeProfile;

#[derive(Debug, Deserialize)]
pub struct DevflowConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub targets: TargetsConfig,
    pub extensions: Option<HashMap<String, ExtensionConfig>>,
}

impl DevflowConfig {
    pub fn load_from_file(path: &str) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {path}"))?;
        let cfg = toml::from_str::<Self>(&text)
            .with_context(|| format!("failed to parse TOML config: {path}"))?;
        Ok(cfg)
    }
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub stack: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub profile: RuntimeProfile,
}

#[derive(Debug, Deserialize, Default)]
pub struct TargetsConfig {
    #[serde(default)]
    pub pr: Vec<String>,
    #[serde(default)]
    pub main: Vec<String>,
    #[serde(default)]
    pub release: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionConfig {
    pub source: ExtensionSource,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub api_version: Option<u32>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionSource {
    Builtin,
    Path,
}
