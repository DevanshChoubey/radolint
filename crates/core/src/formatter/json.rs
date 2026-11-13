// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/Json.hs.

use super::format::{error_message, severity_text, FormatResult};
use crate::rule::DLSeverity;
use serde_json::{json, Value};
use std::io::Write;

/// Render the flattened JSON array (aeson `encode`: compact, keys in alphabetical order,
/// which `serde_json::Map` (BTreeMap) reproduces).
pub fn render(results: &[FormatResult]) -> String {
    let mut out: Vec<Value> = vec![];
    for r in results {
        for err in &r.errors {
            let pos = err.position();
            out.push(json!({
                "file": pos.name,
                "line": pos.line,
                "column": pos.column,
                "level": severity_text(DLSeverity::DLErrorC),
                "code": "DL1000",
                "message": error_message(err),
            }));
        }
        for c in &r.checks {
            out.push(json!({
                "file": r.file_name,
                "line": c.line,
                "column": 1,
                "level": severity_text(c.severity),
                "code": c.code.un_rule_code,
                "message": c.message,
            }));
        }
    }
    serde_json::to_string(&Value::Array(out)).expect("json encode")
}

/// Haskell `hWrite`.
pub fn h_write(handle: &mut dyn Write, results: &[FormatResult]) -> std::io::Result<()> {
    handle.write_all(render(results).as_bytes())
}
