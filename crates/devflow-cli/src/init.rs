use std::{fs, path::Path};

use anyhow::{anyhow, Context, Result};

use crate::Cli;
use tracing::{info, instrument};

/// Runs the `init` command to bootstrap a new Devflow project.
#[instrument(skip(cli))]
pub fn run(cli: &Cli, template_selector: Option<&str>) -> Result<()> {
    let config_path = Path::new(&cli.config);
    let parent = config_path.parent().unwrap_or_else(|| Path::new(""));
    let target_dir = if parent.as_os_str().is_empty() {
        std::env::current_dir()?
    } else {
        parent.to_path_buf()
    };

    let project_name = target_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "devflow-project".to_string());

    let template = match template_selector {
        Some(value) => InitTemplate::from_str(value)?,
        None => detect_template(&target_dir)?,
    };

    let config_content = template.render_config(&project_name);
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

    fn render_config(self, project_name: &str) -> String {
        let template = match self {
            Self::Rust => include_str!("../resources/rust.toml"),
            Self::Node => include_str!("../resources/node.toml"),
            Self::Tsc => include_str!("../resources/tsc.toml"),
            Self::Kotlin => include_str!("../resources/kotlin.toml"),
        };

        template
            .replace("my-rust-project", project_name)
            .replace("my-node-project", project_name)
            .replace("my-typescript-project", project_name)
            .replace("my-kotlin-project", project_name)
    }
}

fn detect_template(base_path: &Path) -> Result<InitTemplate> {
    use devflow_core::constants::{MANIFEST_NODE, MANIFEST_RUST, MANIFEST_TSC};

    if base_path.join(MANIFEST_RUST).exists() {
        return Ok(InitTemplate::Rust);
    }

    if base_path.join(MANIFEST_TSC).exists() {
        return Ok(InitTemplate::Tsc);
    }

    if base_path.join(MANIFEST_NODE).exists() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_cli(dir: &Path) -> Cli {
        Cli {
            command: "init".to_string(),
            selector: None,
            config: dir.join("devflow.toml").to_str().unwrap().to_string(),
            stdout: false,
            ci_output: dir
                .join(".github/workflows/ci.yml")
                .to_str()
                .unwrap()
                .to_string(),
            force: false,
        }
    }

    #[test]
    fn unit_test_detect_template() {
        let dir = tempdir().unwrap();
        let base = dir.path();

        // Fails if no indicators
        assert!(detect_template(base).is_err());

        // Detects Node
        fs::write(base.join("package.json"), "{}").unwrap();
        assert!(matches!(detect_template(base).unwrap(), InitTemplate::Node));

        // If Cargo.toml also exists, it should prefer Rust
        fs::write(base.join("Cargo.toml"), "").unwrap();
        assert!(matches!(detect_template(base).unwrap(), InitTemplate::Rust));
    }

    #[test]
    fn unit_test_write_if_absent() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt").to_str().unwrap().to_string();

        // Successful write
        assert!(write_if_absent(&file_path, "hello", false).is_ok());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "hello");

        // Fail to overwrite existing file without force
        assert!(write_if_absent(&file_path, "world", false).is_err());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "hello");

        // Successful overwrite with force
        assert!(write_if_absent(&file_path, "world", true).is_ok());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "world");
    }

    #[test]
    fn integration_test_init_run_success() {
        let dir = tempdir().unwrap();
        let mut cli = test_cli(dir.path());

        // Run against an explicit template
        let result = run(&cli, Some("rust"));
        assert!(result.is_ok());

        // Verify files were generated
        assert!(Path::new(&cli.config).exists());
        assert!(Path::new(&cli.ci_output).exists());

        let dir_name = dir.path().file_name().unwrap().to_str().unwrap();

        let config_str = fs::read_to_string(&cli.config).unwrap();
        assert!(config_str.contains(dir_name));

        // Ensure running again without force fails on the existing configuration
        let duplicate_run = run(&cli, Some("rust"));
        assert!(duplicate_run.is_err());

        // Applying force overrides the configuration
        cli.force = true;
        assert!(run(&cli, Some("node")).is_ok());
        let updated_config = fs::read_to_string(&cli.config).unwrap();
        assert!(updated_config.contains(dir_name));
    }
}
