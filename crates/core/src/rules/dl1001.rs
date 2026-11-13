// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL1001.hs.

use crate::pragma::parse_ignore_pragma;
use crate::rule::{DLSeverity, Rule, simple_rule};
use radolint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL1001";
    let severity = DLSeverity::DLIgnoreC;
    let message = "Please refrain from using inline ignore pragmas  `# hadolint ignore=DLxxxx`.";
    simple_rule(code, severity, message, |instr| match instr {
        Instruction::Comment(com) => parse_ignore_pragma(com).is_none(),
        _ => true,
    })
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn catches_inline_ignore_pragma() {
        rule_catches("DL1001", "# hadolint ignore=DL3003\nRUN foo bar\n");
    }

    #[test]
    fn does_not_catch_other_pragma() {
        rule_catches_not("DL1001", "# hadolint shell=powershell\nRUN foo bar\n");
    }

    #[test]
    fn does_not_catch_other_comment() {
        rule_catches_not("DL1001", "# foobar\nRUN foo bar\n");
    }

    #[test]
    fn does_not_catch_when_inline_ignore_pragma_absent() {
        rule_catches_not("DL1001", "FROM debian\nRUN foo bar\n");
    }
}
