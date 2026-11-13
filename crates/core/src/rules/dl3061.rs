// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3061.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, add_fail, custom_rule, empty_state, replace_with,
};
use dlint_syntax::Instruction;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3061";
    let severity = DLSeverity::DLErrorC;
    let message = "Invalid instruction order. Dockerfile must begin with `FROM`, `ARG` or comment.";

    let check = move |line, st: crate::rule::State<bool>, instr: &Instruction<_>| match instr {
        Instruction::From(_) => replace_with(true, st),
        Instruction::Comment(_) => st,
        Instruction::Pragma(_) => st,
        Instruction::Arg(_, _) => st,
        _ => {
            if st.state {
                st
            } else {
                add_fail(
                    CheckFailure {
                        code: RuleCode::from(code),
                        severity,
                        message: message.to_string(),
                        line,
                    },
                    st,
                )
            }
        }
    };
    custom_rule(check, empty_state(false))
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn warn_label_before_from() {
        rule_catches("DL3061", "LABEL foo=bar\nFROM foo");
        on_build_rule_catches("DL3061", "LABEL foo=bar\nFROM foo");
    }

    #[test]
    fn dont_warn_from_before_label() {
        rule_catches_not("DL3061", "FROM foo\nLABEL foo=bar");
        on_build_rule_catches_not("DL3061", "FROM foo\nLABEL foo=bar");
    }

    #[test]
    fn dont_warn_arg_then_from_then_label() {
        rule_catches_not("DL3061", "ARG A=B\nFROM foo\nLABEL foo=bar");
        on_build_rule_catches_not("DL3061", "ARG A=B\nFROM foo\nLABEL foo=bar");
    }

    #[test]
    fn dont_warn_pragma_before_from() {
        let d = "# syntax = docker/dockerfile:1.0-experimental\nFROM node:16-alpine3.13";
        rule_catches_not("DL3061", d);
        on_build_rule_catches_not("DL3061", d);
    }

    #[test]
    fn dont_warn_from_then_arg_then_run() {
        rule_catches_not("DL3061", "FROM foo\nARG A=B\nRUN echo bla");
    }
}
