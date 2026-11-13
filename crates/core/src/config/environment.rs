// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Config/Environment.hs.

use crate::config::configuration::PartialConfiguration;
use crate::formatter::format::{read_maybe_output_format, OutputFormat};
use crate::rule::{
    read_either_label_type, read_maybe_severity, DLSeverity, LabelName, LabelSchema, LabelType,
    RuleCode,
};
use dlint_syntax::Registry;
use std::collections::BTreeSet;

/// Haskell `lookupEnv` (DIVERGENCE: non-UTF-8 values are converted lossily).
fn lookup_env(name: &str) -> Option<String> {
    std::env::var_os(name).map(|v| v.to_string_lossy().into_owned())
}

/// Haskell `getConfigFromEnvironment`.
pub fn get_config_from_environment() -> PartialConfiguration {
    PartialConfiguration {
        partial_no_fail: maybe_truthy("HADOLINT_NOFAIL"),
        partial_no_color: is_set("NO_COLOR"),
        partial_verbose: maybe_truthy("HADOLINT_VERBOSE"),
        partial_formats: get_format(),
        partial_error_rules: get_override_list("HADOLINT_OVERRIDE_ERROR"),
        partial_warning_rules: get_override_list("HADOLINT_OVERRIDE_WARNING"),
        partial_info_rules: get_override_list("HADOLINT_OVERRIDE_INFO"),
        partial_style_rules: get_override_list("HADOLINT_OVERRIDE_STYLE"),
        partial_ignore_rules: get_override_list("HADOLINT_IGNORE"),
        partial_allowed_registries: get_allowed_set("HADOLINT_TRUSTED_REGISTRIES"),
        partial_label_schema: get_label_schema("HADOLINT_REQUIRE_LABELS"),
        partial_strict_labels: maybe_truthy("HADOLINT_STRICT_LABELS"),
        partial_disable_ignore_pragma: maybe_truthy("HADOLINT_DISABLE_IGNORE_PRAGMA"),
        partial_failure_threshold: get_failure_threshold(),
    }
}

/// Haskell `isSet`.
pub fn is_set(name: &str) -> Option<bool> {
    match lookup_env(name) {
        Some(_) => Some(true),
        None => None,
    }
}
/// Haskell `maybeTruthy`.
pub fn maybe_truthy(name: &str) -> Option<bool> {
    match lookup_env(name) {
        Some(v) => {
            if truthy(&v) {
                Some(true)
            } else {
                Some(false)
            }
        }
        None => None,
    }
}
/// Haskell `truthy`.
pub fn truthy(s: &str) -> bool {
    // DIVERGENCE: Rust's to_lowercase is full Unicode; irrelevant for the ASCII comparands.
    let l = s.to_lowercase();
    ["1", "y", "on", "true", "yes"].contains(&l.as_str())
}
/// Haskell `getFormat`.
pub fn get_format() -> Vec<OutputFormat> {
    match lookup_env("HADOLINT_FORMAT") {
        None => vec![],
        Some(s) => s.split(',').filter_map(read_maybe_output_format).collect(),
    }
}
/// Haskell `getOverrideList`.
pub fn get_override_list(env: &str) -> Vec<RuleCode> {
    match lookup_env(env) {
        Some(s) => get_rulecodes(&s),
        None => vec![],
    }
}
/// Haskell `getRulecodes`.
pub fn get_rulecodes(s: &str) -> Vec<RuleCode> {
    s.split(',').map(RuleCode::from).collect()
}
/// Haskell `getAllowedSet`.
pub fn get_allowed_set(env: &str) -> BTreeSet<Registry> {
    match lookup_env(env) {
        Some(s) => get_allowed(&s).into_iter().collect(),
        None => BTreeSet::new(),
    }
}
/// Haskell `getAllowed`.
pub fn get_allowed(s: &str) -> Vec<Registry> {
    s.split(',').map(Registry::from).collect()
}
/// Haskell `getLabelSchema`.
pub fn get_label_schema(env: &str) -> LabelSchema {
    match lookup_env(env) {
        // Data.Map.fromList: later duplicates win, as with BTreeMap collect.
        Some(s) => label_schema_from_text(&s).into_iter().collect(),
        None => LabelSchema::new(),
    }
}
/// Haskell `labelSchemaFromText`.
pub fn label_schema_from_text(txt: &str) -> Vec<(LabelName, LabelType)> {
    convert_to_pairs(txt)
        .iter()
        .filter_map(|p| convert_to_label_schema(p).ok())
        .collect()
}
/// Haskell `convertToPairs`.
pub fn convert_to_pairs(txt: &str) -> Vec<(String, String)> {
    txt.split(',')
        .map(|part| match part.find(':') {
            Some(i) => (part[..i].to_string(), part[i..].to_string()),
            None => (part.to_string(), String::new()),
        })
        .collect()
}
/// Haskell `convertToLabelSchema`.
pub fn convert_to_label_schema(pair: &(String, String)) -> Result<(LabelName, LabelType), String> {
    let (tln, tlt) = pair;
    // Data.Text.drop 1
    let mut it = tlt.chars();
    it.next();
    match read_either_label_type(it.as_str()) {
        Ok(lt) => Ok((tln.clone(), lt)),
        Err(e) => Err(e),
    }
}
/// Haskell `getFailureThreshold`.
pub fn get_failure_threshold() -> Option<DLSeverity> {
    lookup_env("HADOLINT_FAILURE_THRESHOLD").and_then(|ft| read_maybe_severity(&ft))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    // SAFETY (set_var/remove_var below): all env access in these tests holds LOCK.

    // Environment is process-global; serialise the tests (hspec runs sequentially).
    static LOCK: Mutex<()> = Mutex::new(());

    const ALL: [&str; 14] = [
        "HADOLINT_NOFAIL",
        "NO_COLOR",
        "HADOLINT_VERBOSE",
        "HADOLINT_FORMAT",
        "HADOLINT_OVERRIDE_ERROR",
        "HADOLINT_OVERRIDE_WARNING",
        "HADOLINT_OVERRIDE_INFO",
        "HADOLINT_OVERRIDE_STYLE",
        "HADOLINT_IGNORE",
        "HADOLINT_TRUSTED_REGISTRIES",
        "HADOLINT_REQUIRE_LABELS",
        "HADOLINT_STRICT_LABELS",
        "HADOLINT_FAILURE_THRESHOLD",
        "HADOLINT_DISABLE_IGNORE_PRAGMA",
    ];

    fn unset_all() {
        for n in ALL {
            unsafe { std::env::remove_var(n) };
        }
    }

    fn without_all<F: FnOnce(PartialConfiguration)>(f: F) {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unset_all();
        f(get_config_from_environment());
    }

    fn with_just_env<F: FnOnce(PartialConfiguration)>(name: &str, value: &str, f: F) {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unset_all();
        unsafe { std::env::set_var(name, value) };
        let conf = get_config_from_environment();
        unsafe { std::env::remove_var(name) };
        f(conf);
    }

    fn rc(v: &[&str]) -> Vec<RuleCode> {
        v.iter().map(|s| RuleCode::from(*s)).collect()
    }

    #[test]
    fn empty_config_when_nothing_is_set() {
        without_all(|c| assert_eq!(c, PartialConfiguration::mempty()));
    }

    #[test]
    fn parse_hadolint_nofail_1() {
        with_just_env("HADOLINT_NOFAIL", "1", |c| {
            assert_eq!(c, PartialConfiguration { partial_no_fail: Some(true), ..Default::default() })
        });
    }

    #[test]
    fn parse_hadolint_nofail_0() {
        with_just_env("HADOLINT_NOFAIL", "0", |c| {
            assert_eq!(c, PartialConfiguration { partial_no_fail: Some(false), ..Default::default() })
        });
    }

    #[test]
    fn parse_no_color_y() {
        with_just_env("NO_COLOR", "y", |c| {
            assert_eq!(c, PartialConfiguration { partial_no_color: Some(true), ..Default::default() })
        });
    }

    #[test]
    fn parse_no_color_n() {
        with_just_env("NO_COLOR", "n", |c| {
            assert_eq!(c, PartialConfiguration { partial_no_color: Some(true), ..Default::default() })
        });
    }

    #[test]
    fn parse_no_color_set_to_empty_string() {
        // POSIX: set-but-empty counts as set (expectedValueVarEmptyString = Just True).
        with_just_env("NO_COLOR", "", |c| {
            assert_eq!(c, PartialConfiguration { partial_no_color: Some(true), ..Default::default() })
        });
    }

    #[test]
    fn parse_no_color_unset() {
        without_all(|c| {
            assert_eq!(c, PartialConfiguration { partial_no_color: None, ..Default::default() })
        });
    }

    #[test]
    fn parse_hadolint_verbose_on() {
        with_just_env("HADOLINT_VERBOSE", "on", |c| {
            assert_eq!(c, PartialConfiguration { partial_verbose: Some(true), ..Default::default() })
        });
    }

    #[test]
    fn parse_hadolint_verbose_off() {
        with_just_env("HADOLINT_VERBOSE", "off", |c| {
            assert_eq!(c, PartialConfiguration { partial_verbose: Some(false), ..Default::default() })
        });
    }

    fn fmt_case(v: &str, want: Vec<OutputFormat>) {
        with_just_env("HADOLINT_FORMAT", v, |c| {
            assert_eq!(c, PartialConfiguration { partial_formats: want, ..Default::default() })
        });
    }

    #[test]
    fn parse_hadolint_format_json() {
        fmt_case("json", vec![OutputFormat::Json]);
    }

    #[test]
    fn parse_hadolint_format_sarif() {
        fmt_case("sarif", vec![OutputFormat::Sarif]);
    }

    #[test]
    fn parse_hadolint_format_sarif_json() {
        fmt_case("sarif,json", vec![OutputFormat::Sarif, OutputFormat::Json]);
    }

    #[test]
    fn parse_hadolint_format_junk_json() {
        fmt_case("junk,json", vec![OutputFormat::Json]);
    }

    #[test]
    fn parse_override_error_one() {
        with_just_env("HADOLINT_OVERRIDE_ERROR", "DL3010", |c| {
            assert_eq!(c.partial_error_rules, rc(&["DL3010"]));
        });
    }

    #[test]
    fn parse_override_error_two() {
        with_just_env("HADOLINT_OVERRIDE_ERROR", "DL3010,DL3011", |c| {
            assert_eq!(c.partial_error_rules, rc(&["DL3010", "DL3011"]));
        });
    }

    #[test]
    fn parse_override_warning_one() {
        with_just_env("HADOLINT_OVERRIDE_WARNING", "DL3010", |c| {
            assert_eq!(c.partial_warning_rules, rc(&["DL3010"]));
        });
    }

    #[test]
    fn parse_override_warning_two() {
        with_just_env("HADOLINT_OVERRIDE_WARNING", "DL3010,DL3011", |c| {
            assert_eq!(c.partial_warning_rules, rc(&["DL3010", "DL3011"]));
        });
    }

    #[test]
    fn parse_override_info_one() {
        with_just_env("HADOLINT_OVERRIDE_INFO", "DL3010", |c| {
            assert_eq!(c.partial_info_rules, rc(&["DL3010"]));
        });
    }

    #[test]
    fn parse_override_info_two() {
        with_just_env("HADOLINT_OVERRIDE_INFO", "DL3010,DL3011", |c| {
            assert_eq!(c.partial_info_rules, rc(&["DL3010", "DL3011"]));
        });
    }

    #[test]
    fn parse_override_style_one() {
        with_just_env("HADOLINT_OVERRIDE_STYLE", "DL3010", |c| {
            assert_eq!(c.partial_style_rules, rc(&["DL3010"]));
        });
    }

    #[test]
    fn parse_override_style_two() {
        with_just_env("HADOLINT_OVERRIDE_STYLE", "DL3010,DL3011", |c| {
            assert_eq!(c.partial_style_rules, rc(&["DL3010", "DL3011"]));
        });
    }

    #[test]
    fn parse_ignore_one() {
        with_just_env("HADOLINT_IGNORE", "DL3010", |c| {
            assert_eq!(c.partial_ignore_rules, rc(&["DL3010"]));
        });
    }

    #[test]
    fn parse_ignore_two() {
        with_just_env("HADOLINT_IGNORE", "DL3010,DL3011", |c| {
            assert_eq!(c.partial_ignore_rules, rc(&["DL3010", "DL3011"]));
        });
    }

    #[test]
    fn parse_trusted_registries_one() {
        with_just_env("HADOLINT_TRUSTED_REGISTRIES", "foobar.com", |c| {
            let want: BTreeSet<Registry> = [Registry::from("foobar.com")].into_iter().collect();
            assert_eq!(c.partial_allowed_registries, want);
        });
    }

    #[test]
    fn parse_trusted_registries_two() {
        with_just_env("HADOLINT_TRUSTED_REGISTRIES", "foobar.com,barfoo.com", |c| {
            let want: BTreeSet<Registry> =
                [Registry::from("foobar.com"), Registry::from("barfoo.com")].into_iter().collect();
            assert_eq!(c.partial_allowed_registries, want);
        });
    }

    #[test]
    fn parse_require_labels_one() {
        with_just_env("HADOLINT_REQUIRE_LABELS", "foo:email", |c| {
            let want: LabelSchema = [("foo".to_string(), LabelType::Email)].into_iter().collect();
            assert_eq!(c.partial_label_schema, want);
        });
    }

    #[test]
    fn parse_require_labels_two() {
        with_just_env("HADOLINT_REQUIRE_LABELS", "foo:email,bar:text", |c| {
            let want: LabelSchema =
                [("foo".to_string(), LabelType::Email), ("bar".to_string(), LabelType::RawText)]
                    .into_iter()
                    .collect();
            assert_eq!(c.partial_label_schema, want);
        });
    }

    #[test]
    fn parse_strict_labels_yes() {
        with_just_env("HADOLINT_STRICT_LABELS", "yes", |c| {
            assert_eq!(c.partial_strict_labels, Some(true));
        });
    }

    #[test]
    fn parse_strict_labels_false() {
        with_just_env("HADOLINT_STRICT_LABELS", "false", |c| {
            assert_eq!(c.partial_strict_labels, Some(false));
        });
    }

    #[test]
    fn parse_disable_ignore_pragma_yes() {
        with_just_env("HADOLINT_DISABLE_IGNORE_PRAGMA", "yes", |c| {
            assert_eq!(c.partial_disable_ignore_pragma, Some(true));
        });
    }

    #[test]
    fn parse_disable_ignore_pragma_false() {
        with_just_env("HADOLINT_DISABLE_IGNORE_PRAGMA", "false", |c| {
            assert_eq!(c.partial_disable_ignore_pragma, Some(false));
        });
    }

    #[test]
    fn parse_failure_threshold_error() {
        with_just_env("HADOLINT_FAILURE_THRESHOLD", "error", |c| {
            assert_eq!(c.partial_failure_threshold, Some(DLSeverity::DLErrorC));
        });
    }

    #[test]
    fn parse_failure_threshold_style() {
        with_just_env("HADOLINT_FAILURE_THRESHOLD", "style", |c| {
            assert_eq!(c.partial_failure_threshold, Some(DLSeverity::DLStyleC));
        });
    }
}
