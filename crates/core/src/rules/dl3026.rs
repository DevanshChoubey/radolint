// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3026.hs.

use crate::rule::{no_failures, Rule};
use dlint_syntax::Registry;
use std::collections::BTreeSet;

/// Haskell `rule`.
// TODO(port): upstream/hadolint/src/Hadolint/Rule/DL3026.hs
pub fn rule(allowed: &BTreeSet<Registry>) -> Box<dyn Rule> {
    no_failures()
}
