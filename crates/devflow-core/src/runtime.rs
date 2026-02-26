use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeProfile {
    Container,
    Host,
    Auto,
}

impl Default for RuntimeProfile {
    fn default() -> Self {
        Self::Auto
    }
}
