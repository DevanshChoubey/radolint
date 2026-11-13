// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/Codeclimate.hs.

use super::format::{FormatResult, error_bundle_pretty, error_position};
use crate::rule::{CheckFailure, DLSeverity};
use radolint_parser::ParseError;
use serde_json::{Value, json};
use std::io::Write;

/// Haskell `Issue`.
struct Issue {
    check_name: String,
    description: String,
    location: Location,
    impact: String,
}

/// Haskell `FingerprintIssue`.
struct FingerprintIssue {
    issue: Issue,
    fingerprint: String,
}

/// Haskell `Location`.
enum Location {
    LocLine(String, usize),
    LocPos(String, Pos),
}

/// Haskell `Pos`.
struct Pos {
    line: usize,
    column: usize,
}

impl Location {
    /// `instance ToJSON Location` (aeson objects serialise with sorted keys; serde_json's
    /// `Map` is a BTreeMap, which reproduces that).
    fn to_json(&self) -> Value {
        match self {
            Location::LocLine(path, l) => {
                json!({"path": path, "lines": {"begin": l, "end": l}})
            }
            Location::LocPos(path, pos) => {
                let p = json!({"line": pos.line, "column": pos.column});
                json!({"path": path, "positions": {"begin": p.clone(), "end": p}})
            }
        }
    }
}

impl Issue {
    /// `instance ToJSON Issue`.
    fn to_json(&self) -> Value {
        json!({
            "type": "issue",
            "check_name": self.check_name,
            "description": self.description,
            "categories": ["Bug Risk"],
            "location": self.location.to_json(),
            "severity": self.impact,
        })
    }
}

impl FingerprintIssue {
    /// `instance ToJSON FingerprintIssue`.
    fn to_json(&self) -> Value {
        json!({
            "type": "issue",
            "fingerprint": self.fingerprint,
            "check_name": self.issue.check_name,
            "description": self.issue.description,
            "categories": ["Bug Risk"],
            "location": self.issue.location.to_json(),
            "severity": self.issue.impact,
        })
    }
}

fn encode(v: &Value) -> String {
    serde_json::to_string(v).expect("json encode")
}

/// Haskell `errorToIssue`.
fn error_to_issue(err: &ParseError) -> Issue {
    let pos = error_position(err);
    let line = pos.line;
    let column = pos.column;
    Issue {
        check_name: "DL1000".to_string(),
        description: error_bundle_pretty(err),
        location: Location::LocPos(pos.name.clone(), Pos { line, column }),
        impact: severity_text(DLSeverity::DLErrorC).to_string(),
    }
}

/// Haskell `checkToIssue`.
fn check_to_issue(file_name: &str, file_path_in_report: Option<&str>, f: &CheckFailure) -> Issue {
    // `null filePathInReport` is True only for Nothing.
    let report_file_name = match file_path_in_report {
        None => file_name.to_string(),
        Some(_) => get_file_path(file_path_in_report),
    };
    Issue {
        check_name: f.code.un_rule_code.clone(),
        description: f.message.clone(),
        location: Location::LocLine(report_file_name, f.line as usize),
        impact: severity_text(f.severity).to_string(),
    }
}

/// Haskell `severityText` (local to this module).
fn severity_text(severity: DLSeverity) -> &'static str {
    match severity {
        DLSeverity::DLErrorC => "blocker",
        DLSeverity::DLWarningC => "major",
        DLSeverity::DLInfoC => "info",
        DLSeverity::DLStyleC => "minor",
        _ => "",
    }
}

/// Haskell `generateFingerprint`: lowercase hex of SHA1 over the encoded issue.
fn generate_fingerprint(i: &Issue) -> String {
    sha1(encode(&i.to_json()).as_bytes())
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

// DIVERGENCE: no SHA1 crate is available in the manifest, so SHA-1 is implemented inline.
fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = h;
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let t = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = t;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for (i, v) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

/// Haskell `issueToFingerprintIssue`.
fn issue_to_fingerprint_issue(i: Issue) -> FingerprintIssue {
    let fingerprint = generate_fingerprint(&i);
    FingerprintIssue {
        issue: i,
        fingerprint,
    }
}

/// Haskell `formatResult`.
fn format_result(result: &FormatResult, file_path_in_report: Option<&str>) -> Vec<Issue> {
    let mut v: Vec<Issue> = result.errors.iter().map(error_to_issue).collect();
    v.extend(
        result
            .checks
            .iter()
            .map(|c| check_to_issue(&result.file_name, file_path_in_report, c)),
    );
    v
}

/// Haskell `formatGitLabResult`.
fn format_git_lab_result(
    result: &FormatResult,
    file_path_in_report: Option<&str>,
) -> Vec<FingerprintIssue> {
    format_result(result, file_path_in_report)
        .into_iter()
        .map(issue_to_fingerprint_issue)
        .collect()
}

/// Haskell `printResult`.
fn print_result(
    handle: &mut dyn Write,
    result: &FormatResult,
    file_path_in_report: Option<&str>,
) -> std::io::Result<()> {
    for value in format_result(result, file_path_in_report) {
        handle.write_all(encode(&value.to_json()).as_bytes())?;
        handle.write_all(&[0x00])?;
    }
    Ok(())
}

/// Haskell `hWrite`.
pub fn h_write(
    handle: &mut dyn Write,
    results: &[FormatResult],
    file_path_in_report: Option<&str>,
) -> std::io::Result<()> {
    for r in results {
        print_result(handle, r, file_path_in_report)?;
    }
    Ok(())
}

/// Haskell `hWriteGitLab`.
pub fn h_write_gitlab(
    handle: &mut dyn Write,
    results: &[FormatResult],
    file_path_in_report: Option<&str>,
) -> std::io::Result<()> {
    let flattened: Vec<Value> = results
        .iter()
        .flat_map(|r| format_git_lab_result(r, file_path_in_report))
        .map(|f| f.to_json())
        .collect();
    handle.write_all(encode(&Value::Array(flattened)).as_bytes())
}

/// Haskell `getFilePath`.
fn get_file_path(file_path: Option<&str>) -> String {
    match file_path {
        None => String::new(),
        Some(p) => p.to_string(),
    }
}

#[cfg(test)]
mod tests {
    // upstream has no CodeclimateSpec.hs; sanity checks only.
    use super::*;

    #[test]
    fn sha1_abc() {
        let h: String = sha1(b"abc").iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(h, "a9993e364706816aba3e25717850c26c9cd0d89d");
    }

    #[test]
    fn check_issue_lines_json() {
        let i = Issue {
            check_name: "DL3000".into(),
            description: "m".into(),
            location: Location::LocLine("f".into(), 2),
            impact: "major".into(),
        };
        assert_eq!(
            encode(&i.to_json()),
            r#"{"categories":["Bug Risk"],"check_name":"DL3000","description":"m","location":{"lines":{"begin":2,"end":2},"path":"f"},"severity":"major","type":"issue"}"#
        );
    }
}
