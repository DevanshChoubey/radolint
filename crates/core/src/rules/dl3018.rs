// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3018.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{self, ParsedShell};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3018(), onbuild(dl3018()))
}

/// Haskell `dl3018`.
fn dl3018() -> Box<dyn Rule> {
    let code = "DL3018";
    let severity = DLSeverity::DLWarningC;
    let message =
        "Pin versions in apk add. Instead of `apk add <package>` use `apk add <package>=<version>`";
    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => fold_arguments(
            |a: &ParsedShell| {
                apk_add_packages(a)
                    .iter()
                    .all(|p| version_fixed(p) || package_file(p))
            },
            args,
        ),
        _ => true,
    };
    fn version_fixed(package: &str) -> bool {
        ["=", "~", ">", "<"].iter().any(|s| package.contains(s))
    }
    fn package_file(package: &str) -> bool {
        package.ends_with(".apk")
    }
    simple_rule(code, severity, message, check)
}

/// Haskell `apkAddPackages`.
fn apk_add_packages(args: &ParsedShell) -> Vec<String> {
    let drop_target =
        |cmd: &shell::Command| shell::drop_flag_arg(&["t", "virtual", "repository", "X"], cmd);
    let mut out = Vec::new();
    for cmd in &args.present_commands {
        if shell::cmd_has_args("apk", &["add"], cmd) {
            for arg in shell::get_args_no_flags(&drop_target(cmd)) {
                if arg != "add" {
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
    fn apk_add_version_pinning_single() {
        rule_catches("DL3018", "RUN apk add flex");
        on_build_rule_catches("DL3018", "RUN apk add flex");
    }

    #[test]
    fn apk_add_no_version_pinning_single() {
        rule_catches_not("DL3018", "RUN apk add flex=2.6.4-r1");
        on_build_rule_catches_not("DL3018", "RUN apk add flex=2.6.4-r1");
    }

    #[test]
    fn apk_add_tilde_version_pinning_single() {
        rule_catches_not("DL3018", "RUN apk add --no-cache git~2.52.0");
        on_build_rule_catches_not("DL3018", "RUN apk add --no-cache git~2.52.0");
    }

    #[test]
    fn apk_add_tilde_eq_version_pinning() {
        rule_catches_not("DL3018", "RUN apk add --no-cache git~=2.52");
        on_build_rule_catches_not("DL3018", "RUN apk add --no-cache git~=2.52");
        rule_catches_not("DL3018", "RUN apk add --no-cache git=~2.52");
        on_build_rule_catches_not("DL3018", "RUN apk add --no-cache git=~2.52");
    }

    #[test]
    fn apk_add_version_pinned_chained() {
        let docker_file = [
            "RUN apk add --no-cache flex=2.6.4-r1 \\",
            " && pip install -r requirements.txt",
        ]
        .join("\n")
            + "\n";
        rule_catches_not("DL3018", &docker_file);
        on_build_rule_catches_not("DL3018", &docker_file);
    }

    #[test]
    fn apk_add_version_pinned_regression() {
        let docker_file = [
            "RUN apk add --no-cache \\",
            "flex=2.6.4-r1 \\",
            "libffi=3.2.1-r3 \\",
            "python2=2.7.13-r1 \\",
            "libbz2=1.0.6-r5",
        ]
        .join("\n")
            + "\n";
        rule_catches_not("DL3018", &docker_file);
        on_build_rule_catches_not("DL3018", &docker_file);
    }

    #[test]
    fn apk_add_version_pinned_with_fuzzy_versions() {
        let docker_file = [
            "RUN apk add --no-cache \\",
            "  \"flex>=2.6.1-r1\" \\",
            "  \"libffi~3.2.1\" \\",
            "  \"python3<3.6.12-r2\" \\",
            "  \"libbz2=~1.0.6-r4\"",
        ]
        .join("\n")
            + "\n";
        rule_catches_not("DL3018", &docker_file);
        on_build_rule_catches_not("DL3018", &docker_file);
    }

    #[test]
    fn apk_add_version_pinned_regression_one_missed() {
        let docker_file = [
            "RUN apk add --no-cache \\",
            "flex=2.6.4-r1 \\",
            "libffi \\",
            "python2=2.7.13-r1 \\",
            "libbz2=1.0.6-r5",
        ]
        .join("\n")
            + "\n";
        rule_catches("DL3018", &docker_file);
        on_build_rule_catches("DL3018", &docker_file);
    }

    #[test]
    fn apk_add_virtual_package() {
        let docker_file = [
            "RUN apk add \\",
            "--virtual build-dependencies \\",
            "python-dev=1.1.1 build-base=2.2.2 wget=3.3.3 \\",
            "&& pip install -r requirements.txt \\",
            "&& python setup.py install \\",
            "&& apk del build-dependencies",
        ]
        .join("\n")
            + "\n";
        rule_catches_not("DL3018", &docker_file);
        on_build_rule_catches_not("DL3018", &docker_file);
    }

    #[test]
    fn apk_add_with_repository_without_equal_sign() {
        let docker_file = [
            "RUN apk add --no-cache \\",
            "--repository https://nl.alpinelinux.org/alpine/edge/testing \\",
            "flow=0.78.0-r0",
        ]
        .join("\n")
            + "\n";
        rule_catches_not("DL3018", &docker_file);
        on_build_rule_catches_not("DL3018", &docker_file);
    }

    #[test]
    fn apk_add_with_repository_with_equal_sign() {
        let docker_file = [
            "RUN apk add --no-cache \\",
            "--repository=https://nl.alpinelinux.org/alpine/edge/testing \\",
            "flow=0.78.0-r0",
        ]
        .join("\n")
            + "\n";
        rule_catches_not("DL3018", &docker_file);
        on_build_rule_catches_not("DL3018", &docker_file);
    }

    #[test]
    fn apk_add_with_repository_x_without_equal_sign() {
        let docker_file = [
            "RUN apk add --no-cache \\",
            "-X https://nl.alpinelinux.org/alpine/edge/testing \\",
            "flow=0.78.0-r0",
        ]
        .join("\n")
            + "\n";
        rule_catches_not("DL3018", &docker_file);
        on_build_rule_catches_not("DL3018", &docker_file);
    }

    #[test]
    fn dont_trigger_when_installing_from_apk_file() {
        rule_catches_not("DL3018", "RUN apk add mypackage-1.1.1.apk");
        on_build_rule_catches_not("DL3018", "RUN apk add mypackage-1.1.1.apk");
    }
}
