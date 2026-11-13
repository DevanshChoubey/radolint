// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3034.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{self, Command, ParsedShell};
use dlint_syntax::{Instruction, RunArgs};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3034(), onbuild(dl3034()))
}

/// Haskell `dl3034`.
fn dl3034() -> Box<dyn Rule> {
    let code = "DL3034";
    let severity = DLSeverity::DLWarningC;
    let message = "Non-interactive switch missing from `zypper` command: `zypper install -y`";

    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => {
            fold_arguments(|a| shell::no_commands(forgot_zypper_yes_option, a), args)
        }
        _ => true,
    };

    fn forgot_zypper_yes_option(cmd: &Command) -> bool {
        is_zypper_install(cmd) && !has_yes_option(cmd)
    }
    fn is_zypper_install(cmd: &Command) -> bool {
        shell::cmd_has_args(
            "zypper",
            &[
                "install",
                "in",
                "remove",
                "rm",
                "source-install",
                "si",
                "patch",
            ],
            cmd,
        )
    }
    fn has_yes_option(cmd: &Command) -> bool {
        shell::has_any_flag(&["non-interactive", "n", "no-confirm", "y"], cmd)
    }
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn not_ok_without_non_interactive_switch() {
        rule_catches("DL3034", "RUN zypper install httpd=2.4.24 && zypper clean");
        on_build_rule_catches("DL3034", "RUN zypper install httpd=2.4.24 && zypper clean");
    }

    #[test]
    fn ok_with_non_interactive_switch_present() {
        rule_catches_not(
            "DL3034",
            "RUN zypper install -n httpd=2.4.24 && zypper clean",
        );
        rule_catches_not(
            "DL3034",
            "RUN zypper install --non-interactive httpd=2.4.24 && zypper clean",
        );
        rule_catches_not(
            "DL3034",
            "RUN zypper install -y httpd=2.4.24 && zypper clean",
        );
        rule_catches_not(
            "DL3034",
            "RUN zypper install --no-confirm httpd=2.4.24 && zypper clean",
        );
        on_build_rule_catches_not(
            "DL3034",
            "RUN zypper install -n httpd=2.4.24 && zypper clean",
        );
        on_build_rule_catches_not(
            "DL3034",
            "RUN zypper install --non-interactive httpd=2.4.24 && zypper clean",
        );
        on_build_rule_catches_not(
            "DL3034",
            "RUN zypper install -y httpd=2.4.24 && zypper clean",
        );
        on_build_rule_catches_not(
            "DL3034",
            "RUN zypper install --no-confirm httpd=2.4.24 && zypper clean",
        );
    }
}
