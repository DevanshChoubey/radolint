// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3025.hs.

use crate::rule::*;
use crate::shell::ParsedShell;
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3025";
    let severity = DLSeverity::DLWarningC;
    let message = "Use arguments JSON notation for CMD and ENTRYPOINT arguments";

    let check = |instr: &Instruction<ParsedShell>| -> bool {
        match instr {
            Instruction::Cmd(Arguments::ArgumentsText(_)) => false,
            Instruction::Entrypoint(Arguments::ArgumentsText(_)) => false,
            Instruction::Healthcheck(Check::Check(CheckArgs {
                check_command: Arguments::ArgumentsText(_),
                ..
            })) => false,
            _ => true,
        }
    };
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn warn_on_entrypoint() {
        rule_catches("DL3025", "FROM node as foo\nENTRYPOINT something\n");
    }
    #[test]
    fn dont_warn_on_entrypoint_json_notation() {
        rule_catches_not("DL3025", "FROM scratch as build\nENTRYPOINT [\"foo\", \"bar\"]\n");
    }
    #[test]
    fn warn_on_cmd() {
        rule_catches("DL3025", "FROM node as foo\nCMD something\n");
    }
    #[test]
    fn dont_warn_on_cmd_json_notation() {
        rule_catches_not(
            "DL3025",
            "FROM scratch as build\nCMD [\"foo\", \"bar\"]\nCMD [ \"foo\", \"bar\" ]\n",
        );
    }
    #[test]
    fn warn_on_healthcheck_cmd_shell_notation() {
        rule_catches("DL3025", "HEALTHCHECK CMD foobar\n");
    }
    #[test]
    fn dont_warn_on_healthcheck_cmd_json_notation() {
        rule_catches_not("DL3025", "HEALTHCHECK CMD [ \"foobar\" ]\n");
    }
    #[test]
    fn dont_warn_on_cmd_json_notation_with_broken_long_strings() {
        rule_catches_not(
            "DL3025",
            "CMD [ \"/bin/sh\", \"-c\", \\\n      \"echo foo && \\\n       echo bar\" \\\n    ]\n",
        );
    }
}
