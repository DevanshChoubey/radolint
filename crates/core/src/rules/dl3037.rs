// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3037.hs.

use crate::rule::{combine, fold_arguments, onbuild, simple_rule, DLSeverity, Rule};
use crate::shell::{self, ParsedShell};
use dlint_syntax::{Instruction, RunArgs};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3037(), onbuild(dl3037()))
}

/// Haskell `dl3037`.
fn dl3037() -> Box<dyn Rule> {
    let code = "DL3037";
    let severity = DLSeverity::DLWarningC;
    let message = "Specify version with `zypper install -y <package>=<version>`.";

    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => {
            fold_arguments(|a| zypper_packages(a).iter().all(|p| version_fixed(p)), args)
        }
        _ => true,
    };

    fn version_fixed(package: &str) -> bool {
        package.contains('=')
            || package.contains(">=")
            || package.contains('>')
            || package.contains("<=")
            || package.contains('<')
            || package.ends_with(".rpm")
    }
    simple_rule(code, severity, message, check)
}

/// Haskell `zypperPackages`.
fn zypper_packages(args: &ParsedShell) -> Vec<String> {
    let mut out = Vec::new();
    for cmd in &args.present_commands {
        if shell::cmd_has_args("zypper", &["install", "in"], cmd) {
            for arg in shell::get_args_no_flags(cmd) {
                if arg != "install" && arg != "in" {
                    out.push(arg);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn not_ok_without_zypper_version_pinning() {
        rule_catches("DL3037", "RUN zypper install -y tomcat && zypper clean");
        on_build_rule_catches("DL3037", "RUN zypper install -y tomcat && zypper clean");
    }

    #[test]
    fn ok_with_different_variants_of_zypper_version_pinning() {
        for p in [
            "tomcat=9.0.39",
            "tomcat\\>=9.0",
            "tomcat\\>9.0",
            "tomcat\\<=9.0",
            "tomcat\\<9.0",
            "tomcat-9.0.39-1.rpm",
        ] {
            let df = format!("RUN zypper install -y {} && zypper clean", p);
            rule_catches_not("DL3037", &df);
            on_build_rule_catches_not("DL3037", &df);
        }
    }
}
