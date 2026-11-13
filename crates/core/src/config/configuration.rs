// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Config/Configuration.hs.

use crate::formatter::format::OutputFormat;
use crate::rule::{DLSeverity, LabelSchema, RuleCode};
use dlint_syntax::Registry;
use std::collections::BTreeSet;

/// Haskell `Configuration`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    pub no_fail: bool,
    pub no_color: bool,
    pub verbose: bool,
    pub formats: Vec<OutputFormat>,
    pub error_rules: Vec<RuleCode>,
    pub warning_rules: Vec<RuleCode>,
    pub info_rules: Vec<RuleCode>,
    pub style_rules: Vec<RuleCode>,
    pub ignore_rules: Vec<RuleCode>,
    pub allowed_registries: BTreeSet<Registry>,
    pub label_schema: LabelSchema,
    pub strict_labels: bool,
    pub disable_ignore_pragma: bool,
    pub failure_threshold: DLSeverity,
}

impl Default for Configuration {
    /// `instance Default Configuration`.
    fn default() -> Self {
        Configuration {
            no_fail: false,
            no_color: false,
            verbose: false,
            formats: vec![],
            error_rules: vec![],
            warning_rules: vec![],
            info_rules: vec![],
            style_rules: vec![],
            ignore_rules: vec![],
            allowed_registries: BTreeSet::new(),
            label_schema: LabelSchema::new(),
            strict_labels: false,
            disable_ignore_pragma: false,
            failure_threshold: DLSeverity::default(),
        }
    }
}

/// Haskell `PartialConfiguration`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartialConfiguration {
    pub partial_no_fail: Option<bool>,
    pub partial_no_color: Option<bool>,
    pub partial_verbose: Option<bool>,
    pub partial_formats: Vec<OutputFormat>,
    pub partial_error_rules: Vec<RuleCode>,
    pub partial_warning_rules: Vec<RuleCode>,
    pub partial_info_rules: Vec<RuleCode>,
    pub partial_style_rules: Vec<RuleCode>,
    pub partial_ignore_rules: Vec<RuleCode>,
    pub partial_allowed_registries: BTreeSet<Registry>,
    pub partial_label_schema: LabelSchema,
    pub partial_strict_labels: Option<bool>,
    pub partial_disable_ignore_pragma: Option<bool>,
    pub partial_failure_threshold: Option<DLSeverity>,
}

impl PartialConfiguration {
    /// `instance Monoid PartialConfiguration` `mempty`.
    pub fn mempty() -> Self {
        Self::default()
    }

    /// `instance Semigroup PartialConfiguration` (`a <> b`): later `Maybe`s win (`b <|> a`),
    /// lists/sets/maps are appended/unioned. Note: `partialFailureThreshold` uses `a <> b`
    /// on `Maybe DLSeverity`, i.e. the Semigroup of `Maybe` over `DLSeverity` whose `<>`
    /// keeps the right-hand severity when both are present.
    pub fn append(mut self, b: PartialConfiguration) -> PartialConfiguration {
        self.partial_no_fail = b.partial_no_fail.or(self.partial_no_fail);
        self.partial_no_color = b.partial_no_color.or(self.partial_no_color);
        self.partial_verbose = b.partial_verbose.or(self.partial_verbose);
        self.partial_formats.extend(b.partial_formats);
        self.partial_error_rules.extend(b.partial_error_rules);
        self.partial_warning_rules.extend(b.partial_warning_rules);
        self.partial_info_rules.extend(b.partial_info_rules);
        self.partial_style_rules.extend(b.partial_style_rules);
        self.partial_ignore_rules.extend(b.partial_ignore_rules);
        self.partial_allowed_registries.extend(b.partial_allowed_registries);
        // Data.Map union is left-biased: keys already present in `self` win.
        for (k, v) in b.partial_label_schema {
            self.partial_label_schema.entry(k).or_insert(v);
        }
        self.partial_strict_labels = b.partial_strict_labels.or(self.partial_strict_labels);
        self.partial_disable_ignore_pragma =
            b.partial_disable_ignore_pragma.or(self.partial_disable_ignore_pragma);
        self.partial_failure_threshold = match (self.partial_failure_threshold, b.partial_failure_threshold) {
            (Some(a), Some(b)) => Some(a.append(b)),
            (Some(a), None) => Some(a),
            (None, b) => b,
        };
        self
    }
}

/// Haskell `applyPartialConfiguration`.
pub fn apply_partial_configuration(config: Configuration, partial: PartialConfiguration) -> Configuration {
    let mut formats = config.formats;
    formats.extend(partial.partial_formats);
    let mut error_rules = config.error_rules;
    error_rules.extend(partial.partial_error_rules);
    let mut warning_rules = config.warning_rules;
    warning_rules.extend(partial.partial_warning_rules);
    let mut info_rules = config.info_rules;
    info_rules.extend(partial.partial_info_rules);
    let mut style_rules = config.style_rules;
    style_rules.extend(partial.partial_style_rules);
    let mut ignore_rules = config.ignore_rules;
    ignore_rules.extend(partial.partial_ignore_rules);
    let mut allowed_registries = config.allowed_registries;
    allowed_registries.extend(partial.partial_allowed_registries);
    let mut label_schema = config.label_schema;
    for (k, v) in partial.partial_label_schema {
        label_schema.entry(k).or_insert(v); // Map union is left-biased
    }
    Configuration {
        no_fail: partial.partial_no_fail.unwrap_or(config.no_fail),
        no_color: partial.partial_no_color.unwrap_or(config.no_color),
        verbose: partial.partial_verbose.unwrap_or(config.verbose),
        formats,
        error_rules,
        warning_rules,
        info_rules,
        style_rules,
        ignore_rules,
        allowed_registries,
        label_schema,
        strict_labels: partial.partial_strict_labels.unwrap_or(config.strict_labels),
        disable_ignore_pragma: partial.partial_disable_ignore_pragma.unwrap_or(config.disable_ignore_pragma),
        failure_threshold: partial.partial_failure_threshold.unwrap_or(config.failure_threshold),
    }
}

impl Configuration {
    /// `instance Pretty Configuration` (printed to stderr with `--verbose`).
    pub fn pretty(&self) -> String {
        // DIVERGENCE: prettyprinter's Doc is rendered directly to text; `nest 2` is applied by
        // indenting every line after the first by the nesting amount.
        let c = self;
        let b = |v: bool| if v { "True" } else { "False" };
        let formats = nest(
            2,
            &format!("output format:\n{}", pretty_print_list(|f| f.pretty().to_string(), &c.formats)),
        );
        let body = vec![
            "Configuration:".to_string(),
            format!("no fail: {}", b(c.no_fail)),
            format!("no color: {}", b(c.no_color)),
            formats,
            format!("failure threshold: {}", c.failure_threshold.pretty()),
            pretty_print_rulelist("error", &c.error_rules),
            pretty_print_rulelist("warning", &c.warning_rules),
            pretty_print_rulelist("info", &c.info_rules),
            pretty_print_rulelist("style", &c.style_rules),
            pretty_print_rulelist("ignore", &c.ignore_rules),
            format!("strict labels: {}", b(c.strict_labels)),
            pretty_print_label_schema(&c.label_schema),
            format!("disable ignore pragma: {}", b(c.disable_ignore_pragma)),
            pretty_print_registries(&c.allowed_registries),
        ];
        nest(2, &body.join("\n"))
    }
}

/// `nest n doc`: indent every line after the first by `n` spaces.
fn nest(n: usize, doc: &str) -> String {
    let pad = " ".repeat(n);
    doc.split('\n').collect::<Vec<_>>().join(&format!("\n{pad}"))
}

fn pretty_print_rulelist(name: &str, lst: &[RuleCode]) -> String {
    nest(2, &format!("override {name}:\n{}", pretty_print_list(|r| r.pretty(), lst)))
}

/// This function needs to convert the set to a list because Doc ann is not ordered.
fn pretty_print_registries(regs: &BTreeSet<Registry>) -> String {
    let v: Vec<&Registry> = regs.iter().collect();
    nest(2, &format!("allowed registries:\n{}", pretty_print_list(|r| format!("- {}", r.un_registry), &v)))
}

fn pretty_print_label_schema(ls: &LabelSchema) -> String {
    let v: Vec<(&String, &crate::rule::LabelType)> = ls.iter().collect();
    nest(
        2,
        &format!("label schema:\n{}", pretty_print_list(|(n, t)| format!("{}: {}", n, t.pretty()), &v)),
    )
}

/// pretty print a list with a custom pretty printing function for each element
fn pretty_print_list<A, F: Fn(&A) -> String>(prnt: F, lst: &[A]) -> String {
    if lst.is_empty() {
        return "none".to_string();
    }
    lst.iter().map(|i| format!("- {}", prnt(i))).collect::<Vec<_>>().join("\n")
}

/// Haskell `instance FromYAML PartialConfiguration` (plus `OverrideConfig`), as a function over YAML text.
pub fn partial_from_yaml(text: &str) -> Result<PartialConfiguration, String> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Configuration.hs FromYAML instances.
    // Best-effort implementation so the conformance runner can load config files.
    use serde_yaml_ng::Value;
    let v: Value = serde_yaml_ng::from_str(text).map_err(|e| e.to_string())?;
    let mut p = PartialConfiguration::default();
    let m = match v {
        Value::Mapping(m) => m,
        Value::Null => return Ok(p),
        _ => return Err("Configuration: expected mapping".to_string()),
    };
    let get = |k: &str| m.get(Value::String(k.to_string()));
    let bool_of = |k: &str| get(k).and_then(|v| v.as_bool());
    let strs = |v: Option<&Value>| -> Vec<String> {
        v.and_then(|v| v.as_sequence())
            .map(|s| s.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default()
    };
    p.partial_no_fail = bool_of("no-fail");
    p.partial_no_color = bool_of("no-color");
    p.partial_verbose = bool_of("verbose");
    let mut fmts = vec![];
    for f in strs(get("formats")).into_iter().chain(get("format").and_then(|v| v.as_str()).map(String::from)) {
        match crate::formatter::format::read_maybe_output_format(&f) {
            Some(x) => fmts.push(x),
            None => return Err(format!("invalid output format {f}")),
        }
    }
    p.partial_formats = fmts;
    if let Some(Value::Mapping(o)) = get("override") {
        let og = |k: &str| strs(o.get(Value::String(k.to_string())));
        p.partial_error_rules = og("error").into_iter().map(RuleCode::from).collect();
        p.partial_warning_rules = og("warning").into_iter().map(RuleCode::from).collect();
        p.partial_info_rules = og("info").into_iter().map(RuleCode::from).collect();
        p.partial_style_rules = og("style").into_iter().map(RuleCode::from).collect();
    }
    p.partial_ignore_rules = strs(get("ignored")).into_iter().map(RuleCode::from).collect();
    p.partial_allowed_registries = strs(get("trustedRegistries")).into_iter().map(|s| Registry::from(s.as_str())).collect();
    if let Some(Value::Mapping(ls)) = get("label-schema") {
        for (k, t) in ls {
            if let (Some(k), Some(t)) = (k.as_str(), t.as_str()) {
                let lt = crate::rule::read_either_label_type(t)?;
                p.partial_label_schema.insert(k.to_string(), lt);
            }
        }
    }
    p.partial_strict_labels = bool_of("strict-labels");
    p.partial_disable_ignore_pragma = bool_of("disable-ignore-pragma");
    if let Some(s) = get("failure-threshold").and_then(|v| v.as_str()) {
        p.partial_failure_threshold = Some(crate::rule::read_either_severity(s)?);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def() -> Configuration {
        Configuration::default()
    }

    #[test]
    fn override_default_configuration_with_empty_config() {
        assert_eq!(apply_partial_configuration(def(), PartialConfiguration::mempty()), def());
    }

    #[test]
    fn override_default_with_specific_configuration_no_fail() {
        let config = PartialConfiguration { partial_no_fail: Some(true), ..Default::default() };
        assert_eq!(apply_partial_configuration(def(), config), Configuration { no_fail: true, ..def() });
    }

    #[test]
    fn override_default_with_specific_configuration_no_color() {
        let config = PartialConfiguration { partial_no_color: Some(true), ..Default::default() };
        assert_eq!(apply_partial_configuration(def(), config), Configuration { no_color: true, ..def() });
    }

    #[test]
    fn empty_should_not_override_no_color() {
        let config = PartialConfiguration { partial_no_color: Some(true), ..Default::default() };
        let config2 = PartialConfiguration::default();
        assert_eq!(
            apply_partial_configuration(def(), config.append(config2)),
            Configuration { no_color: true, ..def() }
        );
    }

    #[test]
    fn override_default_with_specific_configuration_verbose() {
        let config = PartialConfiguration { partial_verbose: Some(true), ..Default::default() };
        assert_eq!(apply_partial_configuration(def(), config), Configuration { verbose: true, ..def() });
    }

    #[test]
    fn override_default_with_specific_configuration_output_format_json() {
        let config = PartialConfiguration { partial_formats: vec![OutputFormat::Json], ..Default::default() };
        assert_eq!(
            apply_partial_configuration(def(), config),
            Configuration { formats: vec![OutputFormat::Json], ..def() }
        );
    }

    #[test]
    fn override_default_with_specific_configuration_disable_ignore_pragma() {
        let config = PartialConfiguration { partial_disable_ignore_pragma: Some(true), ..Default::default() };
        assert_eq!(
            apply_partial_configuration(def(), config),
            Configuration { disable_ignore_pragma: true, ..def() }
        );
    }
}
