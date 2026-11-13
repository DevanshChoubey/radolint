// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Config/Environment.hs.

use crate::config::configuration::PartialConfiguration;
use crate::formatter::format::OutputFormat;
use crate::rule::{DLSeverity, LabelName, LabelSchema, LabelType, RuleCode};
use dlint_syntax::Registry;
use std::collections::BTreeSet;

/// Haskell `getConfigFromEnvironment`.
pub fn get_config_from_environment() -> PartialConfiguration {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs getConfigFromEnvironment
    PartialConfiguration::mempty()
}

/// Haskell `isSet`.
pub fn is_set(name: &str) -> Option<bool> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs isSet
    let _ = name;
    None
}
/// Haskell `maybeTruthy`.
pub fn maybe_truthy(name: &str) -> Option<bool> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs maybeTruthy
    let _ = name;
    None
}
/// Haskell `truthy`.
pub fn truthy(s: &str) -> bool {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs truthy
    let _ = s;
    false
}
/// Haskell `getFormat`.
pub fn get_format() -> Vec<OutputFormat> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs getFormat
    vec![]
}
/// Haskell `getOverrideList`.
pub fn get_override_list(env: &str) -> Vec<RuleCode> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs getOverrideList
    let _ = env;
    vec![]
}
/// Haskell `getRulecodes`.
pub fn get_rulecodes(s: &str) -> Vec<RuleCode> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs getRulecodes
    let _ = s;
    vec![]
}
/// Haskell `getAllowedSet`.
pub fn get_allowed_set(env: &str) -> BTreeSet<Registry> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs getAllowedSet
    let _ = env;
    BTreeSet::new()
}
/// Haskell `getAllowed`.
pub fn get_allowed(s: &str) -> Vec<Registry> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs getAllowed
    let _ = s;
    vec![]
}
/// Haskell `getLabelSchema`.
pub fn get_label_schema(env: &str) -> LabelSchema {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs getLabelSchema
    let _ = env;
    LabelSchema::new()
}
/// Haskell `labelSchemaFromText`.
pub fn label_schema_from_text(txt: &str) -> Vec<(LabelName, LabelType)> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs labelSchemaFromText
    let _ = txt;
    vec![]
}
/// Haskell `convertToPairs`.
pub fn convert_to_pairs(txt: &str) -> Vec<(String, String)> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs convertToPairs
    let _ = txt;
    vec![]
}
/// Haskell `convertToLabelSchema`.
pub fn convert_to_label_schema(pair: &(String, String)) -> Result<(LabelName, LabelType), String> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs convertToLabelSchema
    let _ = pair;
    Err(String::new())
}
/// Haskell `getFailureThreshold`.
pub fn get_failure_threshold() -> Option<DLSeverity> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Environment.hs getFailureThreshold
    None
}
