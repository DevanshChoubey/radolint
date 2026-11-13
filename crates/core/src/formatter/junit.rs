// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/JUnit.hs.

use super::format::{error_bundle_pretty, error_position, severity_text, FormatResult};
use crate::meta::get_short_version;
use crate::rule::CheckFailure;
use dlint_parser::ParseError;
use std::collections::BTreeMap;
use std::io::Write;

// DIVERGENCE: there is no xml-conduit; this is a minimal Element/Node model rendered the way
// `XML.renderLBS XML.def` does (no pretty printing, attributes in Data.Map key order).
enum Node {
    Element(Element),
    Content(String),
}

struct Element {
    element_name: &'static str,
    element_attributes: BTreeMap<&'static str, String>,
    element_nodes: Vec<Node>,
}

fn escape_content(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
}

fn escape_attr(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
}

fn render_element(e: &Element, out: &mut String) {
    out.push('<');
    out.push_str(e.element_name);
    for (k, v) in &e.element_attributes {
        out.push(' ');
        out.push_str(k);
        out.push_str("=\"");
        escape_attr(v, out);
        out.push('"');
    }
    if e.element_nodes.is_empty() {
        out.push_str("/>");
        return;
    }
    out.push('>');
    for n in &e.element_nodes {
        match n {
            Node::Element(c) => render_element(c, out),
            Node::Content(t) => escape_content(t, out),
        }
    }
    out.push_str("</");
    out.push_str(e.element_name);
    out.push('>');
}

fn attrs(pairs: Vec<(&'static str, String)>) -> BTreeMap<&'static str, String> {
    pairs.into_iter().collect()
}

/// `Text.unlines`.
fn unlines(ls: &[String]) -> String {
    ls.iter().map(|l| format!("{l}\n")).collect()
}

fn provider_name() -> String {
    format!("Hadolint {}", get_short_version())
}

fn provider_id() -> &'static str {
    "hadolint"
}

/// UTC broken-down time from the system clock (`Time.getCurrentTime` + `formatTime`).
fn current_time_parts() -> (i64, u32, u32, u32, u32, u32) {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d, (rem / 3600) as u32, ((rem % 3600) / 60) as u32, (rem % 60) as u32)
}

/// Haskell `hWrite`.
pub fn h_write(handle: &mut dyn Write, results: &[FormatResult], file_path_in_report: Option<&str>) -> std::io::Result<()> {
    let time = current_time_parts();
    let root = root(time, results, file_path_in_report);
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    render_element(&root, &mut out);
    handle.write_all(out.as_bytes())
}

fn run_id(t: (i64, u32, u32, u32, u32, u32)) -> String {
    format!("{:04}{:02}{:02}_{:02}{:02}{:02}", t.0, t.1, t.2, t.3, t.4, t.5)
}

fn run_name(t: (i64, u32, u32, u32, u32, u32)) -> String {
    format!("Hadolint run at {:04}-{:02}-{:02} {:02}:{:02}:{:02}", t.0, t.1, t.2, t.3, t.4, t.5)
}

fn root(t: (i64, u32, u32, u32, u32, u32), results: &[FormatResult], maybe_filepath: Option<&str>) -> Element {
    Element {
        element_name: "testsuites",
        element_attributes: attrs(vec![("id", run_id(t)), ("name", run_name(t)), ("time", "0.001".to_string())]),
        element_nodes: results.iter().map(|r| render_result(r, maybe_filepath)).collect(),
    }
}

fn render_result(result: &FormatResult, maybe_filepath: Option<&str>) -> Node {
    let FormatResult { file_name: filename, errors, checks } = result;
    // `null maybeFilepath` is True only for Nothing
    let file: String = if maybe_filepath.is_none() { filename.clone() } else { maybe_filepath.unwrap_or("").to_string() };
    let error_nodes: Vec<Node> = errors.iter().map(|e| error_to_node(e, &file)).collect();
    let check_nodes: Vec<Node> = checks.iter().map(|c| check_to_node(c, &file)).collect();
    let parse_errors = error_nodes.len();
    let mut findings = error_nodes;
    findings.extend(check_nodes);
    Node::Element(Element {
        element_name: "testsuite",
        element_attributes: attrs(vec![
            ("id", provider_id().to_string()),
            ("name", provider_name()),
            ("time", "0.001".to_string()),
            ("failures", findings.len().to_string()),
            ("errors", parse_errors.to_string()),
        ]),
        element_nodes: findings,
    })
}

fn error_to_node(err: &ParseError, filename: &str) -> Node {
    let pos = error_position(err);
    let line = pos.line.to_string();
    let column = pos.column.to_string();
    Node::Element(Element {
        element_name: "testcase",
        element_attributes: attrs(vec![
            ("id", format!("{}.error", provider_id())),
            ("name", "DL1000".to_string()),
            ("time", "0.001".to_string()),
        ]),
        element_nodes: vec![Node::Element(Element {
            element_name: "failure",
            element_attributes: attrs(vec![
                ("type", "error".to_string()),
                ("message", format!("parse error at line {line}, column {column}")),
            ]),
            element_nodes: vec![Node::Content(unlines(&[
                format!("Parse Error in {filename}"),
                format!("Line: {line}"),
                format!("Column: {column}"),
                format!("Parse Error: {}", error_bundle_pretty(err)),
            ]))],
        })],
    })
}

fn check_to_node(check: &CheckFailure, filename: &str) -> Node {
    let CheckFailure { code, severity, message, line } = check;
    let sev = severity_text(*severity);
    Node::Element(Element {
        element_name: "testcase",
        element_attributes: attrs(vec![
            ("id", format!("{}.rule.{}", provider_id(), code.un_rule_code)),
            ("time", "0.001".to_string()),
        ]),
        element_nodes: vec![Node::Element(Element {
            element_name: "failure",
            element_attributes: attrs(vec![
                ("type", sev.to_string()),
                ("message", message.clone()),
                ("id", code.un_rule_code.clone()),
            ]),
            element_nodes: vec![Node::Content(unlines(&[
                format!("File: {filename}"),
                format!("Line: {line}"),
                "Category: Hadolint - Dockerfile Static Analysis".to_string(),
                format!("{sev}: {message}"),
            ]))],
        })],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::format::OutputFormat;
    use crate::rule::DLSeverity;
    use crate::testing::capture_format;

    /// Pull the run id/name out of the output (the spec computes them from "now").
    fn expectation(body: &str, cap: &str) -> String {
        let id = cap.split("id=\"").nth(1).unwrap().split('"').next().unwrap();
        let name = cap.split("name=\"").nth(1).unwrap().split('"').next().unwrap();
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><testsuites id=\"{id}\" name=\"{name}\" time=\"0.001\">{body}</testsuites>"
        )
    }

    fn cf(code: &str, severity: DLSeverity, message: &str, line: i64) -> CheckFailure {
        CheckFailure { code: code.into(), severity, message: message.to_string(), line }
    }

    fn case(code: &str, sev: &str, msg: &str, line: i64) -> String {
        format!(
            "<testcase id=\"hadolint.rule.{code}\" time=\"0.001\"><failure id=\"{code}\" message=\"{msg}\" type=\"{sev}\">File: &lt;string&gt;\nLine: {line}\nCategory: Hadolint - Dockerfile Static Analysis\n{sev}: {msg}\n</failure></testcase>"
        )
    }

    fn suite(failures: usize, inner: &str) -> String {
        format!(
            "<testsuite errors=\"0\" failures=\"{failures}\" id=\"hadolint\" name=\"Hadolint {}\" time=\"0.001\">{inner}</testsuite>",
            get_short_version()
        )
    }

    #[test]
    fn print_empty_results() {
        let cap = capture_format(OutputFormat::JUnit, vec![], true);
        let empty_suite = suite(0, "").replace("></testsuite>", "/>");
        assert_eq!(cap, expectation(&empty_suite, &cap));
    }

    #[test]
    fn print_one_failed_rule() {
        let cap = capture_format(OutputFormat::JUnit, vec![cf("DL2001", DLSeverity::DLWarningC, "test", 1)], true);
        assert_eq!(cap, expectation(&suite(1, &case("DL2001", "warning", "test", 1)), &cap));
    }

    #[test]
    fn print_multiple_failed_rules() {
        let cap = capture_format(
            OutputFormat::JUnit,
            vec![
                cf("DL2001", DLSeverity::DLWarningC, "test", 1),
                cf("DL2003", DLSeverity::DLInfoC, "test 2", 3),
            ],
            true,
        );
        let inner = format!("{}{}", case("DL2001", "warning", "test", 1), case("DL2003", "info", "test 2", 3));
        assert_eq!(cap, expectation(&suite(2, &inner), &cap));
    }
}
