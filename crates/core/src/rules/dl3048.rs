// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3048.hs.

use crate::rule::{simple_rule, DLSeverity, Rule};
use dlint_syntax::Instruction;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3048";
    let severity = DLSeverity::DLStyleC;
    let message = "Invalid label key.";
    let check = |instr: &Instruction<_>| match instr {
        Instruction::Label(pairs) => has_no_invalid_key(pairs),
        _ => true,
    };

    fn has_no_invalid_key(prs: &[(String, String)]) -> bool {
        !prs.iter().any(|(l, _v)| {
            !l.chars().next().map_or(false, |c| c.is_ascii_lowercase())
                || !l.chars().last().map_or(false, |c| c.is_ascii_lowercase() || c.is_ascii_digit())
                || contains_illegal_char(l)
                || has_reserved_namespace(l)
                || has_consecutive_separators(l)
        })
    }

    simple_rule(code, severity, message, check)
}

/// Haskell `containsIllegalChar`.
fn contains_illegal_char(l: &str) -> bool {
    l.chars().any(|c| !valid_chars(c))
}

/// Haskell `hasReservedNamespace`.
fn has_reserved_namespace(l: &str) -> bool {
    l.starts_with("com.docker.") || l.starts_with("io.docker.") || l.starts_with("org.dockerproject.")
}

/// Haskell `hasConsecutiveSeparators`.
fn has_consecutive_separators(l: &str) -> bool {
    l.contains("..") || l.contains("--")
}

/// Haskell `validChars` (membership test).
fn valid_chars(c: char) -> bool {
    c.is_ascii_digit() || c.is_ascii_lowercase() || matches!(c, '.' | '-' | '_' | '/')
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn not_ok_with_reserved_namespace() {
        rule_catches("DL3048", "LABEL com.docker.label=\"foo\"");
        rule_catches("DL3048", "LABEL io.docker.label=\"foo\"");
        rule_catches("DL3048", "LABEL org.dockerproject.label=\"foo\"");
        on_build_rule_catches("DL3048", "LABEL com.docker.label=\"foo\"");
        on_build_rule_catches("DL3048", "LABEL io.docker.label=\"foo\"");
        on_build_rule_catches("DL3048", "LABEL org.dockerproject.label=\"foo\"");
    }

    #[test]
    fn not_ok_with_invalid_character() {
        rule_catches("DL3048", "LABEL invalid$character=\"foo\"");
        on_build_rule_catches("DL3048", "LABEL invalid$character=\"foo\"");
    }

    #[test]
    fn not_ok_with_invalid_start_and_end_characters() {
        rule_catches("DL3048", "LABEL .invalid =\"foo\"");
        rule_catches("DL3048", "LABEL -invalid =\"foo\"");
        rule_catches("DL3048", "LABEL 1invalid =\"foo\"");
        on_build_rule_catches("DL3048", "LABEL .invalid=\"foo\"");
        on_build_rule_catches("DL3048", "LABEL -invalid=\"foo\"");
        on_build_rule_catches("DL3048", "LABEL 1invalid=\"foo\"");
    }

    #[test]
    fn not_ok_with_consecutive_dividers() {
        rule_catches("DL3048", "LABEL invalid..character=\"foo\"");
        rule_catches("DL3048", "LABEL invalid--character=\"foo\"");
        on_build_rule_catches("DL3048", "LABEL invalid..character=\"foo\"");
        on_build_rule_catches("DL3048", "LABEL invalid--character=\"foo\"");
    }

    #[test]
    fn ok_with_valid_labels() {
        rule_catches_not("DL3048", "LABEL org.valid-key.label3=\"foo\"");
        rule_catches_not("DL3048", "LABEL validlabel=\"foo\"");
        on_build_rule_catches_not("DL3048", "LABEL org.valid-key.label3=\"foo\"");
        on_build_rule_catches_not("DL3048", "LABEL validlabel=\"foo\"");
    }
}
