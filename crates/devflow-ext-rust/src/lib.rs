pub fn default_capabilities() -> &'static [&'static str] {
    &[
        "setup",
        "fmt:check",
        "fmt:fix",
        "lint:static",
        "build:debug",
        "build:release",
        "test:unit",
        "test:integration",
        "test:smoke",
        "package:artifact",
        "check",
        "release",
        "ci:generate",
        "ci:check",
    ]
}

pub fn build_command(primary: &str, selector: &str) -> Option<Vec<String>> {
    match (primary, selector) {
        ("setup", "toolchain") => Some(argv(&["rustup", "show"])),
        ("setup", "deps") => Some(argv(&["cargo", "fetch"])),
        ("setup", "doctor") => Some(argv(&["cargo", "--version"])),
        ("fmt", "check") => Some(argv(&["cargo", "fmt", "--all", "--", "--check"])),
        ("fmt", "fix") => Some(argv(&["cargo", "fmt", "--all"])),
        ("lint", "static") => Some(argv(&[
            "cargo",
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])),
        ("build", "debug") => Some(argv(&["cargo", "build"])),
        ("build", "release") => Some(argv(&["cargo", "build", "--release"])),
        ("test", "unit") => Some(argv(&["cargo", "test", "--lib", "--bins"])),
        ("test", "integration") => Some(argv(&["cargo", "test", "--tests"])),
        ("test", "smoke") => Some(argv(&["cargo", "test", "smoke"])),
        ("package", "artifact") => Some(argv(&["cargo", "build", "--release"])),
        ("release", "candidate") => Some(argv(&["cargo", "build", "--release"])),
        _ => None,
    }
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}
