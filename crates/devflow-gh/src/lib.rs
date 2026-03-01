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

    let template = include_str!("../resources/ci-template.yml");

    // Map commands to background execution and capture PIDs.
    // Then wait for each PID and accumulate exit codes.
    let mut script = String::new();
    script.push_str("pids=(); ");

    for cmd in pr {
        let context = cmd.replace(':', "-");
        script.push_str(&format!("dwf --report {} {} & pids+=($!); ", context, cmd));
    }

    script.push_str(
        "exit_code=0; for pid in ${pids[@]}; do wait $pid || exit_code=$?; done; exit $exit_code",
    );

    let rendered = template
        .replace("{{COMMANDS}}", &script)
        .replace("{{PROJECT_NAME}}", &cfg.project.name);

    Ok(rendered)
}

pub fn check_workflow(cfg: &DevflowConfig, workflow: &str) -> Result<()> {
    let pr = cfg
        .targets
        .profiles
        .get("pr")
        .ok_or_else(|| anyhow!("targets.pr profile is required for ci:check"))?;

    let mut issues = Vec::new();

    if !workflow.contains("  prep:") {
        issues.push("missing required 'prep' job".to_string());
    }
    if !workflow.contains("  build:") {
        issues.push("missing required 'build' job".to_string());
    }
    if !workflow.contains("needs: [prep]") {
        issues.push("build job should depend on prep".to_string());
    }

    if !workflow.contains("  verify:") && !workflow.contains("Verify") {
        issues.push("missing required 'verify' job".to_string());
    }

    for _cmd in pr {
        if !workflow.contains("dwf --report") {
            issues.push("missing command invocation 'dwf --report'".to_string());
        }
    }

    if !workflow.contains(" wait") {
        issues.push("missing 'wait' command for parallel checks".to_string());
    }

    if issues.is_empty() {
        return Ok(());
    }

    Err(anyhow!(
        "ci workflow check failed:\n- {}",
        issues.join("\n- ")
    ))
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
        // boilerplate jobs (prep, build) and specific check commands from targets.pr.
        let cfg = fixture();
        let out = render_workflow(&cfg).expect("render should pass");
        assert!(out.contains("  prep:"));
        assert!(out.contains("  build:"));
        assert!(out.contains("Verify"));
        assert!(out.contains("dwf --report fmt-check fmt:check &"));
        assert!(out.contains("dwf --report lint-static lint:static &"));
        assert!(out.contains("dwf --report test-unit test:unit &"));
        assert!(out.contains("wait"));
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
