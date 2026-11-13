// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3046.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{self, Command};
use radolint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3046(), onbuild(dl3046()))
}

/// Haskell `dl3046`.
pub fn dl3046() -> Box<dyn Rule> {
    let code = "DL3046";
    let severity = DLSeverity::DLWarningC;
    let message =
        "`useradd` without flag `-l` and high UID will result in excessively large Image.";

    let check = |instr: &Instruction<_>| match instr {
        Instruction::Run(RunArgs(args, _)) => {
            fold_arguments(|a| shell::no_commands(forgot_flag_l, a), args)
        }
        _ => true,
    };

    fn forgot_flag_l(cmd: &Command) -> bool {
        is_useradd(cmd) && (!has_l_flag(cmd) && has_u_flag(cmd) && has_long_uid(cmd))
    }
    fn is_useradd(cmd: &Command) -> bool {
        cmd.name == "useradd"
    }
    fn has_l_flag(cmd: &Command) -> bool {
        shell::has_any_flag(&["l", "no-log-init"], cmd)
    }
    fn has_u_flag(cmd: &Command) -> bool {
        shell::has_any_flag(&["u", "uid"], cmd)
    }
    fn has_long_uid(cmd: &Command) -> bool {
        shell::get_flag_arg("u", cmd)
            .iter()
            .any(|a| a.chars().count() > 5)
    }

    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn ok_with_useradd_alone() {
        rule_catches_not("DL3046", "RUN useradd luser");
    }

    #[test]
    fn ok_with_useradd_short_uid() {
        rule_catches_not("DL3046", "RUN useradd -u 12345 luser");
    }

    #[test]
    fn ok_with_useradd_long_uid_and_flag_l() {
        rule_catches_not("DL3046", "RUN useradd -l -u 123456 luser");
    }

    #[test]
    fn ok_with_useradd_and_just_flag_l() {
        rule_catches_not("DL3046", "RUN useradd -l luser");
    }

    #[test]
    fn warn_when_useradd_and_long_uid_without_flag_l() {
        rule_catches("DL3046", "RUN useradd -u 123456 luser");
    }
}
