// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3027.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::using_program;
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3027(), onbuild(dl3027()))
}

/// Haskell `dl3027`.
pub fn dl3027() -> Box<dyn Rule> {
    let code = "DL3027";
    let severity = DLSeverity::DLWarningC;
    let message = "Do not use apt as it is meant to be an end-user tool, use apt-get \
                   or apt-cache instead";
    let check = |instr: &Instruction<_>| match instr {
        Instruction::Run(RunArgs(args, _)) => fold_arguments(|a| !using_program("apt", a), args),
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn apt() {
        let docker_file = "FROM ubuntu\nRUN apt install python";
        rule_catches("DL3027", docker_file);
        on_build_rule_catches("DL3027", docker_file);
    }
}
