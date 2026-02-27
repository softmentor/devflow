use std::{fs, path::Path};

use anyhow::{anyhow, Context, Result};

use crate::Cli;
use tracing::{info, instrument};

/// Runs the `init` command to bootstrap a new Devflow project.
#[instrument(skip(cli))]
pub fn run(cli: &Cli, template_selector: Option<&str>) -> Result<()> {
    let template = match template_selector {
        Some(value) => InitTemplate::from_str(value)?,
        None => detect_template()?,
    };

    let config_content = template.render_config();
    write_if_absent(&cli.config, &config_content, cli.force)
        .with_context(|| format!("failed to write '{}'", cli.config))?;

    let cfg = devflow_core::DevflowConfig::load_from_file(&cli.config)?;
    let workflow = devflow_gh::render_workflow(&cfg)?;

    if cli.stdout {
        println!("{workflow}");
    } else {
        write_if_absent(&cli.ci_output, &workflow, cli.force)
            .with_context(|| format!("failed to write '{}'", cli.ci_output))?;
    }

    info!(
        "init complete: template={}, config={}, ci={}",
        template.as_str(),
        cli.config,
        cli.ci_output
    );
    println!("next: run 'dwf check:pr'");

    Ok(())
}

/// Supported project templates for initialization.
#[derive(Debug, Clone, Copy)]
enum InitTemplate {
    /// Standard Rust project.
    Rust,
    /// Node.js project.
    Node,
    /// TypeScript project with common defaults.
    Tsc,
    /// Kotlin project (custom stack example).
    Kotlin,
}

impl InitTemplate {
    fn from_str(value: &str) -> Result<Self> {
        match value {
            "rust" => Ok(Self::Rust),
            "node" => Ok(Self::Node),
            "tsc" | "typescript" => Ok(Self::Tsc),
            "kotlin" => Ok(Self::Kotlin),
            other => Err(anyhow!(
                "unknown init template '{}' (supported: rust,node,tsc,kotlin)",
                other
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Node => "node",
            Self::Tsc => "tsc",
            Self::Kotlin => "kotlin",
        }
    }

    fn render_config(self) -> String {
        match self {
            Self::Rust => r#"[project]
name = "my-rust-project"
stack = ["rust"]

[runtime]
profile = "auto"

[targets]
pr = ["fmt:check", "lint:static", "build:debug", "test:unit", "test:integration"]
main = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration", "test:smoke"]

[extensions.rust]
source = "builtin"
required = true
"#
            .to_string(),
            Self::Node => r#"[project]
name = "my-node-project"
stack = ["node"]

[runtime]
profile = "auto"

[targets]
pr = ["fmt:check", "lint:static", "build:debug", "test:unit", "test:integration"]
main = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration"]

[extensions.node]
source = "builtin"
required = true
"#
            .to_string(),
            Self::Tsc => r#"[project]
name = "my-typescript-project"
stack = ["node"]

[runtime]
profile = "auto"

[targets]
pr = ["fmt:check", "lint:static", "build:debug", "test:unit"]
main = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration"]

[extensions.node]
source = "builtin"
required = true

# Ensure your package.json scripts expose these selectors:
# fmt:check, lint, build, test:unit, test:integration
"#
            .to_string(),
            Self::Kotlin => r#"[project]
name = "my-kotlin-project"
stack = ["custom"]

[runtime]
profile = "host"

[targets]
pr = ["fmt:check", "lint:static", "build:debug", "test:unit"]
main = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration"]

# Custom stack delegates canonical selectors to just/make targets.
# Implement matching targets in justfile or Makefile:
# fmt-check, lint-static, build-debug, test-unit, test-integration
"#
            .to_string(),
        }
    }
}

fn detect_template() -> Result<InitTemplate> {
    if Path::new("Cargo.toml").exists() {
        return Ok(InitTemplate::Rust);
    }

    if Path::new("tsconfig.json").exists() {
        return Ok(InitTemplate::Tsc);
    }

    if Path::new("package.json").exists() {
        return Ok(InitTemplate::Node);
    }

    Err(anyhow!(
        "unable to auto-detect template. Run: dwf init <rust|node|tsc|kotlin>"
    ))
}

fn write_if_absent(path: &str, content: &str, force: bool) -> Result<()> {
    let output = Path::new(path);

    if output.exists() && !force {
        return Err(anyhow!(
            "'{}' already exists. Re-run with --force to overwrite",
            path
        ));
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory '{}'", parent.display()))?;
    }

    fs::write(output, content)
        .with_context(|| format!("failed to write file '{}'", output.display()))
}
