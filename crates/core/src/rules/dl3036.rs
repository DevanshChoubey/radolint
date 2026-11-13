// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3036.hs.

use crate::rule::*;
use crate::shell;
use crate::shell::ParsedShell;
use crate::utils;
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3036(), onbuild(dl3036()))
}

/// Haskell `dl3036`.
fn dl3036() -> Box<dyn Rule> {
    let code = "DL3036";
    let severity = DLSeverity::DLWarningC;
    let message = "`zypper clean` missing after zypper use.";

    let check = |instr: &Instruction<ParsedShell>| -> bool {
        match instr {
            Instruction::Run(RunArgs(args, flags)) => {
                if fold_arguments(|s| shell::no_commands(zypper_install, s), args) {
                    true
                } else if utils::has_cache_or_tmpfs_mount_with("/var/cache/zypp", flags) {
                    true
                } else {
                    let i = fold_arguments(|s| shell::find_command_index(zypper_install, s), args);
                    let c = fold_arguments(|s| shell::find_command_index(zypper_clean, s), args);
                    match (i, c) {
                        (Some(i), Some(c)) => i < c,
                        _ => false,
                    }
                }
            }
            _ => true,
        }
    };

    simple_rule(code, severity, message, check)
}

fn zypper_install(cmd: &shell::Command) -> bool {
    shell::cmd_has_args("zypper", &["install", "in"], cmd)
}

fn zypper_clean(cmd: &shell::Command) -> bool {
    shell::cmd_has_args("zypper", &["clean", "cc"], cmd)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn not_ok_without_zypper_clean() {
        rule_catches("DL3036", "RUN zypper install -y mariadb=10.4");
        on_build_rule_catches("DL3036", "RUN zypper install -y mariadb=10.4");
    }

    #[test]
    fn ok_with_zypper_clean() {
        rule_catches_not("DL3036", "RUN zypper install -y mariadb=10.4 && zypper clean");
        rule_catches_not("DL3036", "RUN zypper install -y mariadb=10.4 && zypper cc");
        on_build_rule_catches_not("DL3036", "RUN zypper install -y mariadb=10.4 && zypper clean");
        on_build_rule_catches_not("DL3036", "RUN zypper install -y mariadb=10.4 && zypper cc");
    }

    #[test]
    fn not_ok_with_clean_before_install() {
        rule_catches("DL3036", "RUN zypper clean && zypper install -y mariadb=10.4");
    }

    #[test]
    fn ok_when_mount_type_cache_is_used() {
        let line = "RUN --mount=type=cache,target=/var/cache/zypp zypper install -y mariadb";
        rule_catches_not("DL3036", line);
        on_build_rule_catches_not("DL3036", line);
    }

    #[test]
    fn ok_when_mount_type_tmpfs_is_used() {
        let line = "RUN --mount=type=tmpfs,target=/var/cache/zypp zypper install -y mariadb";
        rule_catches_not("DL3036", line);
        on_build_rule_catches_not("DL3036", line);
    }
}
