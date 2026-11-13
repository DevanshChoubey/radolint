// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/SonarQube.hs.

use super::format::{FormatResult, error_message, error_position};
use crate::rule::{CheckFailure, DLSeverity};
use radolint_parser::ParseError;
use serde_json::{Value, json};
use std::io::Write;

/// Haskell `SonarQubeFormat s e`.
enum SonarQubeFormat<'a> {
    SonarQubeCheck(String, &'a CheckFailure),
    SonarQubeError(String, &'a ParseError),
}

impl SonarQubeFormat<'_> {
    /// `instance ToJSON SonarQubeFormat`: `toJSON`.
    /// (aeson `object` renders keys alphabetically; `serde_json::Map` does the same.)
    fn to_json(&self) -> Value {
        match self {
            SonarQubeFormat::SonarQubeCheck(filename, c) => json!({
                "engineId": "Hadolint",
                "ruleId": c.code.un_rule_code,
                "severity": to_severity(c.severity),
                "type": to_type(c.severity),
                "primaryLocation": {
                    "message": c.message,
                    "filePath": filename,
                    "textRange": {
                        "startLine": c.line,
                        "endLine": c.line,
                        "startColumn": 0,
                        "endColumn": 1
                    }
                }
            }),
            SonarQubeFormat::SonarQubeError(filename, err) => {
                let pos = error_position(err);
                let linenumber = pos.line;
                let column = pos.column;
                json!({
                    "engineId": "Hadolint",
                    "ruleId": "DL1000",
                    "severity": "BLOCKER",
                    "type": "BUG",
                    "primaryLocation": {
                        "message": error_message(err),
                        "filePath": filename,
                        "textRange": {
                            "startLine": linenumber,
                            "endLine": linenumber,
                            "startColumn": column,
                            "endColumn": column
                        }
                    }
                })
            }
        }
    }
}

/// Haskell `formatResult`.
fn format_result<'a>(
    file_path_in_report: Option<&str>,
    r: &'a FormatResult,
) -> Vec<SonarQubeFormat<'a>> {
    // `if null filePathInReport then filename else getFilePath filePathInReport`
    let filepath = if file_path_in_report.is_none() {
        r.file_name.clone()
    } else {
        get_file_path(file_path_in_report)
    };
    let error_messages = r
        .errors
        .iter()
        .map(|e| SonarQubeFormat::SonarQubeError(filepath.clone(), e));
    let check_messages = r
        .checks
        .iter()
        .map(|c| SonarQubeFormat::SonarQubeCheck(filepath.clone(), c));
    error_messages.chain(check_messages).collect()
}

/// Haskell `hWrite`.
pub fn h_write(
    handle: &mut dyn Write,
    results: &[FormatResult],
    file_path_in_report: Option<&str>,
) -> std::io::Result<()> {
    let flattened: Vec<Value> = results
        .iter()
        .flat_map(|r| format_result(file_path_in_report, r))
        .map(|f| f.to_json())
        .collect();
    let out = serde_json::to_string(&json!({ "issues": flattened })).expect("json encode");
    handle.write_all(out.as_bytes())
}

/// Haskell `toType`.
fn to_type(s: DLSeverity) -> &'static str {
    match s {
        DLSeverity::DLErrorC => "BUG",
        _ => "CODE_SMELL",
    }
}

/// Haskell `toSeverity`.
fn to_severity(s: DLSeverity) -> &'static str {
    match s {
        DLSeverity::DLErrorC => "CRITICAL",
        DLSeverity::DLWarningC => "MAJOR",
        DLSeverity::DLInfoC => "MINOR",
        _ => "INFO",
    }
}

/// Haskell `getFilePath`.
fn get_file_path(file_path: Option<&str>) -> String {
    match file_path {
        None => String::new(),
        Some(file_path) => to_text(&[file_path]),
    }
}

/// Haskell `toText`.
fn to_text(paths: &[&str]) -> String {
    paths.concat()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::format::OutputFormat;
    use crate::rule::RuleCode;
    use crate::testing::assert_formatter_json;

    fn mk_issue(
        rule_id: &str,
        severity: &str,
        issue_type: &str,
        msg: &str,
        line_num: u32,
    ) -> Value {
        mk_issue_path(rule_id, severity, issue_type, msg, "<string>", line_num)
    }

    fn mk_issue_path(
        rule_id: &str,
        severity: &str,
        issue_type: &str,
        msg: &str,
        path: &str,
        line_num: u32,
    ) -> Value {
        json!({
            "engineId": "Hadolint",
            "ruleId": rule_id,
            "severity": severity,
            "type": issue_type,
            "primaryLocation": {
                "message": msg,
                "filePath": path,
                "textRange": {
                    "startLine": line_num,
                    "endLine": line_num,
                    "startColumn": 0,
                    "endColumn": 1
                }
            }
        })
    }

    fn cf(code: &str, severity: DLSeverity, message: &str, line: u32) -> CheckFailure {
        CheckFailure {
            code: RuleCode::from(code),
            severity,
            message: message.to_string(),
            line: line as _,
        }
    }

    #[test]
    fn print_empty_results() {
        assert_formatter_json(
            OutputFormat::SonarQube,
            vec![],
            true,
            &json!({ "issues": [] }),
        );
    }

    #[test]
    fn print_all_severity_levels_with_correct_mapping() {
        let checks = vec![
            cf("DL3000", DLSeverity::DLErrorC, "error msg", 1),
            cf("DL3001", DLSeverity::DLWarningC, "warning msg", 2),
            cf("DL3009", DLSeverity::DLInfoC, "info msg", 3),
            cf("DL3015", DLSeverity::DLStyleC, "style msg", 4),
        ];
        let expected = json!({
            "issues": [
                mk_issue("DL3000", "CRITICAL", "BUG", "error msg", 1),
                mk_issue("DL3001", "MAJOR", "CODE_SMELL", "warning msg", 2),
                mk_issue("DL3009", "MINOR", "CODE_SMELL", "info msg", 3),
                mk_issue("DL3015", "INFO", "CODE_SMELL", "style msg", 4)
            ]
        });
        assert_formatter_json(OutputFormat::SonarQube, checks, true, &expected);
    }

    #[test]
    fn use_custom_file_path_from_file_path_in_report() {
        let checks = vec![cf("DL3000", DLSeverity::DLErrorC, "error msg", 1)];
        let results = vec![FormatResult {
            file_name: "<string>".to_string(),
            errors: vec![],
            checks,
        }];
        let custom_path = Some("path/to/custom/Dockerfile");
        let expected = json!({
            "issues": [mk_issue_path("DL3000", "CRITICAL", "BUG", "error msg", "path/to/custom/Dockerfile", 1)]
        });
        let mut buf: Vec<u8> = vec![];
        // DIVERGENCE: `capture (write ...)` goes to stdout; the handle writer is called directly.
        h_write(&mut buf, &results, custom_path).expect("write");
        let v: Value = serde_json::from_slice(&buf).expect("valid json");
        assert_eq!(v, expected);
    }
}
