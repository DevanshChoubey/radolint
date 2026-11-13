// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/Shellcheck.hs.

use crate::pragma;
use crate::rule::*;
use crate::shell::{self, ParsedShell, ShellOpts};
use radolint_syntax::*;
use shelllint_core::interface::{PositionedComment, Severity};
use std::collections::BTreeMap;

/// Haskell `data Acc = Acc {..} | Empty`.
#[derive(Debug, Clone)]
enum Acc {
    Acc {
        opts: ShellOpts,
        default_opts: ShellOpts,
        stage_idx: i64,
        stage_opts: BTreeMap<i64, ShellOpts>,
        stages: BTreeMap<i64, String>,
    },
    Empty,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(scrule(), onbuild(scrule()))
}

fn scrule() -> Box<dyn Rule> {
    custom_rule(check, empty_state(Acc::Empty))
}

fn check(line: Linenumber, st: State<Acc>, instr: &Instruction<ParsedShell>) -> State<Acc> {
    match instr {
        Instruction::From(bi) => modify(|a| new_stage(bi, a), st),
        Instruction::Arg(name, _) => modify(|a| add_vars(std::slice::from_ref(name), a), st),
        Instruction::Env(pairs) => {
            let names: Vec<String> = pairs.iter().map(|(k, _)| k.clone()).collect();
            modify(|a| add_vars(&names, a), st)
        }
        Instruction::Shell(args) => {
            let sh = fold_arguments(|p: &ParsedShell| p.original.clone(), args);
            modify(|a| set_shell(&sh, a), st)
        }
        Instruction::Comment(com) => match pragma::parse_shell(com) {
            Some(sh) => modify(|a| shell_pragma(&sh, a), st),
            _ => st,
        },
        Instruction::Run(RunArgs(args @ Arguments::ArgumentsText(_), _)) => {
            let fails = get_failures(line, args, &st.state);
            fails.into_iter().fold(st, |s, f| add_fail(f, s))
        }
        Instruction::Run(RunArgs(Arguments::ArgumentsList(_), _)) => st,
        _ => st,
    }
}

fn get_failures(line: Linenumber, args: &Arguments<ParsedShell>, s: &Acc) -> Vec<CheckFailure> {
    match s {
        Acc::Empty => fold_arguments(
            |script| run_shell_check(line, &shell::default_shell_opts(), script),
            args,
        ),
        Acc::Acc { opts, .. } => fold_arguments(|script| run_shell_check(line, opts, script), args),
    }
}

/// `Set.fromList` over `CheckFailure`, whose `Ord` compares only the line: elements with the same
/// line are equal, so a later one replaces an earlier one and the result is ascending by line.
fn run_shell_check(
    line: Linenumber,
    options: &ShellOpts,
    script: &ParsedShell,
) -> Vec<CheckFailure> {
    let mut set: Vec<CheckFailure> = Vec::new();
    for c in shell::shellcheck(options, script) {
        let f = to_failure(line, &c);
        match set.binary_search_by(|e| e.cmp(&f)) {
            Ok(i) => set[i] = f,
            Err(i) => set.insert(i, f),
        }
    }
    set
}

fn new_stage(bi: &BaseImage, acc: Acc) -> Acc {
    let alias = &bi.alias;
    match acc {
        Acc::Empty => Acc::Acc {
            opts: shell::default_shell_opts(),
            default_opts: shell::default_shell_opts(),
            stage_idx: 0,
            stage_opts: BTreeMap::from([(0, shell::default_shell_opts())]),
            stages: match alias {
                None => BTreeMap::new(),
                Some(a) => BTreeMap::from([(0, a.un_image_alias.clone())]),
            },
        },
        Acc::Acc {
            opts: _,
            default_opts,
            stage_idx,
            mut stage_opts,
            mut stages,
        } => {
            // Updates the maps in place: Haskell's persistent maps share structure,
            // so copying them per FROM would make each stage cost O(stages).
            let found = stages
                .iter()
                .find(|(_, v)| **v == bi.image.image_name)
                .map(|(k, _)| *k);
            let the_opts = match found.and_then(|idx| stage_opts.get(&idx)) {
                None => default_opts.clone(),
                Some(o) => o.clone(),
            };
            stage_opts.insert(stage_idx + 1, the_opts.clone());
            if let Some(a) = alias {
                stages.insert(stage_idx + 1, a.un_image_alias.clone());
            }
            Acc::Acc {
                opts: the_opts,
                default_opts,
                stage_idx: stage_idx + 1,
                stage_opts,
                stages,
            }
        }
    }
}

/// `Map.update (\_ -> Just o) k m`: replace only if the key exists.
fn map_update(mut m: BTreeMap<i64, ShellOpts>, k: i64, o: &ShellOpts) -> BTreeMap<i64, ShellOpts> {
    if let Some(v) = m.get_mut(&k) {
        *v = o.clone();
    }
    m
}

fn add_vars(vars: &[String], acc: Acc) -> Acc {
    match acc {
        Acc::Empty => {
            let d = shell::default_shell_opts();
            let mut env_vars = d.env_vars.clone();
            env_vars.extend(vars.iter().cloned());
            let opts = ShellOpts {
                shell_name: d.shell_name.clone(),
                env_vars,
            };
            Acc::Acc {
                opts: opts.clone(),
                default_opts: d,
                stage_idx: 0,
                stage_opts: BTreeMap::from([(0, opts)]),
                stages: BTreeMap::new(),
            }
        }
        Acc::Acc {
            opts,
            default_opts,
            stage_idx,
            stage_opts,
            stages,
        } => {
            let d = shell::default_shell_opts();
            let mut env_vars = opts.env_vars;
            env_vars.extend(vars.iter().cloned());
            let new_opts = ShellOpts {
                shell_name: d.shell_name,
                env_vars,
            };
            Acc::Acc {
                stage_opts: map_update(stage_opts, stage_idx, &new_opts),
                opts: new_opts,
                default_opts,
                stage_idx,
                stages,
            }
        }
    }
}

fn set_shell(sh: &str, acc: Acc) -> Acc {
    match acc {
        Acc::Empty => {
            let d = shell::default_shell_opts();
            let opts = ShellOpts {
                shell_name: sh.to_string(),
                env_vars: d.env_vars.clone(),
            };
            Acc::Acc {
                opts: opts.clone(),
                default_opts: d,
                stage_idx: 0,
                stage_opts: BTreeMap::from([(0, opts)]),
                stages: BTreeMap::new(),
            }
        }
        Acc::Acc {
            opts,
            default_opts,
            stage_idx,
            stage_opts,
            stages,
        } => {
            let new_opts = ShellOpts {
                shell_name: sh.to_string(),
                env_vars: opts.env_vars,
            };
            Acc::Acc {
                stage_opts: map_update(stage_opts, stage_idx, &new_opts),
                opts: new_opts,
                default_opts,
                stage_idx,
                stages,
            }
        }
    }
}

fn shell_pragma(sh: &str, acc: Acc) -> Acc {
    match acc {
        Acc::Empty => {
            let new_opts = ShellOpts {
                shell_name: sh.to_string(),
                env_vars: shell::default_shell_opts().env_vars,
            };
            Acc::Acc {
                opts: new_opts.clone(),
                default_opts: new_opts.clone(),
                stage_idx: 0,
                stage_opts: BTreeMap::from([(0, new_opts)]),
                stages: BTreeMap::new(),
            }
        }
        Acc::Acc {
            opts,
            default_opts,
            stage_idx,
            stage_opts,
            stages,
        } => {
            let new_opts = ShellOpts {
                shell_name: sh.to_string(),
                env_vars: opts.env_vars,
            };
            Acc::Acc {
                stage_opts: map_update(stage_opts, stage_idx, &new_opts),
                opts: new_opts,
                default_opts,
                stage_idx,
                stages,
            }
        }
    }
}

/// Converts ShellCheck errors into our own errors type
fn to_failure(line: Linenumber, c: &PositionedComment) -> CheckFailure {
    CheckFailure {
        code: RuleCode::from(format!("SC{}", c.comment.code)),
        severity: get_dl_severity(&c.comment.severity),
        message: c.comment.message.clone(),
        line,
    }
}

fn get_dl_severity(s: &Severity) -> DLSeverity {
    match s {
        Severity::WarningC => DLSeverity::DLWarningC,
        Severity::InfoC => DLSeverity::DLInfoC,
        Severity::StyleC => DLSeverity::DLStyleC,
        _ => DLSeverity::DLErrorC,
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    fn both(df: &str, on_build_passes: bool, chk_passes: bool) {
        if chk_passes {
            assert_checks(df, |c| passes_shellcheck(&c));
        } else {
            assert_checks(df, |c| fails_shellcheck(&c));
        }
        if on_build_passes {
            assert_on_build_checks(df, |c| passes_shellcheck(&c));
        } else {
            assert_on_build_checks(df, |c| fails_shellcheck(&c));
        }
    }

    #[test]
    fn runs_shellcheck_on_run_instructions() {
        both("RUN echo $MISSING_QUOTES", false, false);
    }

    #[test]
    fn not_warns_on_valid_scripts() {
        both("RUN echo foo", true, true);
    }

    #[test]
    fn no_warnings_on_exec_notations() {
        both("RUN [\"foobar\", \"$f\"]", true, true);
    }

    #[test]
    fn does_not_complain_on_default_env_vars() {
        let df = [
            "RUN echo \"$HTTP_PROXY\"",
            "RUN echo \"$http_proxy\"",
            "RUN echo \"$HTTPS_PROXY\"",
            "RUN echo \"$https_proxy\"",
            "RUN echo \"$FTP_PROXY\"",
            "RUN echo \"$ftp_proxy\"",
            "RUN echo \"$NO_PROXY\"",
            "RUN echo \"$no_proxy\"",
            "",
        ]
        .join("\n");
        both(&df, true, true);
    }

    #[test]
    fn complain_on_missing_env_vars() {
        both("RUN echo \"$RTTP_PROXY\"\n", false, false);
    }

    #[test]
    fn is_aware_of_args_and_env() {
        let df = [
            "ARG foo=bar",
            "ARG another_foo",
            "ENV bar=10 baz=20",
            "RUN echo \"$foo\"",
            "RUN echo \"$another_foo\"",
            "RUN echo \"$bar\"",
            "RUN echo \"$baz\"",
            "",
        ]
        .join("\n");
        both(&df, false, true);
    }

    #[test]
    fn resets_env_vars_after_a_from() {
        let df = [
            "ARG foo=bar",
            "ARG another_foo",
            "ENV bar=10 baz=20",
            "FROM debian",
            "RUN echo \"$foo\"",
            "",
        ]
        .join("\n");
        both(&df, false, false);
    }

    #[test]
    fn defaults_the_shell_to_sh() {
        both("RUN echo $RANDOM\n", false, false);
    }

    #[test]
    fn can_change_the_shell_check_to_bash() {
        let df = [
            "SHELL [\"/bin/bash\", \"-eo\", \"pipefail\", \"-c\"]",
            "RUN echo $RANDOM",
            "",
        ]
        .join("\n");
        // This is debatable, as it should actually pass, but detecting it correctly
        // is quite difficult
        both(&df, false, true);
    }

    #[test]
    fn resets_the_shell_to_sh_after_a_from() {
        let df = [
            "SHELL [\"/bin/bash\", \"-eo\", \"pipefail\", \"-c\"]",
            "FROM debian",
            "RUN echo $RANDOM",
            "",
        ]
        .join("\n");
        both(&df, false, false);
    }

    #[test]
    fn does_not_complain_on_ash_shell() {
        let df = [
            "SHELL [\"/bin/ash\", \"-o\", \"pipefail\", \"-c\"]",
            "RUN echo hello",
            "",
        ]
        .join("\n");
        both(&df, true, true);
    }

    #[test]
    fn does_not_complain_on_non_posix_shells_pwsh() {
        let df = [
            "SHELL [\"pwsh\", \"-c\"]",
            "RUN Get-Variable PSVersionTable | Select-Object -ExpandProperty Value",
            "",
        ]
        .join("\n");
        both(&df, true, true);
    }

    #[test]
    fn does_not_complain_on_non_posix_shells_cmd_exe() {
        let df = [
            "SHELL [\"cmd.exe\", \"/c\"]",
            "RUN Get-Variable PSVersionTable | Select-Object -ExpandProperty Value",
            "",
        ]
        .join("\n");
        both(&df, true, true);
    }

    #[test]
    fn does_not_complain_on_non_posix_shells_powershell_absolute_path() {
        let df = [
            "SHELL [\"C:\\\\Windows\\\\System32\\\\WindowsPowerShell\\\\v1.0\\\\powershell.exe\", \"-noprofile\", \"-noninteractive\", \"-command\"]",
            "RUN Get-Variable PSVersionTable | Select-Object -ExpandProperty Value",
            "",
        ]
        .join("\n");
        both(&df, true, true);
    }

    #[test]
    fn respects_shell_pragma() {
        let df = [
            "FROM mcr.microsoft.com/foo/bar/windows:10",
            "# hadolint shell = powershell.exe",
            "RUN Get-Variable PSVersionTable | Select-Object -ExpandProperty Value",
            "",
        ]
        .join("\n");
        both(&df, true, true);
    }

    #[test]
    fn respects_global_shell_pragma() {
        let df = [
            "# hadolint shell=powershell.exe",
            "FROM mcr.microsoft.com/foo/bar/windows:10",
            "RUN Get-Variable PSVersionTable | Select-Object -ExpandProperty Value",
            "FROM mcr.microsoft.com/foo/bar/windows:10",
            "RUN Get-Variable PSVersionTable | Select-Object -ExpandProperty Value",
            "",
        ]
        .join("\n");
        both(&df, true, true);
    }

    #[test]
    fn shell_instructions_are_propagated_in_multi_stage_builds_1() {
        let df = [
            "FROM ubuntu:24.04 AS base",
            "SHELL [\"/bin/bash\", \"-e\", \"-o\", \"pipefail\", \"-c\"]",
            "",
            "FROM base",
            "RUN source /etc/os-release",
            "",
        ]
        .join("\n");
        assert_checks(&df, |c| passes_shellcheck(&c));
    }

    #[test]
    fn shell_instructions_are_propagated_only_propagated_if_referencing_earlier_stage() {
        let df = [
            "FROM ubuntu:24.04 AS base",
            "SHELL [\"/bin/bash\", \"-e\", \"-o\", \"pipefail\", \"-c\"]",
            "",
            "FROM ubuntu:24.04",
            "RUN source /etc/os-release",
            "",
        ]
        .join("\n");
        assert_checks(&df, |c| fails_shellcheck(&c));
    }

    #[test]
    fn shell_instructions_are_propagated_in_multi_stage_builds_2() {
        let df = [
            "FROM something AS nothing",
            "FROM ubuntu:24.04 AS base",
            "SHELL [\"/bin/bash\", \"-e\", \"-o\", \"pipefail\", \"-c\"]",
            "FROM nothing",
            "FROM base",
            "RUN source /etc/os-release",
            "",
        ]
        .join("\n");
        assert_checks(&df, |c| passes_shellcheck(&c));
    }

    #[test]
    fn shell_instructions_are_propagated_in_multi_stage_builds_3() {
        let df = [
            "FROM ubuntu:24.04 AS base",
            "SHELL [\"/bin/bash\", \"-e\", \"-o\", \"pipefail\", \"-c\"]",
            "FROM base AS next",
            "RUN foobar && barfoo | tee logfile",
            "FROM next",
            "RUN source /etc/os-release",
            "",
        ]
        .join("\n");
        assert_checks(&df, |c| passes_shellcheck(&c));
    }
}
