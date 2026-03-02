use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use anyhow::{bail, Result};

use crate::command::CommandRef;
use crate::config::DevflowConfig;
use tracing::{debug, instrument};

pub mod subprocess;

/// The action an extension wishes to execute for a given command.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionAction {
    /// The executable program (e.g., "cargo", "npm").
    pub program: String,
    /// The arguments to pass to the program.
    pub args: Vec<String>,
    /// Optional environment variables to set for the execution.
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// A contract for all extensions connecting to Devflow.
pub trait Extension: std::fmt::Debug {
    /// Unique name of the extension.
    fn name(&self) -> &str;
    /// The set of command capabilities provided by this extension.
    fn capabilities(&self) -> HashSet<String>;
    /// Maps a command reference to an executable action.
    fn build_action(&self, cmd: &CommandRef) -> Result<Option<ExecutionAction>>;

    /// Whether this extension is considered "trusted" to run on the host during negotiation.
    fn is_trusted(&self) -> bool {
        false
    }

    /// Returns the host-to-container volume mappings required by this extension.
    /// Expected format: `host_relative_dir:container_absolute_dir`
    /// Example: `rust/cargo:/usr/local/cargo`
    fn cache_mounts(&self) -> Vec<String> {
        Vec::new()
    }

    /// Returns the environment variables required by this extension for execution.
    fn env_vars(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Returns a list of files or globs that constitute the execution fingerprint identity.
    /// Example: `["rust-toolchain.toml", "Cargo.lock"]`
    fn fingerprint_inputs(&self) -> Vec<String> {
        Vec::new()
    }
}

/// A registry containing all discovered Devflow extensions.
#[derive(Debug, Default)]
pub struct ExtensionRegistry {
    extensions: HashMap<String, Box<dyn Extension>>,
}

impl ExtensionRegistry {
    /// Discovers extensions based on the provided configuration.
    ///
    /// If no extensions are explicitly configured, it attempts to load
    /// builtin extensions based on the project stack.
    pub fn discover(config: &DevflowConfig) -> Result<Self> {
        debug!(
            "discovering extensions for project: {}",
            config.project.name
        );
        let registry = Self::default();
        // Discovery logic will be delegated to the CLI / wiring phase,
        // but for backwards capability during refactor, we accept the config
        // even if we leave the registry empty.
        Ok(registry)
    }

    /// Registers a new extension into the registry.
    pub fn register(&mut self, extension: Box<dyn Extension>) {
        self.extensions
            .insert(extension.name().to_string(), extension);
    }

    /// Retrieves an extension by name.
    pub fn get(&self, name: &str) -> Option<&dyn Extension> {
        self.extensions.get(name).map(|boxed| boxed.as_ref())
    }

    /// Verifies if any registered extension can handle the given command.
    ///
    /// # Errors
    /// Returns an error if no registered extension exposes the required capability.
    #[instrument(skip(self))]
    pub fn ensure_can_run(&self, cmd: &CommandRef) -> Result<()> {
        debug!("checking capability support for: {}", cmd.canonical());
        if self.extensions.is_empty() {
            // Command planning still works without explicit extensions for early bootstrap.
            return Ok(());
        }

        let selector_key = cmd
            .selector
            .as_ref()
            .map(|selector| format!("{}:{}", cmd.primary.as_str(), selector));
        let primary_key = cmd.primary.as_str().to_string();

        let supported = self.extensions.values().any(|ext| {
            ext.capabilities().contains(&primary_key)
                || selector_key
                    .as_ref()
                    .map(|s| ext.capabilities().contains(s))
                    .unwrap_or(false)
        });

        if supported {
            return Ok(());
        }

        bail!(
            "no extension exposes capability '{}'",
            selector_key.unwrap_or(primary_key)
        )
    }

    /// Validates that all commands defined in the project targets are supported by at least one extension.
    pub fn validate_target_support(&self, cfg: &DevflowConfig) -> Result<()> {
        if self.extensions.is_empty() {
            return Ok(());
        }

        for (profile, commands) in &cfg.targets.profiles {
            for raw in commands {
                let cmd = CommandRef::from_str(raw)?;
                self.ensure_can_run(&cmd).map_err(|e| {
                    anyhow::anyhow!(
                        "unsupported command '{}' in targets profile '{}': {}",
                        raw,
                        profile,
                        e
                    )
                })?;
            }
        }

        Ok(())
    }

    /// Builds the execution arguments for a command against a specific extension.
    pub fn build_action(&self, name: &str, cmd: &CommandRef) -> Result<Option<ExecutionAction>> {
        if let Some(ext) = self.extensions.get(name) {
            let mut action = match ext.build_action(cmd)? {
                Some(a) => a,
                None => return Ok(None),
            };
            // Merge extension global envs with action-specific envs
            let mut merged_env = ext.env_vars();
            merged_env.extend(action.env);
            action.env = merged_env;
            Ok(Some(action))
        } else {
            Ok(None)
        }
    }

    /// Aggregates all cache mounts requested by the active extensions.
    /// Used by the container executor to map generic host directories.
    pub fn all_cache_mounts(&self) -> Vec<String> {
        let mut mounts = HashSet::new();
        for ext in self.extensions.values() {
            for mount in ext.cache_mounts() {
                mounts.insert(mount);
            }
        }
        let mut sorted: Vec<String> = mounts.into_iter().collect();
        sorted.sort();
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::PrimaryCommand;

    #[derive(Debug)]
    struct MockExtension {
        name: String,
        capabilities: HashSet<String>,
        action: Option<ExecutionAction>,
    }

    impl Extension for MockExtension {
        fn name(&self) -> &str {
            &self.name
        }

        fn capabilities(&self) -> HashSet<String> {
            self.capabilities.clone()
        }

        fn build_action(&self, _cmd: &CommandRef) -> Result<Option<ExecutionAction>> {
            Ok(self.action.clone())
        }
    }

    #[test]
    fn register_and_retrieve_action() {
        let mut registry = ExtensionRegistry::default();
        let ext = MockExtension {
            name: "mock".to_string(),
            capabilities: HashSet::from(["test".to_string()]),
            action: Some(ExecutionAction {
                program: "echo".to_string(),
                args: vec!["hello".to_string()],
                env: HashMap::new(),
            }),
        };

        registry.register(Box::new(ext));

        let cmd = CommandRef {
            primary: PrimaryCommand::Test,
            selector: None,
        };

        let action = registry.build_action("mock", &cmd).unwrap().unwrap();
        assert_eq!(action.program, "echo");
        assert_eq!(action.args, vec!["hello"]);

        let missing = registry.build_action("nonexistent", &cmd).unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn ensure_can_run_primary_match() {
        let mut registry = ExtensionRegistry::default();
        let ext = MockExtension {
            name: "mock".to_string(),
            capabilities: HashSet::from(["test".to_string()]),
            action: None,
        };
        registry.register(Box::new(ext));

        let cmd_supported = CommandRef {
            primary: PrimaryCommand::Test,
            selector: None,
        };
        assert!(registry.ensure_can_run(&cmd_supported).is_ok());

        let cmd_unsupported = CommandRef {
            primary: PrimaryCommand::Build,
            selector: None,
        };
        assert!(registry.ensure_can_run(&cmd_unsupported).is_err());
    }

    #[test]
    fn ensure_can_run_selector_match() {
        let mut registry = ExtensionRegistry::default();
        let ext = MockExtension {
            name: "mock".to_string(),
            capabilities: HashSet::from(["test:lint".to_string(), "fmt".to_string()]),
            action: None,
        };
        registry.register(Box::new(ext));

        // Exact match on selector
        let cmd_supported = CommandRef {
            primary: PrimaryCommand::Test,
            selector: Some("lint".to_string()),
        };
        assert!(registry.ensure_can_run(&cmd_supported).is_ok());

        // We only support test:lint, pure "test" or "test:unit" is not explicitly supported here
        let cmd_unsupported_selector = CommandRef {
            primary: PrimaryCommand::Test,
            selector: Some("unit".to_string()),
        };
        assert!(registry.ensure_can_run(&cmd_unsupported_selector).is_err());
    }

    #[test]
    fn get_returns_registered_extension() {
        let mut registry = ExtensionRegistry::default();
        let ext = MockExtension {
            name: "test-ext".to_string(),
            capabilities: HashSet::new(),
            action: None,
        };
        registry.register(Box::new(ext));

        assert!(registry.get("test-ext").is_some());
        assert_eq!(registry.get("test-ext").unwrap().name(), "test-ext");
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn is_trusted_default_returns_false() {
        let ext = MockExtension {
            name: "untrusted".to_string(),
            capabilities: HashSet::new(),
            action: None,
        };
        // The default trait impl for is_trusted() returns false.
        // MockExtension does not override it, so it should be false.
        assert!(!ext.is_trusted());
    }

    // A configurable mock that lets tests control all Extension trait methods.
    #[derive(Debug)]
    struct ConfigurableMockExtension {
        ext_name: String,
        capabilities: HashSet<String>,
        action: Option<ExecutionAction>,
        trusted: bool,
        mounts: Vec<String>,
        envs: HashMap<String, String>,
    }

    impl Extension for ConfigurableMockExtension {
        fn name(&self) -> &str {
            &self.ext_name
        }
        fn capabilities(&self) -> HashSet<String> {
            self.capabilities.clone()
        }
        fn build_action(&self, _cmd: &CommandRef) -> Result<Option<ExecutionAction>> {
            Ok(self.action.clone())
        }
        fn is_trusted(&self) -> bool {
            self.trusted
        }
        fn cache_mounts(&self) -> Vec<String> {
            self.mounts.clone()
        }
        fn env_vars(&self) -> HashMap<String, String> {
            self.envs.clone()
        }
    }

    #[test]
    fn all_cache_mounts_aggregates_and_deduplicates() {
        let mut registry = ExtensionRegistry::default();

        registry.register(Box::new(ConfigurableMockExtension {
            ext_name: "rust".to_string(),
            capabilities: HashSet::new(),
            action: None,
            trusted: true,
            mounts: vec![
                "rust/cargo:/workspace/.cargo".to_string(),
                "shared:/cache".to_string(),
            ],
            envs: HashMap::new(),
        }));

        registry.register(Box::new(ConfigurableMockExtension {
            ext_name: "node".to_string(),
            capabilities: HashSet::new(),
            action: None,
            trusted: true,
            mounts: vec![
                "node/npm:/root/.npm".to_string(),
                "shared:/cache".to_string(), // duplicate
            ],
            envs: HashMap::new(),
        }));

        let mounts = registry.all_cache_mounts();
        // Deduplicated: "shared:/cache" appears once
        assert_eq!(mounts.len(), 3);
        // Sorted alphabetically
        assert_eq!(mounts[0], "node/npm:/root/.npm");
        assert_eq!(mounts[1], "rust/cargo:/workspace/.cargo");
        assert_eq!(mounts[2], "shared:/cache");
    }

    #[test]
    fn build_action_merges_env_vars_with_action_override() {
        let mut registry = ExtensionRegistry::default();

        let mut ext_envs = HashMap::new();
        ext_envs.insert("CARGO_HOME".to_string(), "/default/cargo".to_string());
        ext_envs.insert("CI".to_string(), "true".to_string());

        let mut action_envs = HashMap::new();
        action_envs.insert("CARGO_HOME".to_string(), "/override/cargo".to_string());
        action_envs.insert("EXTRA".to_string(), "value".to_string());

        registry.register(Box::new(ConfigurableMockExtension {
            ext_name: "rust".to_string(),
            capabilities: HashSet::new(),
            action: Some(ExecutionAction {
                program: "cargo".to_string(),
                args: vec!["build".to_string()],
                env: action_envs,
            }),
            trusted: true,
            mounts: Vec::new(),
            envs: ext_envs,
        }));

        let cmd = CommandRef {
            primary: PrimaryCommand::Build,
            selector: None,
        };

        let action = registry.build_action("rust", &cmd).unwrap().unwrap();
        // Action-level env overrides extension-level
        assert_eq!(action.env.get("CARGO_HOME").unwrap(), "/override/cargo");
        // Extension-level env is preserved when not overridden
        assert_eq!(action.env.get("CI").unwrap(), "true");
        // Action-specific env is preserved
        assert_eq!(action.env.get("EXTRA").unwrap(), "value");
    }

    #[test]
    fn all_cache_mounts_empty_when_no_extensions() {
        let registry = ExtensionRegistry::default();
        assert!(registry.all_cache_mounts().is_empty());
    }
}
