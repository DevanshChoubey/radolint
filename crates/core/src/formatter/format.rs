// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/Format.hs.

use crate::rule::{DLSeverity, Failures};
use dlint_parser::ParseError;

/// Haskell `OutputFormat` (`GitLabCodeclimateJson` etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Checkstyle,
    Codacy,
    CodeclimateJson,
    GitLabCodeclimateJson,
    Gnu,
    JUnit,
    Json,
    Sarif,
    SonarQube,
    TTY,
}

impl OutputFormat {
    /// `instance Pretty OutputFormat`.
    pub fn pretty(&self) -> &'static str {
        match self {
            OutputFormat::Checkstyle => "checkstyle",
            OutputFormat::Codacy => "codacy",
            OutputFormat::CodeclimateJson => "codeclimate",
            OutputFormat::GitLabCodeclimateJson => "gitlab_codeclimate",
            OutputFormat::Gnu => "gnu",
            OutputFormat::JUnit => "junit",
            OutputFormat::Json => "json",
            OutputFormat::Sarif => "sarif",
            OutputFormat::SonarQube => "sonarqube",
            OutputFormat::TTY => "tty",
        }
    }
    /// `instance Monoid OutputFormat` `mempty`.
    pub fn mempty() -> Self {
        OutputFormat::TTY
    }
    /// `instance Semigroup OutputFormat`: `_ <> f = f`.
    pub fn append(self, f: OutputFormat) -> OutputFormat {
        f
    }
}

impl Default for OutputFormat {
    /// `instance Default OutputFormat`.
    fn default() -> Self {
        OutputFormat::TTY
    }
}

/// Haskell `readMaybeOutputFormat`.
pub fn read_maybe_output_format(t: &str) -> Option<OutputFormat> {
    Some(match t {
        "checkstyle" => OutputFormat::Checkstyle,
        "codacy" => OutputFormat::Codacy,
        "codeclimate" => OutputFormat::CodeclimateJson,
        "gitlab_codeclimate" => OutputFormat::GitLabCodeclimateJson,
        "gnu" => OutputFormat::Gnu,
        "junit" => OutputFormat::JUnit,
        "json" => OutputFormat::Json,
        "sarif" => OutputFormat::Sarif,
        "sonarqube" => OutputFormat::SonarQube,
        "tty" => OutputFormat::TTY,
        _ => return None,
    })
}

/// Haskell `Result s e` (always `Result Text DockerfileError` in practice).
#[derive(Debug, Clone)]
pub struct FormatResult {
    pub file_name: String,
    pub errors: Vec<ParseError>,
    pub checks: Failures,
}

/// Haskell `toResult`.
pub fn to_result(file: &str, res: Result<Failures, ParseError>) -> FormatResult {
    match res {
        Err(err) => FormatResult {
            file_name: file.to_string(),
            errors: vec![err],
            checks: vec![],
        },
        Ok(c) => {
            // Upstream `Seq.unstableSort`: equal-line order follows containers' finger-tree/pairing-heap shape.
            let c = crate::seq_sort::unstable_sort_by_line(c);
            FormatResult {
                file_name: file.to_string(),
                errors: vec![],
                checks: c,
            }
        }
    }
}

/// Haskell `severityText`.
pub fn severity_text(s: DLSeverity) -> &'static str {
    match s {
        DLSeverity::DLErrorC => "error",
        DLSeverity::DLWarningC => "warning",
        DLSeverity::DLInfoC => "info",
        DLSeverity::DLStyleC => "style",
        _ => "",
    }
}

/// Haskell `stripNewlines`.
pub fn strip_newlines(s: &str) -> String {
    s.chars().map(|c| if c == '\n' { ' ' } else { c }).collect()
}

/// Haskell `errorMessageLine`.
pub fn error_message_line(err: &ParseError) -> String {
    err.message_line()
}

/// Haskell `errorMessage`.
pub fn error_message(err: &ParseError) -> String {
    err.message()
}

/// Haskell `errorPositionPretty`.
pub fn error_position_pretty(err: &ParseError) -> String {
    err.position_pretty()
}

/// Haskell `errorPosition`.
pub fn error_position(err: &ParseError) -> dlint_parser::combinator::SourcePos {
    err.position()
}

/// Haskell `errorBundlePretty`.
pub fn error_bundle_pretty(err: &ParseError) -> String {
    err.bundle_pretty()
}
