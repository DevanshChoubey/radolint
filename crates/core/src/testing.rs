// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/test/Helpers.hs: assertion helpers used by the per-rule specs.
//!
//! The Haskell helpers take the configuration through the implicit parameter `?config`.
//! Here every helper has a default-configuration form (`rule_catches`) and a `_with` form
//! taking `&Configuration` (`rule_catches_with`). Failures `panic!` (the `#[test]` fails).

use crate::config::configuration::Configuration;
use crate::formatter::format::{FormatResult, OutputFormat};
use crate::formatter::tty::format_check;
use crate::process;
use crate::rule::{CheckFailure, Failures, RuleCode};
use radolint_parser::parse_text;
use radolint_syntax::{Dockerfile, Instruction, InstructionPos};

/// Haskell `assertChecks`: parse `dockerfile <> "\n"` and hand the failures to `make_assertions`.
pub fn assert_checks_with<A>(
    config: &Configuration,
    dockerfile: &str,
    make_assertions: impl FnOnce(Failures) -> A,
) -> A {
    match parse_text(&format!("{dockerfile}\n")) {
        Err(err) => panic!("{err:?}"),
        Ok(d) => make_assertions(process::run(config, &d)),
    }
}

/// `assertChecks` with the default configuration.
pub fn assert_checks<A>(dockerfile: &str, make_assertions: impl FnOnce(Failures) -> A) -> A {
    assert_checks_with(&Configuration::default(), dockerfile, make_assertions)
}

/// Haskell `assertOnBuildChecks`: every `RUN` is wrapped in `ONBUILD`.
pub fn assert_on_build_checks_with<A>(
    config: &Configuration,
    dockerfile: &str,
    make_assertions: impl FnOnce(Failures) -> A,
) -> A {
    match parse_text(&format!("{dockerfile}\n")) {
        Err(err) => panic!("{err:?}"),
        Ok(d) => {
            let wrapped: Dockerfile = d.into_iter().map(wrap_in_on_build).collect();
            make_assertions(process::run(config, &wrapped))
        }
    }
}

/// `assertOnBuildChecks` with the default configuration.
pub fn assert_on_build_checks<A>(
    dockerfile: &str,
    make_assertions: impl FnOnce(Failures) -> A,
) -> A {
    assert_on_build_checks_with(&Configuration::default(), dockerfile, make_assertions)
}

fn wrap_in_on_build(ip: InstructionPos<String>) -> InstructionPos<String> {
    match ip.instruction {
        Instruction::Run(args) => InstructionPos {
            instruction: Instruction::OnBuild(Box::new(Instruction::Run(args))),
            sourcename: ip.sourcename,
            line_number: ip.line_number,
        },
        _ => ip,
    }
}

/// Haskell `hasInvalidLines`.
pub fn has_invalid_lines(failures: &Failures) -> bool {
    failures.iter().any(|f| f.line <= 0)
}

/// Haskell `ruleCatches`: at least one failure with the code, and sane line numbers.
pub fn rule_catches_with(config: &Configuration, expected: &str, dockerfile: &str) {
    assert_checks_with(config, dockerfile, |checks| {
        fails_with_some(&RuleCode::from(expected), &checks);
        assert!(
            !has_invalid_lines(&checks),
            "Incorrect line number for result"
        );
    })
}
/// `ruleCatches` with the default configuration.
pub fn rule_catches(expected: &str, dockerfile: &str) {
    rule_catches_with(&Configuration::default(), expected, dockerfile)
}

/// Haskell `onBuildRuleCatches`: exactly one failure.
pub fn on_build_rule_catches_with(config: &Configuration, code: &str, dockerfile: &str) {
    assert_on_build_checks_with(config, dockerfile, |checks| {
        fails_with(1, &RuleCode::from(code), &checks);
        assert!(
            !has_invalid_lines(&checks),
            "Incorrect line number for result"
        );
    })
}
/// `onBuildRuleCatches` with the default configuration.
pub fn on_build_rule_catches(code: &str, dockerfile: &str) {
    on_build_rule_catches_with(&Configuration::default(), code, dockerfile)
}

/// Haskell `ruleCatchesNot`.
pub fn rule_catches_not_with(config: &Configuration, code: &str, dockerfile: &str) {
    assert_checks_with(config, dockerfile, |checks| {
        fails_with(0, &RuleCode::from(code), &checks)
    })
}
/// `ruleCatchesNot` with the default configuration.
pub fn rule_catches_not(code: &str, dockerfile: &str) {
    rule_catches_not_with(&Configuration::default(), code, dockerfile)
}

/// Haskell `onBuildRuleCatchesNot`.
pub fn on_build_rule_catches_not_with(config: &Configuration, code: &str, dockerfile: &str) {
    assert_on_build_checks_with(config, dockerfile, |checks| {
        fails_with(0, &RuleCode::from(code), &checks)
    })
}
/// `onBuildRuleCatchesNot` with the default configuration.
pub fn on_build_rule_catches_not(code: &str, dockerfile: &str) {
    on_build_rule_catches_not_with(&Configuration::default(), code, dockerfile)
}

/// Haskell `formatChecksNoColor`.
pub fn format_checks_no_color<'a>(checks: impl IntoIterator<Item = &'a CheckFailure>) -> String {
    checks
        .into_iter()
        .map(|c| format!("{}\n", format_check(true, "line", c)))
        .collect()
}

/// Haskell `failsWithSome`.
pub fn fails_with_some(expected: &RuleCode, failures: &Failures) {
    if !failures.iter().any(|f| &f.code == expected) {
        panic!(
            "I was expecting to catch at least one error for {}",
            expected.show()
        );
    }
}

/// Haskell `failsWith`.
pub fn fails_with(times: usize, expected: &RuleCode, failures: &Failures) {
    let matched: Vec<&CheckFailure> = failures.iter().filter(|f| &f.code == expected).collect();
    if matched.len() != times {
        panic!(
            "I was expecting to catch exactly {} error(s) for {}. Found: \n{}",
            times,
            expected.show(),
            format_checks_no_color(matched)
        );
    }
}

/// Haskell `failsShellcheck`.
pub fn fails_shellcheck(checks: &Failures) {
    if !checks.iter().any(|f| f.code.un_rule_code.starts_with("SC")) {
        panic!("I was expecting to catch at least one error with shellcheck");
    }
}

/// Haskell `passesShellcheck`.
pub fn passes_shellcheck(checks: &Failures) {
    let matched: Vec<&CheckFailure> = checks
        .iter()
        .filter(|f| f.code.un_rule_code.starts_with("SC"))
        .collect();
    if !matched.is_empty() {
        panic!(
            "I was expecting to catch no errors with shellcheck. Found: \n{}",
            format_checks_no_color(matched)
        );
    }
}

/// Run the formatter into a string (`capture (write [] [format] noColor (Just "<string>") results)`).
pub fn capture_format(format: OutputFormat, failures: Vec<CheckFailure>, no_color: bool) -> String {
    let results = vec![FormatResult {
        file_name: "<string>".to_string(),
        errors: vec![],
        checks: failures,
    }];
    let mut buf: Vec<u8> = vec![];
    crate::formatter::h_write(&mut buf, format, no_color, Some("<string>"), &results)
        .expect("write");
    String::from_utf8(buf).expect("utf8")
}

/// Haskell `assertFormatter`.
pub fn assert_formatter(
    format: OutputFormat,
    failures: Vec<CheckFailure>,
    no_color: bool,
    expectation: &str,
) {
    assert_eq!(capture_format(format, failures, no_color), expectation);
}

/// Haskell `assertFormatterJson`: decode the output as JSON and compare values.
pub fn assert_formatter_json(
    format: OutputFormat,
    failures: Vec<CheckFailure>,
    no_color: bool,
    expectation: &serde_json::Value,
) {
    let cap = capture_format(format, failures, no_color);
    let v: serde_json::Value =
        serde_json::from_str(&cap).unwrap_or_else(|e| panic!("invalid JSON ({e}): {cap}"));
    assert_eq!(&v, expectation);
}

/// Haskell `assertFormatterXML`: compares the captured output with the expected XML text.
/// DIVERGENCE: there is no XML document type here; porters of the XML formatter specs compare
/// the (whitespace-normalised) rendered text instead of `Text.XML.Document` values.
pub fn assert_formatter_xml(
    format: OutputFormat,
    failures: Vec<CheckFailure>,
    no_color: bool,
    expectation: &str,
) {
    let cap = capture_format(format, failures, no_color);
    assert_eq!(cap.trim(), expectation.trim());
}
