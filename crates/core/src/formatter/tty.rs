// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/TTY.hs.

use super::format::{error_message_line, severity_text, strip_newlines, FormatResult};
use crate::rule::{CheckFailure, DLSeverity};
use dlint_parser::ParseError;
use std::io::Write;

/// Haskell `formatError`.
pub fn format_error(err: &ParseError) -> String {
    strip_newlines(&error_message_line(err))
}

/// Haskell `formatCheck`.
pub fn format_check(nocolor: bool, source: &str, c: &CheckFailure) -> String {
    format!(
        "{}:{} {} {}: {}",
        source,
        c.line,
        c.code.un_rule_code,
        if nocolor { severity_text(c.severity).to_string() } else { colorized_severity(c.severity) },
        c.message
    )
}

/// Haskell `colorizedSeverity`.
pub fn colorized_severity(s: DLSeverity) -> String {
    // TODO(port): upstream/hadolint/src/Hadolint/Formatter/TTY.hs colorizedSeverity
    // (Colourista `formatWith [bold, red]` etc.: ANSI SGR escapes around the severity text).
    severity_text(s).to_string()
}

/// Haskell `hWrite`.
pub fn h_write(handle: &mut dyn Write, results: &[FormatResult], nocolor: bool) -> std::io::Result<()> {
    for r in results {
        for e in &r.errors {
            writeln!(handle, "{}", format_error(e))?;
        }
        for c in &r.checks {
            writeln!(handle, "{}", format_check(nocolor, &r.file_name, c))?;
        }
    }
    Ok(())
}
