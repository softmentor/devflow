//! Deterministic fingerprinting for environment isolation.
//!
//! This module provides the core logic for computing SHA256 hashes of project
//! manifests and toolchain configurations, ensuring that containerized environments
//! are perfectly reproducible across different machines and CI runs.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;
use tracing::debug;

/// Computes a deterministic SHA256 fingerprint from a list of files.
///
/// This fingerprint defines the exact runtime identity of the container cache,
/// allowing identical local and CI runs to safely reuse the exact same image base.
pub fn compute_fingerprint(base_dir: &Path, inputs: &[String]) -> Result<String> {
    let mut hasher = Sha256::new();

    // Sort inputs alphabetically so that hash isn't order-dependent based on the Extension order
    let mut sorted_inputs = inputs.to_owned();
    sorted_inputs.sort();

    for input in sorted_inputs {
        let path = base_dir.join(&input);

        // We do not strict-fail if an optional file is missing (e.g., node_modules might not exist yet)
        // But we record its absence in the hash.
        hasher.update(input.as_bytes());
        hasher.update(b"\0");

        if path.is_file() {
            let content = std::fs::read(&path)
                .with_context(|| format!("failed to read fingerprint input: {}", path.display()))?;

            // Hash the content identity
            hasher.update(&content);
            debug!("fingerprint: mixed {} ({} bytes)", input, content.len());
        } else {
            // Include an explicit marker for missing to prevent overlap collisions
            hasher.update(b"missing\0");
            debug!("fingerprint: input {} is absent", input);
        }
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_deterministic_hash() {
        let dir = tempfile::tempdir().unwrap();
        let file1 = dir.path().join("dependency.lock");
        std::fs::write(&file1, b"lock-content").unwrap();

        let inputs = vec!["dependency.lock".to_string(), "missing.toml".to_string()];

        let hash1 = compute_fingerprint(dir.path(), &inputs).unwrap();
        let hash2 = compute_fingerprint(dir.path(), &inputs).unwrap();

        // Must be deterministic
        assert_eq!(hash1, hash2);

        // Content modifications must produce totally different hashes
        std::fs::write(&file1, b"lock-content-v2").unwrap();
        let mutated_hash = compute_fingerprint(dir.path(), &inputs).unwrap();
        assert_ne!(hash1, mutated_hash);
    }

    #[test]
    fn input_order_independence() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.lock"), b"content-a").unwrap();
        std::fs::write(dir.path().join("b.toml"), b"content-b").unwrap();

        let inputs_ab = vec!["a.lock".to_string(), "b.toml".to_string()];
        let inputs_ba = vec!["b.toml".to_string(), "a.lock".to_string()];

        let hash_ab = compute_fingerprint(dir.path(), &inputs_ab).unwrap();
        let hash_ba = compute_fingerprint(dir.path(), &inputs_ba).unwrap();

        assert_eq!(hash_ab, hash_ba, "fingerprint must be order-independent");
    }

    #[test]
    fn empty_inputs_produces_valid_hash() {
        let dir = tempfile::tempdir().unwrap();
        let hash = compute_fingerprint(dir.path(), &[]).unwrap();
        // SHA256 hex string is always 64 chars
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn hash_is_valid_hex_sha256() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.lock"), b"content").unwrap();
        let hash = compute_fingerprint(dir.path(), &["test.lock".to_string()]).unwrap();
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
