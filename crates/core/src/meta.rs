// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Meta.hs.

/// Haskell `getVersion`.
pub fn get_version() -> String {
    format!("Haskell Dockerfile Linter {}", get_short_version())
}

/// Haskell `getShortVersion` (the oracle is hadolint v2.15.1).
pub fn get_short_version() -> String {
    "2.15.1".to_string()
}
