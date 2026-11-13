// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3035.hs.

use crate::rule::{combine, fold_arguments, onbuild, simple_rule, DLSeverity, Rule};
use crate::shell::{cmd_has_args, no_commands, ParsedShell};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3035(), onbuild(dl3035()))
}

/// Haskell `dl3035`.
fn dl3035() -> Box<dyn Rule> {
    let code = "DL3035";
    let severity = DLSeverity::DLWarningC;
    let message = "Do not use `zypper dist-upgrade`.";

    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => fold_arguments(
            |a: &ParsedShell| no_commands(|c| cmd_has_args("zypper", &["dist-upgrade", "dup"], c), a),
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
    fn not_ok_zypper_dist_upgrade() {
        rule_catches("DL3035", "RUN zypper dist-upgrade");
        rule_catches("DL3035", "RUN zypper dup");
        on_build_rule_catches("DL3035", "RUN zypper dist-upgrade");
        on_build_rule_catches("DL3035", "RUN zypper dup");
    }
}
