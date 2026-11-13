// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3004.hs.

use crate::rule::{combine, fold_arguments, onbuild, simple_rule, DLSeverity, Rule};
use crate::shell::using_program;
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3004(), onbuild(dl3004()))
}

/// Haskell `dl3004`.
pub fn dl3004() -> Box<dyn Rule> {
    let code = "DL3004";
    let severity = DLSeverity::DLErrorC;
    let message = "Do not use sudo as it leads to unpredictable behavior. Use a tool like \
                   gosu to enforce root";
    let check = |instr: &Instruction<_>| match instr {
        Instruction::Run(RunArgs(args, _)) => fold_arguments(|a| !using_program("sudo", a), args),
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn sudo() {
        rule_catches("DL3004", "RUN sudo apt-get update");
        on_build_rule_catches("DL3004", "RUN sudo apt-get update");
    }

    #[test]
    fn install_sudo() {
        rule_catches_not("DL3004", "RUN apt-get install sudo");
        on_build_rule_catches_not("DL3004", "RUN apt-get install sudo");
    }

    #[test]
    fn sudo_chained_programs() {
        rule_catches("DL3004", "RUN apt-get update && sudo apt-get install");
        on_build_rule_catches("DL3004", "RUN apt-get update && sudo apt-get install");
    }
}
