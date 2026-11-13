// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/Gnu.hs.
//!
//! Hadolint GNU Format Output
//!
//! See https://www.gnu.org/prep/standards/html_node/Errors.html for reference.

use super::format::{FormatResult, error_position_pretty, severity_text, strip_newlines};
use crate::rule::CheckFailure;
use dlint_parser::ParseError;
use std::io::Write;

/// Haskell `hWrite`.
pub fn h_write(handle: &mut dyn Write, results: &[FormatResult]) -> std::io::Result<()> {
    for r in results {
        print_result(handle, r)?;
    }
    Ok(())
}

fn print_result(handle: &mut dyn Write, result: &FormatResult) -> std::io::Result<()> {
    print_errors(handle, &result.errors)?;
    print_checks(handle, &result.file_name, &result.checks)
}

fn print_errors(handle: &mut dyn Write, errors: &[ParseError]) -> std::io::Result<()> {
    for e in errors {
        writeln!(handle, "{}", format_error(e))?;
    }
    Ok(())
}

fn print_checks(
    handle: &mut dyn Write,
    filename: &str,
    checks: &[CheckFailure],
) -> std::io::Result<()> {
    for c in checks {
        writeln!(handle, "{}", format_check(filename, c))?;
    }
    Ok(())
}

fn format_error(err: &ParseError) -> String {
    format!(
        "hadolint:{}",
        strip_newlines(&format!(
            "{}: {}",
            error_position_pretty(err),
            err.first().text_pretty()
        ))
    )
}

fn format_check(source: &str, f: &CheckFailure) -> String {
    format!(
        "hadolint:{}:{}: {} {}: {}",
        source,
        f.line,
        f.code.un_rule_code,
        severity_text(f.severity),
        f.message
    )
}

#[cfg(test)]
mod tests {
    use crate::formatter::format::OutputFormat;
    use crate::rule::{CheckFailure, DLSeverity};
    use crate::testing::assert_formatter;

    fn cf(code: &str, severity: DLSeverity, message: &str, line: i32) -> CheckFailure {
        CheckFailure {
            code: code.into(),
            severity,
            message: message.to_string(),
            line: line as _,
        }
    }

    #[test]
    fn print_empty_results() {
        assert_formatter(OutputFormat::Gnu, vec![], false, "");
    }

    #[test]
    fn print_one_failed_rule() {
        let c = vec![cf("DL2001", DLSeverity::DLWarningC, "test", 1)];
        assert_formatter(
            OutputFormat::Gnu,
            c,
            false,
            "hadolint:<string>:1: DL2001 warning: test\n",
        );
    }

    #[test]
    fn print_multiple_failed_rules() {
        let c = vec![
            cf("DL2001", DLSeverity::DLWarningC, "test", 1),
            cf("DL2003", DLSeverity::DLInfoC, "test 2", 3),
        ];
        assert_formatter(
            OutputFormat::Gnu,
            c,
            false,
            "hadolint:<string>:1: DL2001 warning: test\nhadolint:<string>:3: DL2003 info: test 2\n",
        );
    }
}
