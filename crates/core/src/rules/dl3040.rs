// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3040.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{self, ParsedShell};
use crate::utils;
use radolint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3040(), onbuild(dl3040()))
}

const DNF_CMDS: [&str; 2] = ["dnf", "microdnf"];
const INSTALL_CMDS: [&str; 8] = [
    "install",
    "in",
    "upgrade",
    "up",
    "upgrade-minimal",
    "up-min",
    "reinstall",
    "rei",
];

/// Haskell `dl3040`.
fn dl3040() -> Box<dyn Rule> {
    let code = "DL3040";
    let severity = DLSeverity::DLWarningC;
    let message = "`dnf clean all` missing after dnf command.";

    let dnf_install =
        |cmd_name: &str, args: &ParsedShell| -> Box<dyn Fn(&shell::Command) -> bool> {
            let _ = args;
            let cmd_name = cmd_name.to_string();
            Box::new(move |c: &shell::Command| {
                DNF_CMDS.contains(&cmd_name.as_str())
                    && shell::cmd_has_args(&cmd_name, &INSTALL_CMDS, c)
            })
        };
    let dnf_clean = |cmd_name: &str| -> Box<dyn Fn(&shell::Command) -> bool> {
        let cmd_name = cmd_name.to_string();
        Box::new(move |c: &shell::Command| {
            shell::cmd_has_args(&cmd_name, &["clean", "all"], c)
                || shell::cmd_has_args("rm", &["-rf", "/var/cache/libdnf5*"], c)
        })
    };
    let check_missing_clean = move |args: &Arguments<ParsedShell>, cmd_name: &str| -> bool {
        fold_arguments(
            |a: &ParsedShell| shell::no_commands(dnf_install(cmd_name, a), a),
            args,
        ) || Some(true)
            == (|| {
                let i = fold_arguments(
                    |a: &ParsedShell| shell::find_command_index(dnf_install(cmd_name, a), a),
                    args,
                )?;
                let j = fold_arguments(
                    |a: &ParsedShell| shell::find_command_index(dnf_clean(cmd_name), a),
                    args,
                )?;
                Some(i < j)
            })()
    };
    let check = move |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, flags)) => {
            if DNF_CMDS.iter().all(|c| check_missing_clean(args, c)) {
                true
            } else if utils::has_cache_or_tmpfs_mount_with("/var/cache/libdnf5", flags) {
                true
            } else {
                utils::has_cache_or_tmpfs_mount_with(".cache/libdnf5", flags)
            }
        }
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn no_ok_without_dnf_clean_all() {
        rule_catches("DL3040", "RUN dnf install -y mariadb-10.4");
        rule_catches("DL3040", "RUN microdnf install -y mariadb-10.4");
        rule_catches("DL3040", "RUN dnf in -y mariadb-10.4");
        on_build_rule_catches("DL3040", "RUN dnf install -y mariadb-10.4");
        on_build_rule_catches("DL3040", "RUN dnf in -y mariadb-10.4");
        on_build_rule_catches(
            "DL3040",
            "RUN dnf install -y mariadb-10.4 && microdnf clean all",
        );
        on_build_rule_catches("DL3040", "RUN microdnf install -y mariadb-10.4");
    }

    #[test]
    fn ok_with_dnf_clean_all() {
        rule_catches_not("DL3040", "RUN dnf install -y mariadb-10.4 && dnf clean all");
        rule_catches_not("DL3040", "RUN dnf in -y mariadb-10.4 && dnf clean all");
        rule_catches_not(
            "DL3040",
            "RUN microdnf install -y mariadb-10.4 && microdnf clean all",
        );
        rule_catches_not("DL3040", "RUN notdnf install mariadb");
        on_build_rule_catches_not("DL3040", "RUN dnf install -y mariadb-10.4 && dnf clean all");
        on_build_rule_catches_not(
            "DL3040",
            "RUN microdnf install -y mariadb-10.4 && microdnf clean all",
        );
        on_build_rule_catches_not("DL3040", "RUN notdnf install mariadb");
    }

    #[test]
    fn ok_with_rm_var_cache_yum() {
        rule_catches_not(
            "DL3040",
            "RUN dnf install -y mariadb-10.4 && rm -rf /var/cache/yum/*",
        );
        rule_catches_not(
            "DL3040",
            "RUN dnf in -y mariadb-10.4 && rm -rf /var/cache/yum/*",
        );
        rule_catches_not(
            "DL3040",
            "RUN microdnf install -y mariadb-10.4 && rm -rf /var/cache/yum/*",
        );
        on_build_rule_catches_not(
            "DL3040",
            "RUN dnf install -y mariadb-10.4 && rm -rf /var/cache/yum/*",
        );
        on_build_rule_catches_not(
            "DL3040",
            "RUN dnf in -y mariadb-10.4 && rm -rf /var/cache/yum/*",
        );
        on_build_rule_catches_not(
            "DL3040",
            "RUN microdnf install -y mariadb-10.4 && rm -rf /var/cache/yum/*",
        );
    }

    #[test]
    fn not_ok_with_clean_before_install() {
        rule_catches(
            "DL3040",
            "RUN microdnf clean all && dnf install -y mariadb-10.4",
        );
        rule_catches("DL3040", "RUN microdnf clean all && dnf in -y mariadb-10.4");
        rule_catches(
            "DL3040",
            "RUN rm -rf /var/cache/libdnf5 && dnf install -y mariadb-10.4",
        );
        rule_catches(
            "DL3040",
            "RUN rm -rf /var/cache/libdnf5 && dnf in -y mariadb-10.4",
        );
        rule_catches(
            "DL3040",
            "RUN rm -rf /var/cache/libdnf5 && microdnf install -y mariadb-10.4",
        );
        on_build_rule_catches(
            "DL3040",
            "RUN rm -rf /var/cache/libdnf5 && dnf install -y mariadb-10.4",
        );
        on_build_rule_catches(
            "DL3040",
            "RUN rm -rf /var/cache/libdnf5 && dnf in -y mariadb-10.4",
        );
        on_build_rule_catches(
            "DL3040",
            "RUN rm -rf /var/cache/libdnf5 && microdnf install -y mariadb-10.4",
        );
    }

    #[test]
    fn ok_with_cache_mount_at_var_cache_yum() {
        rule_catches_not(
            "DL3040",
            "RUN --mount=type=cache,target=/var/cache/libdnf5 dnf install -y mariadb-10.4",
        );
        rule_catches_not(
            "DL3040",
            "RUN --mount=type=cache,target=/var/cache/libdnf5 dnf in -y mariadb-10.4",
        );
        rule_catches_not(
            "DL3040",
            "RUN --mount=type=cache,target=/var/cache/libdnf5 microdnf install -y mariadb-10.4",
        );
        on_build_rule_catches_not(
            "DL3040",
            "RUN --mount=type=cache,target=/var/cache/libdnf5 dnf install -y mariadb-10.4",
        );
        on_build_rule_catches_not(
            "DL3040",
            "RUN --mount=type=cache,target=/var/cache/libdnf5 dnf in -y mariadb-10.4",
        );
        on_build_rule_catches_not(
            "DL3040",
            "RUN --mount=type=cache,target=/var/cache/libdnf5 microdnf install -y mariadb-10.4",
        );
    }

    #[test]
    fn ok_with_tmpfs_mount_at_var_cache_yum() {
        rule_catches_not(
            "DL3040",
            "RUN --mount=type=tmpfs,target=/var/cache/libdnf5 dnf install -y mariadb-10.4",
        );
        rule_catches_not(
            "DL3040",
            "RUN --mount=type=tmpfs,target=/var/cache/libdnf5 dnf in -y mariadb-10.4",
        );
        rule_catches_not(
            "DL3040",
            "RUN --mount=type=tmpfs,target=/var/cache/libdnf5 microdnf install -y mariadb-10.4",
        );
        on_build_rule_catches_not(
            "DL3040",
            "RUN --mount=type=tmpfs,target=/var/cache/libdnf5 dnf install -y mariadb-10.4",
        );
        on_build_rule_catches_not(
            "DL3040",
            "RUN --mount=type=tmpfs,target=/var/cache/libdnf5 dnf in -y mariadb-10.4",
        );
        on_build_rule_catches_not(
            "DL3040",
            "RUN --mount=type=tmpfs,target=/var/cache/libdnf5 microdnf install -y mariadb-10.4",
        );
    }

    #[test]
    fn not_ok_with_dnf_upgrade() {
        rule_catches("DL3040", "RUN dnf -y upgrade");
        rule_catches("DL3040", "RUN dnf -y up");
        on_build_rule_catches("DL3040", "RUN dnf -y upgrade");
        on_build_rule_catches("DL3040", "RUN dnf -y up");
    }

    #[test]
    fn different_install_command_variants() {
        rule_catches("DL3040", "RUN dnf -y install");
        rule_catches("DL3040", "RUN dnf -y in");
        rule_catches("DL3040", "RUN dnf -y upgrade");
        rule_catches("DL3040", "RUN dnf -y up");
        rule_catches("DL3040", "RUN dnf -y upgrade-minimal");
        rule_catches("DL3040", "RUN dnf -y up-min");
        rule_catches("DL3040", "RUN dnf -y reinstall");
        rule_catches("DL3040", "RUN dnf -y rei");
        rule_catches_not("DL3040", "RUN dnf -y install && dnf clean all");
        rule_catches_not("DL3040", "RUN dnf -y in && dnf clean all");
        rule_catches_not("DL3040", "RUN dnf -y upgrade && dnf clean all");
        rule_catches_not("DL3040", "RUN dnf -y up && dnf clean all");
        rule_catches_not("DL3040", "RUN dnf -y upgrade-minimal && dnf clean all");
        rule_catches_not("DL3040", "RUN dnf -y up-min && dnf clean all");
        rule_catches_not("DL3040", "RUN dnf -y reinstall && dnf clean all");
        rule_catches_not("DL3040", "RUN dnf -y rei && dnf clean all");
    }
}
