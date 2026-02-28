use anyhow::{anyhow, Result};
use tracing::{debug, instrument};

use devflow_core::DevflowConfig;

#[instrument(skip(cfg))]
pub fn render_workflow(cfg: &DevflowConfig) -> Result<String> {
    debug!("rendering workflow for project: {}", cfg.project.name);
    let pr = cfg
        .targets
        .profiles
        .get("pr")
        .ok_or_else(|| anyhow!("targets.pr profile is required for ci:generate"))?;

    let mut jobs = String::new();
    jobs.push_str("  prep:\n");
    jobs.push_str("    runs-on: ubuntu-latest\n");
    jobs.push_str("    steps:\n");
    jobs.push('\n');

    jobs.push_str("  build:\n");
    jobs.push_str("    runs-on: ubuntu-latest\n");
    jobs.push_str("    needs: [prep]\n");
    jobs.push_str("    steps:\n");
    jobs.push_str("      - uses: actions/checkout@v4\n");
    jobs.push_str("      - run: dwf build:debug\n");
    jobs.push('\n');

    for cmd in pr {
        let id = format!("check_{}", sanitize_job_name(cmd));
        jobs.push_str(&format!("  {}:\n", id));
        jobs.push_str("    runs-on: ubuntu-latest\n");
        jobs.push_str("    needs: [prep, build]\n");
        jobs.push_str("    steps:\n");
        jobs.push_str("      - uses: actions/checkout@v4\n");
        jobs.push_str(&format!("      - run: dwf {}\n", cmd));
        jobs.push('\n');
    }

    Ok(format!(
        "name: ci\n\non: [pull_request, push]\n\njobs:\n{}# project: {}\n",
        jobs, cfg.project.name
    ))
}

pub fn check_workflow(cfg: &DevflowConfig, workflow: &str) -> Result<()> {
    let pr = cfg
        .targets
        .profiles
        .get("pr")
        .ok_or_else(|| anyhow!("targets.pr profile is required for ci:check"))?;

    let mut issues = Vec::new();

    if !workflow.contains("jobs:\n  prep:") {
        issues.push("missing required 'prep' job".to_string());
    }
    if !workflow.contains("\n  build:") {
        issues.push("missing required 'build' job".to_string());
    }
    if !workflow.contains("needs: [prep]") {
        issues.push("build job should depend on prep".to_string());
    }

    for cmd in pr {
        let id = format!("check_{}", sanitize_job_name(cmd));
        if !workflow.contains(&format!("\n  {}:", id)) {
            issues.push(format!(
                "missing check job for targets.pr command '{}'",
                cmd
            ));
        }
        if !workflow.contains(&format!("run: dwf {}", cmd)) {
            issues.push(format!("missing command invocation 'dwf {}'", cmd));
        }
    }

    if issues.is_empty() {
        return Ok(());
    }

    Err(anyhow!(
        "ci workflow check failed:\n- {}",
        issues.join("\n- ")
    ))
}

fn sanitize_job_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch.to_ascii_lowercase(),
            _ => '_',
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> DevflowConfig {
        toml::from_str(
            r#"
            [project]
            name = "demo"
            stack = ["rust"]

            [targets]
            pr = ["fmt:check", "lint:static", "test:unit"]
            "#,
        )
        .expect("fixture config should parse")
    }

    #[test]
    fn renders_prep_build_and_profile_jobs() {
        // Verifies that the rendered GitHub workflow contains the necessary
        // boilerplate jobs (prep, build) and specific check jobs from targets.pr.
        let cfg = fixture();
        let out = render_workflow(&cfg).expect("render should pass");
        assert!(out.contains("  prep:"));
        assert!(out.contains("  build:"));
        assert!(out.contains("  check_fmt_check:"));
        assert!(out.contains("  check_lint_static:"));
        assert!(out.contains("  check_test_unit:"));
    }

    #[test]
    fn check_passes_for_rendered_output() {
        // Ensures that a workflow rendered by Devflow passes its own internal validation.
        let cfg = fixture();
        let out = render_workflow(&cfg).expect("render should pass");
        check_workflow(&cfg, &out).expect("rendered output should validate");
    }

    #[test]
    fn check_fails_when_required_job_missing() {
        // Ensures that the workflow validator correctly identifies missing required jobs.
        let cfg = fixture();
        let broken = "name: ci\n\njobs:\n  prep:\n";
        let err = check_workflow(&cfg, broken).expect_err("must fail");
        assert!(err.to_string().contains("missing required 'build' job"));
    }
}
