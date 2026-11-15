// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3003.hs.

use crate::rule::{DLSeverity, Rule, fold_arguments, simple_rule};
use crate::shell::using_program_outside_subshell;
use radolint_syntax::{Instruction, RunArgs};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3003";
    let severity = DLSeverity::DLWarningC;
    let message = "Use WORKDIR to switch to a directory";
    let check = |instr: &Instruction<crate::shell::ParsedShell>| match instr {
        // DIVERGENCE: hadolint flags every `cd`, even in a `( ... )` subshell, which is the
        // escape hatch its own DL3003 documentation suggests (hadolint/hadolint#422). A `cd` in a
        // subshell does not move the commands after it, so it is not reported here.
        Instruction::Run(RunArgs(args, _)) => {
            fold_arguments(|a| !using_program_outside_subshell("cd", a), args)
        }
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

    #[test]
    fn ok_using_cd_in_a_subshell() {
        rule_catches_not(
            "DL3003",
            "RUN git clone x /tmp/jo && (cd /tmp/jo && make) && rm -rf /tmp/jo",
        );
    }

    #[test]
    fn not_ok_using_cd_outside_the_subshell() {
        rule_catches("DL3003", "RUN (make) && cd /opt && make");
    }

    #[test]
    fn not_ok_using_cd_in_a_brace_group() {
        rule_catches("DL3003", "RUN { cd /opt && make; }");
    }
}
