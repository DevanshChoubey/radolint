// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3030.hs.

use crate::rule::*;
use crate::shell;
use crate::shell::ParsedShell;
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3030(), onbuild(dl3030()))
}

/// Haskell `dl3030`.
fn dl3030() -> Box<dyn Rule> {
    let code = "DL3030";
    let severity = DLSeverity::DLWarningC;
    let message = "Use the -y switch to avoid manual input `yum install -y <package>`";

    let check = |instr: &Instruction<ParsedShell>| -> bool {
        match instr {
            Instruction::Run(RunArgs(args, _)) => {
                fold_arguments(|a| shell::no_commands(forgot_yum_yes_option, a), args)
            }
            _ => true,
        }
    };
    simple_rule(code, severity, message, check)
}

/// Haskell `forgotYumYesOption`.
fn forgot_yum_yes_option(cmd: &shell::Command) -> bool {
    is_yum_install(cmd) && !has_yes_option(cmd)
}

/// Haskell `isYumInstall`.
fn is_yum_install(cmd: &shell::Command) -> bool {
    shell::cmd_has_args("yum", &["install", "groupinstall", "localinstall"], cmd)
}

/// Haskell `hasYesOption`.
fn has_yes_option(cmd: &shell::Command) -> bool {
    shell::has_any_flag(&["y", "assumeyes"], cmd)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn not_ok_when_not_using_y_switch() {
        rule_catches("DL3030", "RUN yum install httpd-2.4.24 && yum clean all");
        on_build_rule_catches("DL3030", "RUN yum install httpd-2.4.24 && yum clean all");
    }

    #[test]
    fn ok_when_using_y_switch() {
        rule_catches_not("DL3030", "RUN yum install -y httpd-2.4.24 && yum clean all");
        rule_catches_not("DL3030", "RUN bash -c `# not even a yum command`");
        on_build_rule_catches_not("DL3030", "RUN yum install -y httpd-2.4.24 && yum clean all");
        on_build_rule_catches_not("DL3030", "RUN bash -c `# not even a yum command`");
    }
}
