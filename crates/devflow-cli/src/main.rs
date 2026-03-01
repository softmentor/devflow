use std::str::FromStr;
use std::{fs, path::Path};

use anyhow::{anyhow, Context, Result};
use clap::Parser;

use devflow_core::{CommandRef, DevflowConfig, ExtensionRegistry, PrimaryCommand};
use tracing::debug;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod discovery;
mod executor;
mod init;

/// The command-line interface for Devflow.
#[derive(Debug, Parser)]
#[command(name = "dwf")]
#[command(about = "Devflow CLI - Modern developer workflow automation")]
pub(crate) struct Cli {
    /// Command in canonical form, for example: `check:pr`, `fmt:fix`, `test:unit`
    command: Option<String>,
    /// Optional selector (supports `dwf test unit` style)
    selector: Option<String>,
    /// Path to devflow config file.
    #[arg(long, default_value = "devflow.toml")]
    config: String,
    /// Print generated CI workflow to stdout instead of writing to file.
    #[arg(long, default_value_t = false)]
    stdout: bool,
    /// Output path for `ci:generate` when writing files.
    #[arg(long, default_value = ".github/workflows/ci.yml")]
    ci_output: String,
    /// Overwrite generated files if they already exist.
    #[arg(long, default_value_t = false)]
    force: bool,
}

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let cli = Cli::parse();
    debug!("parsed cli arguments: {:?}", cli);

    let command_name = match &cli.command {
        Some(cmd) => cmd,
        None => {
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!(); // Add a newline after help
            return Ok(());
        }
    };

    let command_text = match &cli.selector {
        Some(selector) => format!("{}:{}", command_name, selector),
        None => command_name.clone(),
    };

    let command = CommandRef::from_str(&command_text)
        .map_err(|e| anyhow!("failed to parse command '{}': {e}", command_text))?;

    if command.primary == PrimaryCommand::Init {
        return init::run(&cli, command.selector.as_deref());
    }

    let cfg = DevflowConfig::load_from_file(&cli.config)
        .with_context(|| format!("unable to load config '{}'", cli.config))?;
    let mut registry = ExtensionRegistry::discover(&cfg)?;

    // Phase 1 Wiring: Explicitly compile in the required trait implementations
    registry.register(Box::new(devflow_ext_rust::RustExtension::new()));
    registry.register(Box::new(devflow_ext_node::NodeExtension::new()));

    // Phase 2 Wiring: Runtime discovery of Subprocess Extensions
    discovery::discover_subprocess_extensions(&cfg, &mut registry)?;

    registry.validate_target_support(&cfg)?;

    execute(&cli, &cfg, &registry, &command)
}

/// Executes a validated Devflow command.
fn execute(
    cli: &Cli,
    cfg: &DevflowConfig,
    registry: &ExtensionRegistry,
    command: &CommandRef,
) -> Result<()> {
    match command.primary {
        PrimaryCommand::Check => {
            let selector = command.selector.as_deref().unwrap_or("pr");
            let resolved = devflow_policy::resolve_policy_commands(cfg, selector)?;
            println!("check:{selector} (runtime={:?})", cfg.runtime.profile);
            for cmd in resolved {
                registry.ensure_can_run(&cmd)?;
                println!(" - {}", cmd);
                executor::run(cfg, registry, &cmd)?;
            }
            Ok(())
        }
        PrimaryCommand::Ci if command.selector.as_deref() == Some("generate") => {
            let workflow = devflow_gh::render_workflow(cfg)?;
            if cli.stdout {
                println!("{workflow}");
            } else {
                write_ci_workflow(&cli.ci_output, &workflow)?;
                println!("ci:generate wrote {}", cli.ci_output);
            }
            Ok(())
        }
        PrimaryCommand::Ci if command.selector.as_deref() == Some("check") => {
            let expected = devflow_gh::render_workflow(cfg)?;
            let actual = read_ci_workflow(&cli.ci_output)?;
            devflow_gh::check_workflow(cfg, &actual)?;
            if actual != expected {
                return Err(anyhow!(
                    "ci workflow drift detected in '{}': run 'dwf ci:generate' to resync",
                    cli.ci_output
                ));
            }
            println!("ci:check passed");
            Ok(())
        }
        PrimaryCommand::Ci if command.selector.as_deref() == Some("plan") => {
            let profiles = cfg
                .targets
                .profiles
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            println!("ci:plan profiles=[{}]", profiles);
            Ok(())
        }
        _ => {
            registry.ensure_can_run(command)?;
            executor::run(cfg, registry, command)
        }
    }
}

fn write_ci_workflow(path: &str, content: &str) -> Result<()> {
    let output = Path::new(path);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory '{}'", parent.display()))?;
    }
    fs::write(output, content).with_context(|| format!("failed to write '{}'", output.display()))
}

fn read_ci_workflow(path: &str) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read '{}'", path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use devflow_core::config::{ProjectConfig, RuntimeConfig};
    use tempfile::tempdir;

    fn test_cfg() -> DevflowConfig {
        let mut profiles = std::collections::HashMap::new();
        profiles.insert("pr".to_string(), vec!["test:unit".to_string()]);

        DevflowConfig {
            project: ProjectConfig {
                name: "test-main".to_string(),
                stack: vec!["rust".to_string()],
            },
            runtime: RuntimeConfig::default(),
            targets: devflow_core::config::TargetsConfig { profiles },
            extensions: None,
            container: None,
            cache: None,
            source_dir: None,
        }
    }

    fn test_cli(ci_output: &str) -> Cli {
        Cli {
            command: Some("ci".to_string()),
            selector: None,
            config: "devflow.toml".to_string(),
            stdout: true,
            ci_output: ci_output.to_string(),
            force: false,
        }
    }

    #[test]
    fn smoke_test_execute_ci_plan() {
        let cfg = test_cfg();
        let registry = ExtensionRegistry::default();
        let cmd = CommandRef::from_str("ci:plan").unwrap();
        let cli = test_cli("none");

        // Should print CI plan logic without failing
        assert!(execute(&cli, &cfg, &registry, &cmd).is_ok());
    }

    #[test]
    fn smoke_test_execute_ci_generate_stdout() {
        let cfg = test_cfg();
        let registry = ExtensionRegistry::default();
        let cmd = CommandRef::from_str("ci:generate").unwrap();
        let mut cli = test_cli("none");
        cli.stdout = true;

        // Should print generated workflows without filesystem interaction
        execute(&cli, &cfg, &registry, &cmd).expect("execute ci:generate stdout failed");
    }

    #[test]
    fn integration_test_execute_ci_generate_filesystem() {
        let dir = tempdir().unwrap();
        let ci_path = dir.path().join("ci.yml");

        let cfg = test_cfg();
        let registry = ExtensionRegistry::default();
        let cmd = CommandRef::from_str("ci:generate").unwrap();

        // Write to filesystem
        let mut cli = test_cli(ci_path.to_str().unwrap());
        cli.stdout = false;

        execute(&cli, &cfg, &registry, &cmd).expect("execute ci:generate filesystem failed");
        assert!(ci_path.exists());
        let content = fs::read_to_string(&ci_path).unwrap();
        assert!(content.contains("test:unit"));
    }
}
