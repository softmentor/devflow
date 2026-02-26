use std::str::FromStr;

use anyhow::{anyhow, Result};

use devflow_core::{CommandRef, DevflowConfig};

pub fn resolve_policy_commands(cfg: &DevflowConfig, selector: &str) -> Result<Vec<CommandRef>> {
    let entries = match selector {
        "pr" => &cfg.targets.pr,
        "main" => &cfg.targets.main,
        "release" => &cfg.targets.release,
        _ => return Err(anyhow!("unknown check profile '{selector}'")),
    };

    entries
        .iter()
        .map(|item| CommandRef::from_str(item).map_err(|e| anyhow!(e)))
        .collect()
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
            pr = ["fmt:check", "test:unit"]
            main = ["fmt:check", "test:unit", "test:integration"]
            release = ["fmt:check", "test:unit", "package:artifact"]
            "#,
        )
        .expect("fixture config should parse")
    }

    #[test]
    fn resolves_pr_profile() {
        let cfg = fixture();
        let out = resolve_policy_commands(&cfg, "pr").expect("pr profile should resolve");
        let values = out.iter().map(|c| c.canonical()).collect::<Vec<_>>();
        assert_eq!(values, vec!["fmt:check", "test:unit"]);
    }
}
