use std::str::FromStr;
use std::{fs, path::Path};

use anyhow::{anyhow, Context, Result};
use clap::Parser;

use devflow_core::{CommandRef, DevflowConfig, ExtensionRegistry, PrimaryCommand};

mod executor;

#[derive(Debug, Parser)]
#[command(name = "dwf")]
#[command(about = "Devflow CLI")]
struct Cli {
    /// Command in canonical form, for example: check:pr, fmt:fix, test:unit
    command: String,
    /// Optional selector (supports `dwf test unit` style)
    selector: Option<String>,
    /// Path to devflow config
    #[arg(long, default_value = "devflow.toml")]
    config: String,
    /// Print generated CI workflow to stdout instead of writing to file
    #[arg(long, default_value_t = false)]
    stdout: bool,
    /// Output path for `ci:render` when writing files
    #[arg(long, default_value = ".github/workflows/ci.yml")]
    ci_output: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let command_text = match &cli.selector {
        Some(selector) => format!("{}:{}", cli.command, selector),
        None => cli.command.clone(),
    };

    let cfg = DevflowConfig::load_from_file(&cli.config)
        .with_context(|| format!("unable to load config '{}'", cli.config))?;
    let registry = ExtensionRegistry::discover(&cfg)?;
    registry.validate_target_support(&cfg)?;

    let command = CommandRef::from_str(&command_text)
        .map_err(|e| anyhow!("failed to parse command '{}': {e}", command_text))?;

    execute(&cli, &cfg, &registry, &command)
}

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
        PrimaryCommand::Ci if command.selector.as_deref() == Some("render") => {
            let workflow = devflow_gh::render_workflow(cfg)?;
            if cli.stdout {
                println!("{workflow}");
            } else {
                write_ci_workflow(&cli.ci_output, &workflow)?;
                println!("ci:render wrote {}", cli.ci_output);
            }
            Ok(())
        }
        PrimaryCommand::Ci if command.selector.as_deref() == Some("check") => {
            let workflow = devflow_gh::render_workflow(cfg)?;
            devflow_gh::check_workflow(cfg, &workflow)?;
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
