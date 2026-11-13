// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3014.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{self, Command, ParsedShell};
use radolint_syntax::{Instruction, RunArgs};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3014(), onbuild(dl3014()))
}

/// Haskell `dl3014`.
fn dl3014() -> Box<dyn Rule> {
    let code = "DL3014";
    let severity = DLSeverity::DLWarningC;
    let message = "Use the `-y` switch to avoid manual input `apt-get -y install <package>`";

    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => {
            fold_arguments(|a| shell::no_commands(forgot_apt_yes_option, a), args)
        }
        _ => true,
    };

    fn forgot_apt_yes_option(cmd: &Command) -> bool {
        is_apt_get_install(cmd) && !has_yes_option(cmd)
    }
    fn is_apt_get_install(cmd: &Command) -> bool {
        shell::cmd_has_args("apt-get", &["install"], cmd)
    }
    fn has_yes_option(cmd: &Command) -> bool {
        shell::has_any_flag(&["y", "yes", "qq", "assume-yes"], cmd)
            || shell::count_flag("q", cmd) == 2
            || shell::count_flag("quiet", cmd) == 2
            || shell::get_args(cmd).iter().any(|a| a == "-q=2")
    }
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn apt_get_auto_yes() {
        rule_catches("DL3014", "RUN apt-get install python");
        on_build_rule_catches("DL3014", "RUN apt-get install python");
    }

    #[test]
    fn apt_get_q() {
        rule_catches("DL3014", "RUN apt-get install -q python");
        on_build_rule_catches("DL3014", "RUN apt-get install -q python");
    }

    #[test]
    fn apt_get_quiet() {
        rule_catches("DL3014", "RUN apt-get install --quiet python");
        on_build_rule_catches("DL3014", "RUN apt-get install -q python");
    }

    #[test]
    fn apt_get_yes_shortflag() {
        rule_catches_not("DL3014", "RUN apt-get install -yq python");
        on_build_rule_catches_not("DL3014", "RUN apt-get install -yq python");
    }

    #[test]
    fn apt_get_yes_different_pos() {
        rule_catches_not("DL3014", "RUN apt-get install -y python");
        on_build_rule_catches_not("DL3014", "RUN apt-get install -y python");
    }

    #[test]
    fn apt_get_with_auto_yes() {
        rule_catches_not("DL3014", "RUN apt-get -y install python");
        on_build_rule_catches_not("DL3014", "RUN apt-get -y install python");
    }

    #[test]
    fn apt_get_with_auto_expanded_yes() {
        rule_catches_not("DL3014", "RUN apt-get --yes install python");
        on_build_rule_catches_not("DL3014", "RUN apt-get --yes install python");
    }

    #[test]
    fn apt_get_with_assume_yes() {
        rule_catches_not("DL3014", "RUN apt-get --assume-yes install python");
        on_build_rule_catches_not("DL3014", "RUN apt-get --assume-yes install python");
    }

    #[test]
    fn apt_get_qq() {
        rule_catches_not("DL3014", "RUN apt-get install -qq python");
        on_build_rule_catches_not("DL3014", "RUN apt-get install -qq python");
    }

    #[test]
    fn apt_get_q_q() {
        rule_catches_not("DL3014", "RUN apt-get install -q -q python");
        on_build_rule_catches_not("DL3014", "RUN apt-get install -q -q python");
    }

    #[test]
    fn apt_get_q_eq_2() {
        rule_catches_not("DL3014", "RUN apt-get install -q=2 python");
        on_build_rule_catches_not("DL3014", "RUN apt-get install -q=2 python");
    }

    #[test]
    fn apt_get_quiet_quiet() {
        rule_catches_not("DL3014", "RUN apt-get install --quiet --quiet python");
        on_build_rule_catches_not("DL3014", "RUN apt-get install --quiet --quiet python");
    }
}
