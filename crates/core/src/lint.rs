// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Lint.hs.

use crate::config::configuration::Configuration;
use crate::formatter::format::{FormatResult, to_result};
use crate::process;
use crate::rule::{CheckFailure, DLSeverity, Failures, RuleCode};
use dlint_parser::ParseError;
use dlint_syntax::Dockerfile;

/// Haskell `lintIO`: parse each file (or stdin for "-") and lint it.
pub fn lint_io(config: &Configuration, d_files: &[String]) -> std::io::Result<Vec<FormatResult>> {
    let mut parsed = vec![];
    for f in d_files {
        if f == "-" {
            parsed.push(("-".to_string(), dlint_parser::parse_stdin()?));
        } else {
            parsed.push((f.clone(), dlint_parser::parse_file(f)?));
        }
    }
    Ok(lint(config, parsed))
}

/// Haskell `lint`.
pub fn lint(
    config: &Configuration,
    parsed_files: Vec<(String, Result<Dockerfile, ParseError>)>,
) -> Vec<FormatResult> {
    parsed_files
        .into_iter()
        .map(|(name, res)| to_result(&name, res.map(|d| analyze(config, &d))))
        .collect()
}

/// Haskell `analyze`.
pub fn analyze(config: &Configuration, dockerfile: &Dockerfile) -> Failures {
    fix_severity(config, process::run(config, dockerfile))
}

/// Haskell `fixSeverity`.
pub fn fix_severity(config: &Configuration, failures: Failures) -> Failures {
    let make_severity =
        |s: DLSeverity, rules: &[RuleCode], mut rule: CheckFailure| -> CheckFailure {
            if rules.contains(&rule.code) {
                rule.severity = s;
            }
            rule
        };
    let correct = |rule: CheckFailure| -> CheckFailure {
        // composed right-to-left: style, info, warning, error (error applied last, so it wins)
        let r = make_severity(DLSeverity::DLStyleC, &config.style_rules, rule);
        let r = make_severity(DLSeverity::DLInfoC, &config.info_rules, r);
        let r = make_severity(DLSeverity::DLWarningC, &config.warning_rules, r);
        make_severity(DLSeverity::DLErrorC, &config.error_rules, r)
    };
    failures
        .into_iter()
        .map(correct)
        .filter(|f| !config.ignore_rules.contains(&f.code) && f.severity != DLSeverity::DLIgnoreC)
        .collect()
}
