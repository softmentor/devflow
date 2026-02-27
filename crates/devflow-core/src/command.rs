use std::fmt::{Display, Formatter};
use std::str::FromStr;

use thiserror::Error;

/// The primary categories of commands supported by Devflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimaryCommand {
    /// Initialize a new Devflow project.
    Init,
    /// Setup the environment for the project.
    Setup,
    /// Format source code.
    Fmt,
    /// Run static analysis and linting.
    Lint,
    /// Build the project.
    Build,
    /// Run tests.
    Test,
    /// Package the project for distribution.
    Package,
    /// Run all necessary checks (e.g., for a PR).
    Check,
    /// Perform a release.
    Release,
    /// CI-related operations (e.g., configuration generation).
    Ci,
}

impl PrimaryCommand {
    /// Returns the string representation of the primary command.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Setup => "setup",
            Self::Fmt => "fmt",
            Self::Lint => "lint",
            Self::Build => "build",
            Self::Test => "test",
            Self::Package => "package",
            Self::Check => "check",
            Self::Release => "release",
            Self::Ci => "ci",
        }
    }
}

/// A reference to a Devflow command, including its primary type and an optional selector.
///
/// Example: `test:unit` -> primary: `Test`, selector: `Some("unit")`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandRef {
    /// The primary command category.
    pub primary: PrimaryCommand,
    /// An optional sub-command or target selector.
    pub selector: Option<String>,
}

impl CommandRef {
    /// Returns the canonical string representation of the command (e.g., `primary:selector`).
    pub fn canonical(&self) -> String {
        match &self.selector {
            Some(selector) => format!("{}:{}", self.primary.as_str(), selector),
            None => self.primary.as_str().to_string(),
        }
    }
}

impl Display for CommandRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.canonical())
    }
}

#[derive(Debug, Error)]
pub enum CommandParseError {
    #[error("unknown primary command '{0}'")]
    UnknownPrimary(String),
}

impl FromStr for CommandRef {
    type Err = CommandParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.splitn(2, ':');
        let primary_text = parts.next().unwrap_or_default();
        let selector = parts.next().map(ToOwned::to_owned);

        let primary = match primary_text {
            "init" => PrimaryCommand::Init,
            "setup" => PrimaryCommand::Setup,
            "fmt" => PrimaryCommand::Fmt,
            "lint" => PrimaryCommand::Lint,
            "build" => PrimaryCommand::Build,
            "test" => PrimaryCommand::Test,
            "package" => PrimaryCommand::Package,
            "check" => PrimaryCommand::Check,
            "release" => PrimaryCommand::Release,
            "ci" => PrimaryCommand::Ci,
            _ => return Err(CommandParseError::UnknownPrimary(primary_text.to_string())),
        };

        Ok(Self { primary, selector })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_primary_only_command() {
        // Verifies that a simple command string like "check" is correctly parsed
        // as a PrimaryCommand with no selector.
        let cmd = CommandRef::from_str("check").expect("check should parse");
        assert_eq!(cmd.primary, PrimaryCommand::Check);
        assert_eq!(cmd.selector, None);
    }

    #[test]
    fn parses_selector_command() {
        // Verifies that a colon-separated command like "test:unit" is correctly split
        // into a PrimaryCommand and a selector.
        let cmd = CommandRef::from_str("test:unit").expect("test:unit should parse");
        assert_eq!(cmd.primary, PrimaryCommand::Test);
        assert_eq!(cmd.selector.as_deref(), Some("unit"));
    }

    #[test]
    fn rejects_unknown_primary() {
        // Ensures that an invalid primary command results in a parsing error.
        let err = CommandRef::from_str("unknown:foo").expect_err("must fail");
        assert!(matches!(err, CommandParseError::UnknownPrimary(_)));
    }
}
