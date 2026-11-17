// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3032.hs.

use crate::rule::*;
use crate::shell;
use crate::shell::ParsedShell;
use crate::utils;
use radolint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3032(), onbuild(dl3032()))
}

/// Haskell `dl3032`.
fn dl3032() -> Box<dyn Rule> {
    let code = "DL3032";
    let severity = DLSeverity::DLWarningC;
    let message = "`yum clean all` missing after yum command.";

    let check = |instr: &Instruction<ParsedShell>| -> bool {
        match instr {
            Instruction::Run(RunArgs(args, flags)) => {
                fold_arguments(|a| shell::no_commands(yum_install, a), args)
                    // DIVERGENCE: hadolint #1139. A cache or tmpfs mount of
                    // the yum cache keeps it out of the layer, like DL3036
                    // and DL3040 already allow for zypper and dnf.
                    || utils::has_cache_or_tmpfs_mount_with("/var/cache/yum", flags)
                    || match (
                        fold_arguments(|a| shell::find_command_index(yum_install, a), args),
                        fold_arguments(|a| shell::find_command_index(yum_clean, a), args),
                    ) {
                        (Some(i), Some(c)) => i < c,
                        _ => false,
                    }
            }
            _ => true,
        }
    };
    simple_rule(code, severity, message, check)
}

/// Haskell `yumInstall`.
fn yum_install(cmd: &shell::Command) -> bool {
    shell::cmd_has_args("yum", &["install"], cmd)
}

/// Haskell `yumClean`.
fn yum_clean(cmd: &shell::Command) -> bool {
    shell::cmd_has_args("yum", &["clean", "all"], cmd)
        || shell::cmd_has_args("rm", &["-rf", "/var/cache/yum/*"], cmd)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn not_ok_with_no_clean_all() {
        rule_catches("DL3032", "RUN yum install -y mariadb-10.4");
        on_build_rule_catches("DL3032", "RUN yum install -y mariadb-10.4");
    }
    #[test]
    fn ok_with_yum_clean_all() {
        rule_catches_not("DL3032", "RUN yum install -y mariadb-10.4 && yum clean all");
        rule_catches_not("DL3032", "RUN bash -c `# not even a yum command`");
        on_build_rule_catches_not("DL3032", "RUN yum install -y mariadb-10.4 && yum clean all");
        on_build_rule_catches_not("DL3032", "RUN bash -c `# not even a yum command`");
    }
    #[test]
    fn ok_with_rm_rf_var_cache_yum() {
        rule_catches_not(
            "DL3032",
            "RUN yum install -y mariadb-10.4 && rm -rf /var/cache/yum/*",
        );
        on_build_rule_catches_not(
            "DL3032",
            "RUN yum install -y mariadb-10.4 && rm -rf /var/cache/yum/*",
        );
    }
    #[test]
    fn not_ok_with_clean_before_install() {
        rule_catches_not("DL3032", "RUN yum install -y mariadb-10.4 && yum clean all");
        rule_catches("DL3032", "RUN yum clean all && yum install -y");
    }

    // DIVERGENCE: hadolint #1139.
    #[test]
    fn ok_with_cache_mount_at_var_cache_yum() {
        rule_catches_not(
            "DL3032",
            "RUN --mount=type=cache,target=/var/cache/yum yum install -y mariadb-10.4",
        );
        rule_catches_not(
            "DL3032",
            "RUN --mount=type=tmpfs,target=/var/cache/yum yum install -y mariadb-10.4",
        );
        on_build_rule_catches_not(
            "DL3032",
            "RUN --mount=type=cache,target=/var/cache/yum yum install -y mariadb-10.4",
        );
        rule_catches(
            "DL3032",
            "RUN --mount=type=cache,target=/var/cache/apt yum install -y mariadb-10.4",
        );
    }
}
