// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL4005.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{ParsedShell, cmd_has_args, no_commands};
use radolint_syntax::{Instruction, RunArgs};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl4005(), onbuild(dl4005()))
}

/// Haskell `dl4005`.
fn dl4005() -> Box<dyn Rule> {
    let code = "DL4005";
    let severity = DLSeverity::DLWarningC;
    let message = "Use SHELL to change the default shell";
    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => fold_arguments(
            |a: &ParsedShell| no_commands(|c| cmd_has_args("ln", &["/bin/sh"], c), a),
            args,
        ),
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn run_ln() {
        rule_catches("DL4005", "RUN ln -sfv /bin/bash /bin/sh");
        on_build_rule_catches("DL4005", "RUN ln -sfv /bin/bash /bin/sh");
    }

    #[test]
    fn run_ln_with_unrelated_symlinks() {
        rule_catches_not("DL4005", "RUN ln -sf /bin/true /sbin/initctl");
        on_build_rule_catches_not("DL4005", "RUN ln -sf /bin/true /sbin/initctl");
    }

    #[test]
    fn run_ln_with_multiple_acceptable_commands() {
        rule_catches_not(
            "DL4005",
            "RUN ln -s foo bar && unrelated && something_with /bin/sh",
        );
        on_build_rule_catches_not(
            "DL4005",
            "RUN ln -s foo bar && unrelated && something_with /bin/sh",
        );
    }
}
