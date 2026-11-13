// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3047.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{self, Command};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3047(), onbuild(dl3047()))
}

/// Haskell `dl3047`.
pub fn dl3047() -> Box<dyn Rule> {
    let code = "DL3047";
    let severity = DLSeverity::DLInfoC;
    let message = "Avoid use of wget without progress bar. Use `wget --progress=dot:giga <url>`. \
                   Or consider using `-q` or `-nv` (shorthands for `--quiet` or `--no-verbose`).";

    let check = |instr: &Instruction<_>| match instr {
        Instruction::Run(RunArgs(args, _)) => {
            fold_arguments(|a| shell::no_commands(forgot_progress, a), args)
        }
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

fn forgot_progress(cmd: &Command) -> bool {
    is_wget(cmd) && (!has_progress_option(cmd) && !has_special_flags(cmd))
}

fn is_wget(cmd: &Command) -> bool {
    cmd.name == "wget"
}

fn has_progress_option(cmd: &Command) -> bool {
    shell::has_flag("progress", cmd)
}

fn has_special_flags(cmd: &Command) -> bool {
    has_quiet_flag(cmd)
        || has_output_flag(cmd)
        || has_append_output_flag(cmd)
        || has_no_verbose_flag(cmd)
}

fn has_quiet_flag(cmd: &Command) -> bool {
    shell::has_any_flag(&["q", "quiet"], cmd)
}

fn has_output_flag(cmd: &Command) -> bool {
    shell::has_any_flag(&["o", "output-file"], cmd)
}

fn has_append_output_flag(cmd: &Command) -> bool {
    shell::has_any_flag(&["a", "append-output"], cmd)
}

fn has_no_verbose_flag(cmd: &Command) -> bool {
    shell::has_any_flag(&["no-verbose"], cmd) || shell::cmd_has_args("wget", &["-nv"], cmd)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn warns_when_using_wget_without_progress_option() {
        let docker_file = "FROM node as foo\nRUN wget my.xyz";
        rule_catches("DL3047", docker_file);
        on_build_rule_catches("DL3047", docker_file);
    }

    #[test]
    fn does_not_warn_when_running_with_progress_option() {
        let docker_file = "FROM node as foo\nRUN wget --progress=dot:giga my.xyz";
        rule_catches_not("DL3047", docker_file);
        on_build_rule_catches_not("DL3047", docker_file);
    }

    #[test]
    fn does_not_warn_with_q() {
        let docker_file = "FROM node as foo\nRUN wget -q my.xyz";
        rule_catches_not("DL3047", docker_file);
        on_build_rule_catches_not("DL3047", docker_file);
    }

    #[test]
    fn does_not_warn_with_quiet() {
        let docker_file = "FROM node as foo\nRUN wget --quiet my.xyz";
        rule_catches_not("DL3047", docker_file);
        on_build_rule_catches_not("DL3047", docker_file);
    }

    #[test]
    fn does_not_warn_with_nv() {
        let docker_file = "FROM node as foo\nRUN wget -nv my.xyz";
        rule_catches_not("DL3047", docker_file);
        on_build_rule_catches_not("DL3047", docker_file);
    }

    #[test]
    fn does_not_warn_with_no_verbose() {
        let docker_file = "FROM node as foo\nRUN wget --no-verbose my.xyz";
        rule_catches_not("DL3047", docker_file);
        on_build_rule_catches_not("DL3047", docker_file);
    }

    #[test]
    fn does_not_warn_with_output_file() {
        let docker_file = "FROM node as foo\nRUN wget --output-file=/tmp/wget.log my.xyz";
        rule_catches_not("DL3047", docker_file);
        on_build_rule_catches_not("DL3047", docker_file);
    }

    #[test]
    fn does_not_warn_with_o() {
        let docker_file = "FROM node as foo\nRUN wget -o /tmp/wget.log my.xyz";
        rule_catches_not("DL3047", docker_file);
        on_build_rule_catches_not("DL3047", docker_file);
    }

    #[test]
    fn does_not_warn_with_append_output() {
        let docker_file = "FROM node as foo\nRUN wget --append-output=/tmp/wget.log my.xyz";
        rule_catches_not("DL3047", docker_file);
        on_build_rule_catches_not("DL3047", docker_file);
    }

    #[test]
    fn does_not_warn_with_a() {
        let docker_file = "FROM node as foo\nRUN wget -a /tmp/wget.log my.xyz";
        rule_catches_not("DL3047", docker_file);
        on_build_rule_catches_not("DL3047", docker_file);
    }
}
