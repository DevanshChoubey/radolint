// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/Codacy.hs.

use super::format::{FormatResult, error_bundle_pretty};
use serde_json::{Value, json};
use std::io::Write;

/// Haskell `Issue` (rendered by its `ToJSON` instance).
struct Issue {
    filename: String,
    msg: String,
    pattern_id: String,
    line: u64,
}

impl Issue {
    /// `toJSON`; aeson `encode` emits keys in alphabetical order, which `serde_json::Map`
    /// (BTreeMap) reproduces.
    fn to_json(&self) -> Value {
        json!({
            "filename": self.filename,
            "patternId": self.pattern_id,
            "message": self.msg,
            "line": self.line,
        })
    }
}

/// Haskell `errorToIssue`.
fn error_to_issue(err: &radolint_parser::ParseError) -> Issue {
    let pos = err.position();
    Issue {
        filename: pos.name.to_string(),
        pattern_id: "DL1000".to_string(),
        msg: error_bundle_pretty(err),
        line: pos.line as u64,
    }
}

/// Haskell `checkToIssue`.
fn check_to_issue(filename: &str, c: &crate::rule::CheckFailure) -> Issue {
    Issue {
        filename: filename.to_string(),
        pattern_id: c.code.un_rule_code.to_string(),
        msg: c.message.to_string(),
        line: c.line as u64,
    }
}

/// Haskell `formatResult`.
fn format_result(r: &FormatResult) -> Vec<Issue> {
    let mut all_issues: Vec<Issue> = r.errors.iter().map(error_to_issue).collect();
    all_issues.extend(r.checks.iter().map(|c| check_to_issue(&r.file_name, c)));
    all_issues
}

/// Render: every issue encoded separately and concatenated (no separator, no newline).
pub fn render(results: &[FormatResult]) -> String {
    let mut out = String::new();
    for r in results {
        for issue in format_result(r) {
            out.push_str(&serde_json::to_string(&issue.to_json()).expect("json encode"));
        }
    }
    out
}

/// Haskell `hWrite`.
pub fn h_write(handle: &mut dyn Write, results: &[FormatResult]) -> std::io::Result<()> {
    handle.write_all(render(results).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{CheckFailure, DLSeverity, RuleCode};

    // No upstream spec exists for Codacy; this checks the object-per-issue encoding.
    #[test]
    fn issues_are_concatenated_objects() {
        let r = FormatResult {
            file_name: "f".to_string(),
            errors: vec![],
            checks: vec![CheckFailure {
                code: RuleCode {
                    un_rule_code: "DL3000".into(),
                },
                severity: DLSeverity::DLErrorC,
                message: "m".into(),
                line: 3,
            }],
        };
        let out = render(&[r.clone(), r]);
        let one = r#"{"filename":"f","line":3,"message":"m","patternId":"DL3000"}"#;
        assert_eq!(out, format!("{one}{one}"));
    }
}
