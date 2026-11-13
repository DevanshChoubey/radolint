// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3055.hs.

use crate::rule::{no_failures, Rule};
use crate::rule::LabelSchema;

/// Haskell `rule`.
// TODO(port): upstream/hadolint/src/Hadolint/Rule/DL3055.hs
pub fn rule(label_schema: &LabelSchema) -> Box<dyn Rule> {
    no_failures()
}
