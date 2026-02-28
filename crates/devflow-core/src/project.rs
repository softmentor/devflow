use crate::constants::*;
use std::path::Path;

/// Determines if a given stack type is applicable to the project at `base_path`.
pub fn stack_is_applicable(base_path: &Path, stack: &str) -> bool {
    match stack {
        "rust" => base_path.join(MANIFEST_RUST).exists(),
        "node" => base_path.join(MANIFEST_NODE).exists(),
        "custom" => {
            base_path.join(TARGET_CUSTOM_JUST).exists()
                || base_path.join(TARGET_CUSTOM_MAKE).exists()
        }
        // Subprocess extensions always apply initially; execution will fail if bad mapping
        _ => true,
    }
}
