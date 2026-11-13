// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/Sarif.hs.

use super::format::{FormatResult, error_message, error_position};
use crate::meta::get_short_version;
use crate::rule::{CheckFailure, DLSeverity};
use dlint_parser::ParseError;
use serde_json::{Value, json};
use std::io::Write;

/// Haskell `data SarifFormat s e`.
enum SarifFormat<'a> {
    SarifCheck(&'a str, &'a CheckFailure),
    SarifError(&'a ParseError),
}

impl SarifFormat<'_> {
    /// Haskell `toJSON`.
    fn to_json(&self) -> Value {
        match self {
            SarifFormat::SarifCheck(filename, c) => {
                let code = &c.code.un_rule_code;
                let language = if code.starts_with("DL") {
                    "dockerfile"
                } else {
                    "sh"
                };
                json!({
                    "ruleId": code,
                    "level": to_severity(c.severity),
                    "message": { "text": c.message },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": { "uri": filename },
                            "region": {
                                "startLine": c.line,
                                "endLine": c.line,
                                "startColumn": 1,
                                "endColumn": 1,
                                "sourceLanguage": language
                            }
                        }
                    }]
                })
            }
            SarifFormat::SarifError(err) => {
                let pos = error_position(err);
                let linenumber = pos.line;
                let column = pos.column;
                json!({
                    "ruleId": "DL1000",
                    "level": "error",
                    "message": { "text": error_message(err) },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": { "uri": pos.name },
                            "region": {
                                "startLine": linenumber,
                                "endLine": linenumber,
                                "startColumn": column,
                                "endColumn": column,
                                "sourceLanguage": "dockerfile"
                            }
                        }
                    }]
                })
            }
        }
    }
}

/// Haskell `formatResult`.
fn format_result(r: &FormatResult) -> Vec<SarifFormat<'_>> {
    let error_messages = r.errors.iter().map(SarifFormat::SarifError);
    let check_messages = r
        .checks
        .iter()
        .map(|c| SarifFormat::SarifCheck(&r.file_name, c));
    error_messages.chain(check_messages).collect()
}

/// Haskell `hWrite`.
pub fn h_write(handle: &mut dyn Write, results: &[FormatResult]) -> std::io::Result<()> {
    let flattened: Vec<Value> = results
        .iter()
        .flat_map(format_result)
        .map(|f| f.to_json())
        .collect();
    // aeson `encode` emits keys in alphabetical order; serde_json's Map (BTreeMap) does too.
    let doc = json!({
        "version": "2.1.0",
        "$schema": "http://json.schemastore.org/sarif-2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "Hadolint",
                    "fullName": "Haskell Dockerfile Linter",
                    "downloadUri": "https://github.com/hadolint/hadolint",
                    "version": get_short_version(),
                    "shortDescription": {
                        "text": "Dockerfile linter, validate inline bash, written in Haskell"
                    }
                }
            },
            "results": flattened,
            "defaultSourceLanguage": "dockerfile"
        }]
    });
    handle.write_all(serde_json::to_string(&doc).expect("json encode").as_bytes())
}

/// SARIF only specifies three severities "error", "warning" and "note"
/// We pack our "info" and "style" severities together into the "note" severity
/// here.
fn to_severity(s: DLSeverity) -> &'static str {
    match s {
        DLSeverity::DLErrorC => "error",
        DLSeverity::DLWarningC => "warning",
        _ => "note",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::format::OutputFormat;
    use crate::testing::assert_formatter_json;

    fn tool() -> Value {
        json!({
            "driver": {
                "name": "Hadolint",
                "fullName": "Haskell Dockerfile Linter",
                "downloadUri": "https://github.com/hadolint/hadolint",
                "version": get_short_version(),
                "shortDescription": {
                    "text": "Dockerfile linter, validate inline bash, written in Haskell"
                }
            }
        })
    }

    fn result(code: &str, level: &str, text: &str, line: u32) -> Value {
        json!({
            "level": level,
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": { "uri": "<string>" },
                    "region": {
                        "startLine": line, "endLine": line,
                        "startColumn": 1, "endColumn": 1,
                        "sourceLanguage": "dockerfile"
                    }
                }
            }],
            "message": { "text": text },
            "ruleId": code
        })
    }

    fn expectation(results: Vec<Value>) -> Value {
        json!({
            "version": "2.1.0",
            "$schema": "http://json.schemastore.org/sarif-2.1.0",
            "runs": [{
                "tool": tool(),
                "results": results,
                "defaultSourceLanguage": "dockerfile"
            }]
        })
    }

    fn cf(code: &str, severity: DLSeverity, message: &str, line: u32) -> CheckFailure {
        CheckFailure {
            code: code.into(),
            severity,
            message: message.to_string(),
            line: line as _,
        }
    }

    #[test]
    fn print_empty_results() {
        assert_formatter_json(OutputFormat::Sarif, vec![], true, &expectation(vec![]));
    }

    #[test]
    fn print_one_failed_rule() {
        let check_fails = vec![cf("DL2001", DLSeverity::DLWarningC, "test", 1)];
        assert_formatter_json(
            OutputFormat::Sarif,
            check_fails,
            true,
            &expectation(vec![result("DL2001", "warning", "test", 1)]),
        );
    }

    #[test]
    fn print_multiple_failed_rules() {
        let check_fails = vec![
            cf("DL2001", DLSeverity::DLWarningC, "test", 1),
            cf("DL2003", DLSeverity::DLInfoC, "test 2", 3),
        ];
        assert_formatter_json(
            OutputFormat::Sarif,
            check_fails,
            true,
            &expectation(vec![
                result("DL2001", "warning", "test", 1),
                result("DL2003", "note", "test 2", 3),
            ]),
        );
    }
}
