use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::command::CommandRef;
use crate::runtime::RuntimeProfile;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        for stack in &self.project.stack {
            match stack.as_str() {
                "rust" | "node" | "custom" => {}
                other => {
                    return Err(anyhow!(
                        "unsupported stack '{}' (supported: rust,node,custom)",
                        other
                    ));
                }
            }
        }

        for (profile, commands) in &self.targets.profiles {
            for raw in commands {
                CommandRef::from_str(raw).map_err(|e| {
                    anyhow!(
                        "invalid command '{}' in targets profile '{}': {}",
                        raw,
                        profile,
                        e
                    )
                })?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub name: String,
    pub stack: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub profile: RuntimeProfile,
}

#[derive(Debug, Deserialize, Default)]
pub struct TargetsConfig {
    #[serde(flatten, default)]
    pub profiles: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_top_level_key() {
        let text = r#"
        random_top = "not-allowed"

        [project]
        name = "demo"
        stack = ["rust"]

        [targets]
        pr = ["fmt:check"]
        "#;

        let err = toml::from_str::<DevflowConfig>(text).expect_err("must reject unknown key");
        assert!(err.to_string().contains("random_top"));
    }

    #[test]
    fn rejects_unknown_project_key() {
        let text = r#"
        [project]
        name = "demo"
        stack = ["rust"]
        owner = "team"

        [targets]
        pr = ["fmt:check"]
        "#;

        let err =
            toml::from_str::<DevflowConfig>(text).expect_err("must reject unknown project key");
        assert!(err.to_string().contains("owner"));
    }
}
