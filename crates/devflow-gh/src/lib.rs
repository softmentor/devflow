use anyhow::Result;

use devflow_core::DevflowConfig;

pub fn render_workflow_stub(cfg: &DevflowConfig) -> Result<String> {
    let out = format!(
        "name: ci\n\non: [pull_request, push]\n\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: dwf check:pr\n\n# project: {}\n",
        cfg.project.name
    );
    Ok(out)
}
