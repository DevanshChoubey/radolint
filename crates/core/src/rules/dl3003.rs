// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3003.hs.

use crate::rule::{fold_arguments, simple_rule, DLSeverity, Rule};
use crate::shell::using_program;
use dlint_syntax::{Instruction, RunArgs};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3003";
    let severity = DLSeverity::DLWarningC;
    let message = "Use WORKDIR to switch to a directory";
    let check = |instr: &Instruction<crate::shell::ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => fold_arguments(|a| !using_program("cd", a), args),
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn ok_using_workdir() {
        rule_catches_not("DL3003", "WORKDIR /opt");
    }

    #[test]
    fn not_ok_using_cd() {
        rule_catches("DL3003", "RUN cd /opt");
    }
}
