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
        "package:artifact",
        "check",
        "release",
        "ci:generate",
        "ci:check",
    ]
}

pub fn build_command(primary: &str, selector: &str) -> Option<Vec<String>> {
    match (primary, selector) {
        ("setup", "deps") => Some(argv(&["npm", "ci"])),
        ("setup", "doctor") => Some(argv(&["npm", "--version"])),
        ("fmt", "check") => Some(argv(&["npm", "run", "fmt:check"])),
        ("fmt", "fix") => Some(argv(&["npm", "run", "fmt:fix"])),
        ("lint", "static") => Some(argv(&["npm", "run", "lint"])),
        ("build", "debug") => Some(argv(&["npm", "run", "build"])),
        ("build", "release") => Some(argv(&["npm", "run", "build"])),
        ("test", "unit") => Some(argv(&["npm", "run", "test:unit"])),
        ("test", "integration") => Some(argv(&["npm", "run", "test:integration"])),
        ("test", "smoke") => Some(argv(&["npm", "run", "test:smoke"])),
        ("package", "artifact") => Some(argv(&["npm", "pack", "--dry-run"])),
        _ => None,
    }
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}
