// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/Checkstyle.hs.

use super::format::{error_bundle_pretty, error_position, severity_text, FormatResult};
use crate::rule::{CheckFailure, DLSeverity};
use dlint_parser::ParseError;
use std::io::Write;

/// Minimal stand-in for `Text.XML.Node` (only elements without children or with nodes).
enum Node {
    Element { name: &'static str, attrs: Vec<(&'static str, String)>, nodes: Vec<Node> },
}

// DIVERGENCE: xml-conduit's renderer is emulated by hand: attributes are emitted in `Data.Map`
// (sorted by name) order, childless elements are self-closed, and `& < > " '` are escaped.
fn escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '&' => o.push_str("&amp;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&apos;"),
            c => o.push(c),
        }
    }
    o
}

fn render_node(n: &Node, out: &mut String) {
    let Node::Element { name, attrs, nodes } = n;
    let mut attrs: Vec<&(&'static str, String)> = attrs.iter().collect();
    attrs.sort_by(|a, b| a.0.cmp(b.0)); // Map.fromList
    out.push('<');
    out.push_str(name);
    for (k, v) in attrs {
        out.push(' ');
        out.push_str(k);
        out.push_str("=\"");
        out.push_str(&escape(v));
        out.push('"');
    }
    if nodes.is_empty() {
        out.push_str("/>");
    } else {
        out.push('>');
        for c in nodes {
            render_node(c, out);
        }
        out.push_str("</");
        out.push_str(name);
        out.push('>');
    }
}

/// Haskell `errorToNode`.
fn error_to_node(err: &ParseError) -> Node {
    let pos = error_position(err);
    Node::Element {
        name: "error",
        attrs: vec![
            ("line", pos.line.to_string()),
            ("column", pos.column.to_string()),
            ("severity", severity_text(DLSeverity::DLErrorC).to_string()),
            ("message", error_bundle_pretty(err)),
        ],
        nodes: vec![],
    }
}

/// Haskell `checkToNode`.
fn check_to_node(c: &CheckFailure) -> Node {
    Node::Element {
        name: "error",
        attrs: vec![
            ("line", c.line.to_string()),
            ("column", "1".to_string()),
            ("severity", severity_text(c.severity).to_string()),
            ("message", c.message.clone()),
            ("source", c.code.un_rule_code.clone()),
        ],
        nodes: vec![],
    }
}

/// Haskell `renderNodes`.
fn render_nodes(result: &FormatResult) -> Vec<Node> {
    let is_empty = result.checks.is_empty() && result.errors.is_empty();
    if is_empty {
        vec![]
    } else {
        let mut v: Vec<Node> = result.errors.iter().map(error_to_node).collect();
        v.extend(result.checks.iter().map(check_to_node));
        v
    }
}

/// Haskell `toFile`.
fn to_file(result: &FormatResult, file_path_in_report: Option<&str>) -> Node {
    // `null filePathInReport` is true for Nothing only (Maybe's Foldable).
    let filepath = match file_path_in_report {
        None => result.file_name.clone(),
        Some(p) => p.to_string(),
    };
    Node::Element { name: "file", attrs: vec![("name", filepath)], nodes: render_nodes(result) }
}

/// Haskell `renderResults`.
fn render_results(results: &[FormatResult], file_path_in_report: Option<&str>) -> Node {
    Node::Element {
        name: "checkstyle",
        attrs: vec![("version", "4.3".to_string())],
        nodes: results.iter().map(|r| to_file(r, file_path_in_report)).collect(),
    }
}

/// Haskell `hWrite`.
pub fn h_write(handle: &mut dyn Write, results: &[FormatResult], file_path_in_report: Option<&str>) -> std::io::Result<()> {
    let mut s = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    render_node(&render_results(results, file_path_in_report), &mut s);
    handle.write_all(s.as_bytes())
}

// Upstream has no CheckstyleSpec; these cover the renderer.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::RuleCode;

    fn render(r: &[FormatResult], p: Option<&str>) -> String {
        let mut buf = Vec::new();
        h_write(&mut buf, r, p).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn empty_result_is_self_closed_file() {
        let r = FormatResult { file_name: "a".into(), errors: vec![], checks: vec![] };
        assert_eq!(
            render(&[r], None),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><checkstyle version=\"4.3\"><file name=\"a\"/></checkstyle>"
        );
    }

    #[test]
    fn check_is_rendered_with_sorted_escaped_attrs() {
        let r = FormatResult {
            file_name: "a".into(),
            errors: vec![],
            checks: vec![CheckFailure {
                code: RuleCode { un_rule_code: "DL3000".into() },
                severity: DLSeverity::DLErrorC,
                message: "x<y".into(),
                line: 2,
            }],
        };
        assert_eq!(
            render(&[r], Some("b")),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><checkstyle version=\"4.3\"><file name=\"b\"><error column=\"1\" line=\"2\" message=\"x&lt;y\" severity=\"error\" source=\"DL3000\"/></file></checkstyle>"
        );
    }
}
