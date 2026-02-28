use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::command::CommandRef;
use crate::runtime::RuntimeProfile;

/// The root configuration structure for a Devflow project.
///
/// This structure is typically deserialized from a `devflow.toml` file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevflowConfig {
    /// Basic project metadata.
    pub project: ProjectConfig,
    /// Runtime settings (e.g., local, CI).
    #[serde(default)]
    pub runtime: RuntimeConfig,
    /// Custom target profiles (e.g., `pr`, `main`, `release`).
    #[serde(default)]
    pub targets: TargetsConfig,
    /// Optional extension configurations.
    pub extensions: Option<HashMap<String, ExtensionConfig>>,
    /// Container configuration placeholders (for future use).
    #[serde(default)]
    pub container: Option<ContainerConfig>,
    /// Cache configuration placeholders (for future use).
    #[serde(default)]
    pub cache: Option<CacheConfig>,
    /// Path to the directory containing this config file, used to anchor relative paths.
    #[serde(skip)]
    pub source_dir: Option<PathBuf>,
}

impl DevflowConfig {
    /// Loads a `DevflowConfig` from a TOML file at the given path.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read, the TOML is invalid,
    /// or the configuration fails validation.
    pub fn load_from_file(path: &str) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {path}"))?;
        let mut cfg = toml::from_str::<Self>(&text)
            .with_context(|| format!("failed to parse TOML config: {path}"))?;

        cfg.source_dir = Some(
            PathBuf::from(path)
                .parent()
                .unwrap_or(std::path::Path::new(""))
                .to_path_buf(),
        );
        cfg.validate()?;
        Ok(cfg)
    }

    /// Validates the configuration for logical consistency.
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

/// Metadata about the project.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    /// Name of the project.
    pub name: String,
    /// Technology stacks used in the project (e.g., "rust", "node").
    pub stack: Vec<String>,
}

/// Configuration for the Devflow runtime.
#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    /// The current runtime profile.
    #[serde(default)]
    pub profile: RuntimeProfile,
}

/// Placeholder for container configuration.
#[derive(Debug, Deserialize, Default)]
pub struct ContainerConfig {
    pub image: Option<String>,
    #[serde(default)]
    pub fingerprint_inputs: Vec<String>,
}

/// Placeholder for cache configuration.
#[derive(Debug, Deserialize, Default)]
pub struct CacheConfig {
    pub root: Option<String>,
    pub strategy: Option<String>,
}

/// Configuration for target profiles.
///
/// Maps profile names (e.g., "pr") to a list of command strings.
#[derive(Debug, Deserialize, Default)]
pub struct TargetsConfig {
    /// A map of profile names to command lists.
    #[serde(flatten, default)]
    pub profiles: HashMap<String, Vec<String>>,
}

/// Configuration for an individual extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionConfig {
    /// Where the extension is sourced from (builtin or path).
    pub source: ExtensionSource,
    /// Optional path for path-sourced extensions.
    pub path: Option<PathBuf>,
    /// Optional version string.
    pub version: Option<String>,
    /// The API version the extension expects.
    pub api_version: Option<u32>,
    /// List of capabilities exposed by the extension.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Whether this extension is required for project operations.
    #[serde(default)]
    pub required: bool,
}

/// Source types for extensions.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionSource {
    /// A builtin extension bundled with the Devflow binary.
    Builtin,
    /// An extension loaded from a local directory.
    Path,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_top_level_key() {
        // Ensures that the configuration parser rejects unknown keys at the top level
        // to prevent silent errors from typos.
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
        // Ensures that the [project] section does not allow unauthorized keys.
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

    #[test]
    fn unit_test_validate_rejects_unsupported_stack() {
        let text = r#"
        [project]
        name = "invalid-stack"
        stack = ["ruby"]
        "#;

        let cfg = toml::from_str::<DevflowConfig>(text).expect("Valid TOML parse");
        let err = cfg
            .validate()
            .expect_err("Must fail validation for unsupported stack");
        assert!(err.to_string().contains("unsupported stack 'ruby'"));
    }

    #[test]
    fn unit_test_validate_rejects_invalid_commands() {
        let text = r#"
        [project]
        name = "bad-commands"
        stack = ["rust"]

        [targets]
        pr = ["build:debug", "not-a-command:selector"]
        "#;

        let cfg = toml::from_str::<DevflowConfig>(text).expect("Valid TOML parse");
        let err = cfg
            .validate()
            .expect_err("Must fail validation for invalid command format");

        // CommandRef from_str returns an error because primary command is unknown
        assert!(err
            .to_string()
            .contains("invalid command 'not-a-command:selector'"));
    }

    #[test]
    fn integration_test_load_from_file_anchors_source_dir() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("devflow.toml");

        let text = r#"
        [project]
        name = "success-load"
        stack = ["node"]
        "#;

        std::fs::write(&config_path, text).unwrap();

        // Should successfully parse, validate, and anchor `source_dir`
        let cfg = DevflowConfig::load_from_file(config_path.to_str().unwrap()).unwrap();
        assert_eq!(cfg.project.name, "success-load");
        assert_eq!(cfg.project.stack, vec!["node"]);
        assert_eq!(cfg.source_dir, Some(dir.path().to_path_buf()));
    }

    #[test]
    fn security_boundary_test_load_missing_or_malformed_file() {
        // Missing file
        let err = DevflowConfig::load_from_file("/tmp/nonexistent-devflow-random-file.toml")
            .expect_err("Missing file should return an error, not panic");
        assert!(err.to_string().contains("failed to read config file"));

        // Malformed TOML file
        let dir = tempfile::tempdir().unwrap();
        let malformed_path = dir.path().join("malformed.toml");
        std::fs::write(&malformed_path, "[project\nname=\"missing-bracket\"").unwrap();

        let err = DevflowConfig::load_from_file(malformed_path.to_str().unwrap())
            .expect_err("Malformed TOML should return an error, not panic");
        assert!(err.to_string().contains("failed to parse TOML"));
    }
}
