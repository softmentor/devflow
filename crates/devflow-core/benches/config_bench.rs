use criterion::{black_box, criterion_group, criterion_main, Criterion};
use devflow_core::DevflowConfig;

fn bench_config_parse(c: &mut Criterion) {
    let toml_text = r#"
[project]
name = "bench-demo"
stack = ["rust", "node"]

[runtime]
profile = "auto"

[targets]
pr = ["fmt:check", "lint:static", "build:debug", "test:unit"]
main = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration"]

[extensions.rust]
source = "builtin"
required = true
"#;

    c.bench_function("parse_config", |b| {
        b.iter(|| {
            let _cfg: DevflowConfig = toml::from_str(black_box(toml_text)).unwrap();
        })
    });
}

criterion_group!(benches, bench_config_parse);
criterion_main!(benches);
