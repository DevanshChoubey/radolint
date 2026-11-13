// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3062.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{self, ParsedShell};
use radolint_syntax::{Instruction, RunArgs};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3062(), onbuild(dl3062()))
}

/// Haskell `dl3062`.
fn dl3062() -> Box<dyn Rule> {
    let code = "DL3062";
    let severity = DLSeverity::DLWarningC;
    let message = "Pin versions in go. Instead of `go install <package>` use `go install <package>@<version>`";

    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => fold_arguments(
            |a: &ParsedShell| {
                go_package_command(a)
                    .iter()
                    .all(|p| check_go_package_version(p))
            },
            args,
        ),
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

/// Haskell `goPackageCommand`.
fn go_package_command(args: &ParsedShell) -> Vec<String> {
    let run = run_go_package(args);
    if !run.is_empty() {
        return run;
    }
    let get = get_go_package(args);
    if !get.is_empty() {
        return get;
    }
    vec![]
}

/// Haskell `runGoPackage`.
fn run_go_package(args: &ParsedShell) -> Vec<String> {
    let mut out = vec![];
    for cmd in &args.present_commands {
        if shell::cmds_have_args(&["go"], &["run"], cmd) {
            for (idx, arg) in shell::get_args_no_flags(cmd).into_iter().enumerate() {
                // everything after the package name is an argument to the go program
                if arg != "run" && idx <= 1 {
                    out.push(arg);
                }
            }
        }
    }
    out
}

/// Haskell `getGoPackage`.
fn get_go_package(args: &ParsedShell) -> Vec<String> {
    let mut out = vec![];
    for cmd in &args.present_commands {
        if shell::cmds_have_args(&["go"], &["install", "get"], cmd) {
            for arg in shell::get_args_no_flags(cmd) {
                if !["install", "get", "tool"].contains(&arg.as_str()) {
                    out.push(arg);
                }
            }
        }
    }
    out
}

/// Haskell `hasVersionSymbol`.
fn has_version_symbol(package: &str) -> bool {
    package.contains('@')
}

/// Haskell `isTagsVersion`.
fn is_tags_version(package: &str) -> bool {
    ["latest", "none"]
        .iter()
        .any(|tag| package.ends_with(&format!("@{}", tag)))
}

/// Haskell `isLocalPath`.
fn is_local_path(p: &str) -> bool {
    p == "." || p.starts_with('/') || p.starts_with('.')
}

/// Haskell `checkGoPackageVersion`.
fn check_go_package_version(package: &str) -> bool {
    is_local_path(package) || has_version_symbol(package) && !is_tags_version(package)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn go_install_version_not_pinned() {
        rule_catches("DL3062", "RUN go install example.com/pkg");
        on_build_rule_catches("DL3062", "RUN go install example.com/pkg");
    }

    #[test]
    fn go_install_version_pinned_as_latest() {
        rule_catches("DL3062", "RUN go install example.com/pkg@latest");
        on_build_rule_catches("DL3062", "RUN go install example.com/pkg@latest");
    }

    #[test]
    fn go_install_version_pinned() {
        rule_catches_not("DL3062", "RUN go install example.com/pkg@v1.2.3");
        on_build_rule_catches_not("DL3062", "RUN go install example.com/pkg@v1.2.3");
    }

    #[test]
    fn go_install_with_local_absolute_path() {
        rule_catches_not("DL3062", "RUN go install /go/app/foobar");
        on_build_rule_catches_not("DL3062", "RUN go install /go/app/foobar");
    }

    #[test]
    fn go_install_with_local_relative_path() {
        rule_catches_not("DL3062", "RUN go install ./app/foobar");
        on_build_rule_catches_not("DL3062", "RUN go install ./app/foobar");
    }

    #[test]
    fn go_install_local_dir() {
        rule_catches_not("DL3062", "RUN go install .");
        on_build_rule_catches_not("DL3062", "RUN go install .");
    }

    #[test]
    fn go_get_version_not_pinned() {
        rule_catches("DL3062", "RUN go get example.com/pkg");
        on_build_rule_catches("DL3062", "RUN go get example.com/pkg");
    }

    #[test]
    fn go_get_version_pinned_as_latest() {
        rule_catches("DL3062", "RUN go get example.com/pkg@latest");
        on_build_rule_catches("DL3062", "RUN go get example.com/pkg@latest");
    }

    #[test]
    fn go_get_version_pinned() {
        rule_catches_not("DL3062", "RUN go get example.com/pkg@v1.2.3");
        on_build_rule_catches_not("DL3062", "RUN go get example.com/pkg@v1.2.3");
    }

    #[test]
    fn go_get_tool_version_not_pinned() {
        rule_catches("DL3062", "RUN go get -tool foobar");
        on_build_rule_catches("DL3062", "RUN go get -tool foobar");
    }

    #[test]
    fn go_get_tool_version_pinned_as_latest() {
        rule_catches("DL3062", "RUN go get -tool foobar@latest");
        on_build_rule_catches("DL3062", "RUN go get -tool foobar@latest");
    }

    #[test]
    fn go_get_tool_version_pinned() {
        rule_catches_not("DL3062", "RUN go get -tool foobar@v1.2.3");
        on_build_rule_catches_not("DL3062", "RUN go get -tool foobar@v1.2.3");
    }

    #[test]
    fn go_run_version_not_pinned() {
        rule_catches("DL3062", "RUN go run example.com/pkg");
        on_build_rule_catches("DL3062", "RUN go run example.com/pkg");
    }

    #[test]
    fn go_run_version_pinned_as_latest() {
        rule_catches("DL3062", "RUN go run example.com/pkg@latest");
        on_build_rule_catches("DL3062", "RUN go run example.com/pkg@latest");
    }

    #[test]
    fn go_run_version_pinned() {
        rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3");
        on_build_rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3");
    }

    #[test]
    fn go_run_with_local_absolute_path() {
        rule_catches_not("DL3062", "RUN go run /go/app/foobar");
        on_build_rule_catches_not("DL3062", "RUN go run /go/app/foobar");
    }

    #[test]
    fn go_run_with_local_relative_path() {
        rule_catches_not("DL3062", "RUN go run ./app/foobar");
        on_build_rule_catches_not("DL3062", "RUN go run ./app/foobar");
    }

    #[test]
    fn go_run_local_dir() {
        rule_catches_not("DL3062", "RUN go run .");
        on_build_rule_catches_not("DL3062", "RUN go run .");
    }

    #[test]
    fn go_run_version_pinned_with_flags() {
        rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3 -f");
        on_build_rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3 -f");
        rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3 -foobar");
        on_build_rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3 -foobar");
        rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3 --foobar");
        on_build_rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3 --foobar");
    }

    #[test]
    fn go_run_version_pinned_with_arguments() {
        rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3 foobar");
        on_build_rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3 foobar");
        rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3 --foo bar");
        on_build_rule_catches_not("DL3062", "RUN go run example.com/pkg@v1.2.3 --foo bar");
    }
}
