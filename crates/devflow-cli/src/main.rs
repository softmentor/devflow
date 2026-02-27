use std::str::FromStr;
use std::{fs, path::Path};

use anyhow::{anyhow, Context, Result};
use clap::Parser;

use devflow_core::{CommandRef, DevflowConfig, ExtensionRegistry, PrimaryCommand};
use tracing::debug;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod executor;
mod init;

/// The command-line interface for Devflow.
#[derive(Debug, Parser)]
#[command(name = "dwf")]
#[command(about = "Devflow CLI - Modern developer workflow automation")]
pub(crate) struct Cli {
    /// Command in canonical form, for example: `check:pr`, `fmt:fix`, `test:unit`
    command: String,
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

    let command_text = match &cli.selector {
        Some(selector) => format!("{}:{}", cli.command, selector),
        None => cli.command.clone(),
    };

    let command = CommandRef::from_str(&command_text)
        .map_err(|e| anyhow!("failed to parse command '{}': {e}", command_text))?;

    if command.primary == PrimaryCommand::Init {
        return init::run(&cli, command.selector.as_deref());
    }

    let cfg = DevflowConfig::load_from_file(&cli.config)
        .with_context(|| format!("unable to load config '{}'", cli.config))?;
    let registry = ExtensionRegistry::discover(&cfg)?;
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
                executor::run(cfg, &cmd)?;
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
            executor::run(cfg, command)
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
