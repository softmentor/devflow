use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use clap::Parser;

use devflow_core::{CommandRef, DevflowConfig, ExtensionRegistry, PrimaryCommand};

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let command_text = match cli.selector {
        Some(selector) => format!("{}:{}", cli.command, selector),
        None => cli.command,
    };

    // Legacy compatibility aliases.
    let command_text = match command_text.as_str() {
        "verify" => "check".to_string(),
        "smoke" => "test:smoke".to_string(),
        other => other.to_string(),
    };

    let cfg = DevflowConfig::load_from_file(&cli.config)
        .with_context(|| format!("unable to load config '{}'", cli.config))?;
    let registry = ExtensionRegistry::discover(&cfg)?;

    let command = CommandRef::from_str(&command_text)
        .map_err(|e| anyhow!("failed to parse command '{}': {e}", command_text))?;

    execute(&cfg, &registry, &command)
}

fn execute(cfg: &DevflowConfig, registry: &ExtensionRegistry, command: &CommandRef) -> Result<()> {
    match command.primary {
        PrimaryCommand::Check => {
            let selector = command.selector.as_deref().unwrap_or("pr");
            let resolved = devflow_policy::resolve_policy_commands(cfg, selector)?;
            println!("check:{selector} (runtime={:?})", cfg.runtime.profile);
            for cmd in resolved {
                registry.ensure_can_run(&cmd)?;
                println!(" - {}", cmd);
            }
            Ok(())
        }
        PrimaryCommand::Ci if command.selector.as_deref() == Some("render") => {
            let workflow = devflow_gh::render_workflow_stub(cfg)?;
            println!("{workflow}");
            Ok(())
        }
        _ => {
            registry.ensure_can_run(command)?;
            println!(
                "run {} (runtime={:?}, project={}, stack={})",
                command,
                cfg.runtime.profile,
                cfg.project.name,
                cfg.project.stack.join(",")
            );
            Ok(())
        }
    }
}
