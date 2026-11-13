// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3038.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{self, Command, ParsedShell};
use radolint_syntax::{Instruction, RunArgs};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3038(), onbuild(dl3038()))
}

/// Haskell `dl3038`.
fn dl3038() -> Box<dyn Rule> {
    let code = "DL3038";
    let severity = DLSeverity::DLWarningC;
    let message = "Use the -y switch to avoid manual input `dnf install -y <package>`";

    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => {
            fold_arguments(|a| shell::no_commands(forgot_dnf_yes_option, a), args)
        }
        _ => true,
    };

    fn forgot_dnf_yes_option(cmd: &Command) -> bool {
        is_dnf_install(cmd) && !has_yes_option(cmd)
    }
    fn is_dnf_install(cmd: &Command) -> bool {
        shell::cmds_have_args(
            &["dnf", "microdnf"],
            &["install", "groupinstall", "localinstall"],
            cmd,
        )
    }
    fn has_yes_option(cmd: &Command) -> bool {
        shell::has_any_flag(&["y", "assumeyes"], cmd)
    }
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn not_ok_without_dnf_non_interactive_flag() {
        rule_catches("DL3038", "RUN dnf install httpd-2.4.24 && dnf clean all");
        rule_catches(
            "DL3038",
            "RUN microdnf install httpd-2.4.24 && microdnf clean all",
        );
        on_build_rule_catches("DL3038", "RUN dnf install httpd-2.4.24 && dnf clean all");
        on_build_rule_catches(
            "DL3038",
            "RUN microdnf install httpd-2.4.24 && microdnf clean all",
        );
    }

    #[test]
    fn ok_with_dnf_non_interactive_flag() {
        rule_catches_not("DL3038", "RUN dnf install -y httpd-2.4.24 && dnf clean all");
        rule_catches_not(
            "DL3038",
            "RUN microdnf install -y httpd-2.4.24 && microdnf clean all",
        );
        rule_catches_not("DL3038", "RUN notdnf install httpd");
        on_build_rule_catches_not("DL3038", "RUN dnf install -y httpd-2.4.24 && dnf clean all");
        on_build_rule_catches_not(
            "DL3038",
            "RUN microdnf install -y httpd-2.4.24 && microdnf clean all",
        );
        on_build_rule_catches_not("DL3038", "RUN notdnf install httpd");
    }
}
