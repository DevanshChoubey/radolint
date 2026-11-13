// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL4004.hs.

use crate::rule::{no_failures, Rule};

/// Haskell `rule`.
// TODO(port): upstream/hadolint/src/Hadolint/Rule/DL4004.hs
pub fn rule() -> Box<dyn Rule> {
    no_failures()
}
