// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3015.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{self, Command, ParsedShell};
use radolint_syntax::{Instruction, RunArgs};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3015(), onbuild(dl3015()))
}

/// Haskell `dl3015`.
fn dl3015() -> Box<dyn Rule> {
    let code = "DL3015";
    let severity = DLSeverity::DLInfoC;
    let message = "Avoid additional packages by specifying `--no-install-recommends`";

    let is_apt_get_install = |cmd: &Command| shell::cmd_has_args("apt-get", &["install"], cmd);
    let disables_recommend_option = |cmd: &Command| {
        shell::has_flag("no-install-recommends", cmd)
            || shell::has_arg("APT::Install-Recommends=false", cmd)
    };
    let forgot_no_install_recommends =
        move |cmd: &Command| is_apt_get_install(cmd) && !disables_recommend_option(cmd);

    let check = move |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => fold_arguments(
            |a| shell::no_commands(forgot_no_install_recommends, a),
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
    fn apt_get_install_recommends() {
        rule_catches_not(
            "DL3015",
            "RUN apt-get install --no-install-recommends python",
        );
        on_build_rule_catches_not(
            "DL3015",
            "RUN apt-get install --no-install-recommends python",
        );
    }

    #[test]
    fn apt_get_no_install_recommends() {
        rule_catches("DL3015", "RUN apt-get install python");
        on_build_rule_catches("DL3015", "RUN apt-get install python");
    }

    #[test]
    fn apt_get_no_install_recommends_y() {
        rule_catches("DL3015", "RUN apt-get -y install python");
        on_build_rule_catches("DL3015", "RUN apt-get -y install python");
    }

    #[test]
    fn apt_get_no_install_recommends_via_option() {
        rule_catches_not(
            "DL3015",
            "RUN apt-get -o APT::Install-Recommends=false install python",
        );
        on_build_rule_catches_not(
            "DL3015",
            "RUN apt-get -o APT::Install-Recommends=false install python",
        );
    }
}
