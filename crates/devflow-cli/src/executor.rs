use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use devflow_core::{CommandRef, DevflowConfig, PrimaryCommand};

#[derive(Debug, Clone, Copy)]
enum Stack {
    Rust,
    Node,
}

pub fn run(cfg: &DevflowConfig, command: &CommandRef) -> Result<()> {
    let stacks = resolve_stacks(cfg);
    let mut attempted = false;

    for stack in stacks {
        if !stack_is_applicable(stack) {
            println!("skip {}: manifest not found", stack_name(stack));
            continue;
        }

        let effective = with_default_selector(command);
        let Some(argv) = map_command(stack, &effective) else {
            println!(
                "skip {}: unsupported command {}",
                stack_name(stack),
                effective.canonical()
            );
            continue;
        };

        attempted = true;
        println!("run {} on {}", effective.canonical(), stack_name(stack));
        run_argv(&argv).with_context(|| {
            format!("{} failed for {}", effective.canonical(), stack_name(stack))
        })?;
    }

    if !attempted {
        bail!(
            "command '{}' did not match any runnable stack",
            command.canonical()
        );
    }

    Ok(())
}

fn resolve_stacks(cfg: &DevflowConfig) -> Vec<Stack> {
    cfg.project
        .stack
        .iter()
        .filter_map(|value| match value.as_str() {
            "rust" => Some(Stack::Rust),
            "node" => Some(Stack::Node),
            _ => {
                println!("warn: unknown stack '{}' ignored", value);
                None
            }
        })
        .collect()
}

fn with_default_selector(command: &CommandRef) -> CommandRef {
    if command.selector.is_some() {
        return command.clone();
    }

    let selector = match command.primary {
        PrimaryCommand::Setup => "doctor",
        PrimaryCommand::Fmt => "check",
        PrimaryCommand::Lint => "static",
        PrimaryCommand::Build => "debug",
        PrimaryCommand::Test => "unit",
        PrimaryCommand::Package => "artifact",
        PrimaryCommand::Check => "pr",
        PrimaryCommand::Release => "candidate",
        PrimaryCommand::Ci => "check",
    };

    CommandRef {
        primary: command.primary,
        selector: Some(selector.to_string()),
    }
}

fn stack_is_applicable(stack: Stack) -> bool {
    match stack {
        Stack::Rust => Path::new("Cargo.toml").exists(),
        Stack::Node => Path::new("package.json").exists(),
    }
}

fn stack_name(stack: Stack) -> &'static str {
    match stack {
        Stack::Rust => "rust",
        Stack::Node => "node",
    }
}

fn map_command(stack: Stack, cmd: &CommandRef) -> Option<Vec<&'static str>> {
    match stack {
        Stack::Rust => map_rust(cmd),
        Stack::Node => map_node(cmd),
    }
}

fn map_rust(cmd: &CommandRef) -> Option<Vec<&'static str>> {
    let selector = cmd.selector.as_deref().unwrap_or("");

    match (cmd.primary, selector) {
        (PrimaryCommand::Setup, "toolchain") => Some(vec!["rustup", "show"]),
        (PrimaryCommand::Setup, "deps") => Some(vec!["cargo", "fetch"]),
        (PrimaryCommand::Setup, "doctor") => Some(vec!["cargo", "--version"]),
        (PrimaryCommand::Fmt, "check") => Some(vec!["cargo", "fmt", "--all", "--", "--check"]),
        (PrimaryCommand::Fmt, "fix") => Some(vec!["cargo", "fmt", "--all"]),
        (PrimaryCommand::Lint, "static") => Some(vec![
            "cargo",
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ]),
        (PrimaryCommand::Build, "debug") => Some(vec!["cargo", "build"]),
        (PrimaryCommand::Build, "release") => Some(vec!["cargo", "build", "--release"]),
        (PrimaryCommand::Test, "unit") => Some(vec!["cargo", "test", "--lib", "--bins"]),
        (PrimaryCommand::Test, "integration") => Some(vec!["cargo", "test", "--tests"]),
        (PrimaryCommand::Test, "smoke") => Some(vec!["cargo", "test", "smoke"]),
        (PrimaryCommand::Package, "artifact") => Some(vec!["cargo", "build", "--release"]),
        (PrimaryCommand::Release, "candidate") => Some(vec!["cargo", "build", "--release"]),
        _ => None,
    }
}

fn map_node(cmd: &CommandRef) -> Option<Vec<&'static str>> {
    let selector = cmd.selector.as_deref().unwrap_or("");

    match (cmd.primary, selector) {
        (PrimaryCommand::Setup, "deps") => Some(vec!["npm", "ci"]),
        (PrimaryCommand::Setup, "doctor") => Some(vec!["npm", "--version"]),
        (PrimaryCommand::Fmt, "check") => Some(vec!["npm", "run", "fmt:check"]),
        (PrimaryCommand::Fmt, "fix") => Some(vec!["npm", "run", "fmt:fix"]),
        (PrimaryCommand::Lint, "static") => Some(vec!["npm", "run", "lint"]),
        (PrimaryCommand::Build, "debug") => Some(vec!["npm", "run", "build"]),
        (PrimaryCommand::Build, "release") => Some(vec!["npm", "run", "build"]),
        (PrimaryCommand::Test, "unit") => Some(vec!["npm", "run", "test:unit"]),
        (PrimaryCommand::Test, "integration") => Some(vec!["npm", "run", "test:integration"]),
        (PrimaryCommand::Test, "smoke") => Some(vec!["npm", "run", "test:smoke"]),
        (PrimaryCommand::Package, "artifact") => Some(vec!["npm", "pack", "--dry-run"]),
        _ => None,
    }
}

fn run_argv(argv: &[&str]) -> Result<()> {
    let (program, args) = argv
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("empty command argv"))?;

    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("failed to start command '{} {}'", program, args.join(" ")))?;

    if !status.success() {
        bail!(
            "command failed with status {}: {} {}",
            status,
            program,
            args.join(" ")
        );
    }

    Ok(())
}
