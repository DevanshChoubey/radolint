// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3042.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, State, add_fail, combine, custom_rule, empty_state,
    fold_arguments, modify, onbuild, pipe,
};
use crate::shell::{self, ParsedShell};
use crate::utils;
use dlint_syntax::{BaseImage, Instruction, Pairs, RunArgs};
use std::collections::BTreeMap;

/// This data encapsulates the name of a build stage. It may be None withing an
/// `ONBUILD` context.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    Stage(String),
    None,
}

#[derive(Debug, Clone)]
enum Acc {
    Acc {
        current: Stage,
        no_cache_map: BTreeMap<Stage, bool>,
    },
    Empty,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3042(), onbuild(dl3042()))
}

fn dl3042() -> Box<dyn Rule> {
    let code = "DL3042";
    let severity = DLSeverity::DLWarningC;
    let message =
        "Avoid use of cache directory with pip. Use `pip install --no-cache-dir <package>`";

    let check = move |line, st: State<Acc>, instr: &Instruction<ParsedShell>| match instr {
        Instruction::From(from) => pipe(st, |s| modify(|a| remember_stage(from, a), s)),
        Instruction::Env(pairs) => pipe(st, |s| modify(|a| register_env(pairs, a), s)),
        Instruction::Run(RunArgs(args, flags)) => {
            if let Acc::Acc {
                current,
                no_cache_map,
            } = &st.state
                && no_cache_map.get(current) == Some(&true)
            {
                return st;
            }
            if fold_arguments(pip_no_cache_dir_is_set, args) {
                st
            } else if fold_arguments(
                |s: &ParsedShell| shell::no_commands(forgot_no_cache_dir, s),
                args,
            ) {
                st
            } else if utils::has_cache_or_tmpfs_mount_with(".cache/pip", flags) {
                st
            } else {
                add_fail(
                    CheckFailure {
                        code: RuleCode::from(code),
                        severity,
                        message: message.to_string(),
                        line,
                    },
                    st,
                )
            }
        }
        _ => st,
    };
    custom_rule(check, empty_state(Acc::Empty))
}

fn forgot_no_cache_dir(cmd: &shell::Command) -> bool {
    shell::is_pip_install(cmd) && !uses_no_cache_dir(cmd) && !is_pip_wrapper(cmd)
}

fn uses_no_cache_dir(cmd: &shell::Command) -> bool {
    shell::get_args(cmd).iter().any(|a| a == "--no-cache-dir")
}

fn pip_no_cache_dir_is_set(shell: &ParsedShell) -> bool {
    let orig = shell.original.as_str();
    // snd (Text.breakOn "PIP_NO_CACHE_DIR=" original)
    let broken = match orig.find("PIP_NO_CACHE_DIR=") {
        Some(i) => &orig[i..],
        None => "",
    };
    // Text.drop 1 (Text.dropWhile (/= '=') ..)
    let after = match broken.find('=') {
        Some(i) => &broken[i + 1..],
        None => "",
    };
    TRUTHY.iter().any(|t| after.starts_with(t))
}

fn is_pip_wrapper(cmd: &shell::Command) -> bool {
    let name = cmd.name.as_str();
    let is_wrapper = |w: &str| -> bool {
        name.contains(w)
            || (name.starts_with("python") && {
                let args = shell::get_args(cmd);
                args.windows(2).any(|p| p[0] == "-m" && p[1] == w)
            })
    };
    is_wrapper("pipx") || is_wrapper("pipenv")
}

fn remember_stage(base: &BaseImage, acc: Acc) -> Acc {
    match (acc, &base.alias) {
        (Acc::Empty, Some(als)) => Acc::Acc {
            current: Stage::Stage(als.un_image_alias.clone()),
            no_cache_map: BTreeMap::new(),
        },
        (Acc::Empty, None) => Acc::Acc {
            current: Stage::Stage(base.image.image_name.clone()),
            no_cache_map: BTreeMap::new(),
        },
        (
            Acc::Acc {
                mut no_cache_map, ..
            },
            alias,
        ) => {
            let parent_value = no_cache_map
                .get(&Stage::Stage(base.image.image_name.clone()))
                .copied()
                .unwrap_or(false);
            let name = match alias {
                Some(als) => als.un_image_alias.clone(),
                None => base.image.image_name.clone(),
            };
            no_cache_map.insert(Stage::Stage(name.clone()), parent_value);
            Acc::Acc {
                current: Stage::Stage(name),
                no_cache_map,
            }
        }
    }
}

fn register_env(pairs: &Pairs, acc: Acc) -> Acc {
    match acc {
        Acc::Empty => {
            if pip_no_cache_dir_set(pairs) {
                let mut m = BTreeMap::new();
                m.insert(Stage::None, true);
                Acc::Acc {
                    current: Stage::None,
                    no_cache_map: m,
                }
            } else {
                Acc::Empty
            }
        }
        Acc::Acc {
            current,
            mut no_cache_map,
        } => {
            if pip_no_cache_dir_set(pairs) {
                no_cache_map.insert(current.clone(), true);
            }
            Acc::Acc {
                current,
                no_cache_map,
            }
        }
    }
}

fn pip_no_cache_dir_set(pairs: &Pairs) -> bool {
    match pairs.iter().find(|(k, _)| k == "PIP_NO_CACHE_DIR") {
        None => false,
        Some((_, v)) => TRUTHY.contains(&v.as_str()),
    }
}

const TRUTHY: [&str; 10] = [
    "1", "true", "True", "TRUE", "on", "On", "ON", "yes", "Yes", "YES",
];

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn pip2_no_cache_dir_not_used() {
        rule_catches("DL3042", "RUN pip2 install MySQL_python");
        on_build_rule_catches("DL3042", "RUN pip2 install MySQL_python");
    }
    #[test]
    fn pip3_no_cache_dir_not_used() {
        rule_catches("DL3042", "RUN pip3 install MySQL_python");
        on_build_rule_catches("DL3042", "RUN pip3 install MySQL_python");
    }
    #[test]
    fn pip_no_cache_dir_not_used() {
        rule_catches("DL3042", "RUN pip install MySQL_python");
        on_build_rule_catches("DL3042", "RUN pip install MySQL_python");
    }
    #[test]
    fn pip2_no_cache_dir_used() {
        rule_catches_not("DL3042", "RUN pip2 install MySQL_python --no-cache-dir");
        on_build_rule_catches_not("DL3042", "RUN pip2 install MySQL_python --no-cache-dir");
    }
    #[test]
    fn pip3_no_cache_dir_used() {
        rule_catches_not("DL3042", "RUN pip3 install --no-cache-dir MySQL_python");
        on_build_rule_catches_not("DL3042", "RUN pip3 install --no-cache-dir MySQL_python");
    }
    #[test]
    fn pip_no_cache_dir_used() {
        rule_catches_not("DL3042", "RUN pip install MySQL_python --no-cache-dir");
        on_build_rule_catches_not("DL3042", "RUN pip install MySQL_python --no-cache-dir");
    }
    #[test]
    fn dont_match_on_pipx() {
        rule_catches_not("DL3042", "RUN pipx install software");
        on_build_rule_catches_not("DL3042", "Run pipx install software");
    }
    #[test]
    fn dont_match_on_pipenv() {
        rule_catches_not("DL3042", "RUN pipenv install library");
        on_build_rule_catches_not("DL3042", "RUN pipenv install library");
    }
    #[test]
    fn respect_env_truthy() {
        for v in ["1", "on", "yes", "true"] {
            rule_catches_not(
                "DL3042",
                &format!("ENV PIP_NO_CACHE_DIR={v}\nRUN pip install MySQL_python"),
            );
        }
    }
    #[test]
    fn respect_run_truthy() {
        for v in ["1", "on", "yes", "true"] {
            rule_catches_not(
                "DL3042",
                &format!("RUN PIP_NO_CACHE_DIR={v} pip install MySQL_python"),
            );
        }
    }
    #[test]
    fn respect_run_export_truthy() {
        for v in ["1", "on", "yes", "true"] {
            rule_catches_not(
                "DL3042",
                &format!("RUN export PIP_NO_CACHE_DIR={v} && pip install MySQL_python"),
            );
        }
    }
    #[test]
    fn respect_env_falsy() {
        for v in ["0", "off", "no", "false"] {
            rule_catches(
                "DL3042",
                &format!("ENV PIP_NO_CACHE_DIR={v}\nRUN pip install MySQL_python"),
            );
        }
    }
    #[test]
    fn respect_run_falsy() {
        for v in ["0", "off", "no", "false"] {
            rule_catches(
                "DL3042",
                &format!("RUN PIP_NO_CACHE_DIR={v} pip install MySQL_python"),
            );
        }
    }
    #[test]
    fn respect_run_export_falsy() {
        for v in ["0", "off", "no", "false"] {
            rule_catches(
                "DL3042",
                &format!("RUN export PIP_NO_CACHE_DIR={v} && pip install MySQL_python"),
            );
        }
    }
    #[test]
    fn dont_trigger_if_inherited() {
        let d = [
            "FROM debian:buster as base",
            "ENV PIP_NO_CACHE_DIR=1",
            "FROM base",
            "RUN pip install six",
            "",
        ]
        .join("\n");
        rule_catches_not("DL3042", &d);
    }
    #[test]
    fn trigger_if_not_inherited() {
        let d = [
            "FROM debian:buster as base",
            "ENV PIP_NO_CACHE_DIR=1",
            "FROM debian:buster",
            "RUN pip install six",
            "",
        ]
        .join("\n");
        rule_catches("DL3042", &d);
    }
    #[test]
    fn ok_with_cache_mount_in_cache_dir() {
        let line = "RUN --mount=type=cache,target=/root/.cache/pip pip install foobar";
        rule_catches_not("DL3042", line);
        on_build_rule_catches_not("DL3042", line);
    }
    #[test]
    fn ok_with_tmpfs_mount_in_cache_dir() {
        let line = "RUN --mount=type=tmpfs,target=/root/.cache/pip pip install foobar";
        rule_catches_not("DL3042", line);
        on_build_rule_catches_not("DL3042", line);
    }
}
