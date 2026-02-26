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
        "ci:render",
        "ci:check",
    ]
}
