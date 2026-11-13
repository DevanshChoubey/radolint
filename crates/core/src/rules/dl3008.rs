// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3008.hs.

use crate::rule::{combine, fold_arguments, onbuild, simple_rule, DLSeverity, Rule};
use crate::shell::{self, ParsedShell};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3008(), onbuild(dl3008()))
}

/// Haskell `dl3008`.
fn dl3008() -> Box<dyn Rule> {
    let code = "DL3008";
    let severity = DLSeverity::DLWarningC;
    let message = "Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`";
    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => {
            fold_arguments(|a: &ParsedShell| apt_get_packages(a).iter().all(|p| version_fixed(p)), args)
        }
        _ => true,
    };
    fn version_fixed(package: &str) -> bool {
        package.contains('=') || (package.contains('/') || package.ends_with(".deb"))
    }
    simple_rule(code, severity, message, check)
}

/// Haskell `aptGetPackages`.
fn apt_get_packages(args: &ParsedShell) -> Vec<String> {
    let drop_target = |cmd: &shell::Command| shell::drop_flag_arg(&["t", "target-release"], cmd);
    let mut out = Vec::new();
    for cmd in &args.present_commands {
        if shell::cmd_has_args("apt-get", &["install"], cmd) {
            for arg in shell::get_args_no_flags(&drop_target(cmd)) {
                if arg != "install" {
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
    fn apt_get_version_pinning() {
        rule_catches("DL3008", "RUN apt-get update && apt-get install python");
        on_build_rule_catches("DL3008", "RUN apt-get update && apt-get install python");
    }

    #[test]
    fn apt_get_pinned_chained() {
        let docker_file = [
            "RUN apt-get update \\",
            " && apt-get -yqq --no-install-recommends install nodejs=0.10 \\",
            " && rm -rf /var/lib/apt/lists/*",
        ]
        .join("\n")
            + "\n";
        rule_catches_not("DL3008", &docker_file);
        on_build_rule_catches_not("DL3008", &docker_file);
    }

    #[test]
    fn apt_get_pinned_regression() {
        let docker_file = [
            "RUN apt-get update && apt-get install --no-install-recommends -y \\",
            "python-demjson=2.2.2* \\",
            "wget=1.16.1* \\",
            "git=1:2.5.0* \\",
            "ruby=1:2.1.*",
        ]
        .join("\n")
            + "\n";
        rule_catches_not("DL3008", &docker_file);
        on_build_rule_catches_not("DL3008", &docker_file);
    }

    #[test]
    fn apt_get_version() {
        rule_catches_not("DL3008", "RUN apt-get install -y python=1.2.2");
        on_build_rule_catches_not("DL3008", "RUN apt-get install -y python=1.2.2");
    }

    #[test]
    fn apt_get_version_deb() {
        rule_catches_not("DL3008", "RUN apt-get install ./wkhtmltox_0.12.5-1.bionic_amd64.deb");
        on_build_rule_catches_not("DL3008", "RUN apt-get install ./wkhtmltox_0.12.5-1.bionic_amd64.deb");
    }

    #[test]
    fn apt_get_pinned() {
        rule_catches_not("DL3008", "RUN apt-get -y --no-install-recommends install nodejs=0.10");
        on_build_rule_catches_not("DL3008", "RUN apt-get -y --no-install-recommends install nodejs=0.10");
    }

    #[test]
    fn apt_get_tolerate_target_release() {
        let docker_file = [
            "RUN set -e &&\\",
            " echo \"deb http://http.debian.net/debian jessie-backports main\" > /etc/apt/sources.list.d/jessie-backports.list &&\\",
            " apt-get update &&\\",
            " apt-get install -y --no-install-recommends -t jessie-backports openjdk-8-jdk=8u131-b11-1~bpo8+1 &&\\",
            " rm -rf /var/lib/apt/lists/*",
        ]
        .join("\n")
            + "\n";
        rule_catches_not("DL3008", &docker_file);
        on_build_rule_catches_not("DL3008", &docker_file);
    }
}
