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

/// Colourista `formatWith`: each style's SGR escape, then the text, then reset.
fn format_with(styles: &[&str], text: &str) -> String {
    let mut out = String::new();
    for st in styles {
        out.push_str(st);
    }
    out.push_str(text);
    out.push_str("\x1b[0m");
    out
}

/// Haskell `colorizedSeverity`.
pub fn colorized_severity(s: DLSeverity) -> String {
    const BOLD: &str = "\x1b[1m";
    const RED: &str = "\x1b[91m";
    const YELLOW: &str = "\x1b[93m";
    const GREEN: &str = "\x1b[92m";
    const CYAN: &str = "\x1b[96m";
    match s {
        DLSeverity::DLErrorC => format_with(&[BOLD, RED], severity_text(s)),
        DLSeverity::DLWarningC => format_with(&[BOLD, YELLOW], severity_text(s)),
        DLSeverity::DLInfoC => format_with(&[GREEN], severity_text(s)),
        DLSeverity::DLStyleC => format_with(&[CYAN], severity_text(s)),
        _ => severity_text(s).to_string(),
    }
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

#[cfg(test)]
mod tests {
    use crate::formatter::OutputFormat;
    use crate::rule::{CheckFailure, DLSeverity};
    use crate::testing::assert_formatter;

    fn cf(code: &str, severity: DLSeverity, message: &str, line: i64) -> CheckFailure {
        CheckFailure { code: code.into(), severity, message: message.to_string(), line: line as _ }
    }

    #[test]
    fn print_empty_results() {
        assert_formatter(OutputFormat::TTY, vec![], true, "");
    }

    #[test]
    fn print_some_result_no_colors() {
        assert_formatter(
            OutputFormat::TTY,
            vec![cf("DL2001", DLSeverity::DLInfoC, "test", 1)],
            true,
            "<string>:1 DL2001 info: test\n",
        );
    }

    #[test]
    fn print_some_result_with_colors() {
        assert_formatter(
            OutputFormat::TTY,
            vec![cf("DL2001", DLSeverity::DLInfoC, "test", 1)],
            false,
            "<string>:1 DL2001 \x1b[92minfo\x1b[0m: test\n",
        );
    }

    #[test]
    fn print_multiple_results_no_colors() {
        assert_formatter(
            OutputFormat::TTY,
            vec![
                cf("DL2001", DLSeverity::DLInfoC, "test", 1),
                cf("DL2002", DLSeverity::DLWarningC, "foo", 3),
            ],
            true,
            "<string>:1 DL2001 info: test\n<string>:3 DL2002 warning: foo\n",
        );
    }

    #[test]
    fn print_multiple_results_with_colors() {
        assert_formatter(
            OutputFormat::TTY,
            vec![
                cf("DL2001", DLSeverity::DLInfoC, "test", 1),
                cf("DL2002", DLSeverity::DLWarningC, "foo", 3),
            ],
            false,
            "<string>:1 DL2001 \x1b[92minfo\x1b[0m: test\n<string>:3 DL2002 \x1b[1m\x1b[93mwarning\x1b[0m: foo\n",
        );
    }
}
