// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3060.hs.

use crate::rule::*;
use crate::shell::{self, Command, ParsedShell};
use crate::utils;
use radolint_syntax::*;
use std::collections::BTreeMap;

/// Haskell `data Acc`.
#[derive(Debug, Clone)]
enum Acc {
    Acc {
        current: BaseImage,
        active: BTreeMap<String, Linenumber>,
        inactive: BTreeMap<Linenumber, BaseImage>,
    },
    Empty,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3060(), onbuild(dl3060()))
}

/// Haskell `dl3060`.
fn dl3060() -> Box<dyn Rule> {
    let code = "DL3060";
    let severity = DLSeverity::DLInfoC;
    let message = "`yarn cache clean` missing after `yarn install` was run.";

    let check =
        |line: Linenumber, st: State<Acc>, instr: &Instruction<ParsedShell>| -> State<Acc> {
            match instr {
                Instruction::From(from) => {
                    let from = from.clone();
                    modify(move |a| remember_stage(line, from, a), st)
                }
                Instruction::Run(RunArgs(args, flags)) => {
                    if fold_arguments(|a| shell::any_commands(yarn_install, a), args)
                        && fold_arguments(|a| shell::no_commands(yarn_cache_clean, a), args)
                        && !utils::has_cache_or_tmpfs_mount_with(".cache/yarn", flags)
                    {
                        return modify(move |a| remember_line(line, a), st);
                    }
                    if fold_arguments(|a| shell::any_commands(yarn_install, a), args)
                        && fold_arguments(|a| shell::any_commands(yarn_cache_clean, a), args)
                        && fold_arguments(is_clean_before_install, args)
                    {
                        modify(move |a| remember_line(line, a), st)
                    } else {
                        st
                    }
                }
                _ => st,
            }
        };

    // Produce failures from the final state Acc
    let mark_failures = move |st: State<Acc>| -> Failures {
        match st.state {
            Acc::Empty => st.failures,
            Acc::Acc {
                current,
                active,
                inactive,
            } => {
                let mut out = vec![];
                for (line, from) in inactive.iter() {
                    let fail = || CheckFailure {
                        code: RuleCode::from(code),
                        severity,
                        message: message.to_string(),
                        line: *line,
                    };
                    if *from == current {
                        out.push(fail());
                    } else if let BaseImage {
                        alias:
                            Some(ImageAlias {
                                un_image_alias: als,
                            }),
                        ..
                    } = from
                        && active.contains_key(als)
                    {
                        out.push(fail());
                    }
                }
                out
            }
        }
    };

    very_custom_rule(check, empty_state(Acc::Empty), mark_failures)
}

/// Haskell `rememberStage`.
fn remember_stage(line: Linenumber, stage: BaseImage, acc: Acc) -> Acc {
    // NB: upstream binds `Image _ als`, i.e. the image *name* (not the alias).
    let als = stage.image.image_name.clone();
    match acc {
        Acc::Empty => {
            let mut active = BTreeMap::new();
            active.insert(als, line);
            Acc::Acc {
                current: stage,
                active,
                inactive: BTreeMap::new(),
            }
        }
        Acc::Acc {
            active, inactive, ..
        } => {
            let mut active = active;
            active.insert(als, line);
            Acc::Acc {
                current: stage,
                active,
                inactive,
            }
        }
    }
}

/// Haskell `rememberLine`.
fn remember_line(line: Linenumber, acc: Acc) -> Acc {
    match acc {
        Acc::Empty => {
            let mut inactive = BTreeMap::new();
            inactive.insert(line, scratch());
            Acc::Acc {
                current: scratch(),
                active: BTreeMap::new(),
                inactive,
            }
        }
        Acc::Acc {
            current,
            active,
            mut inactive,
        } => {
            inactive.insert(line, current.clone());
            Acc::Acc {
                current,
                active,
                inactive,
            }
        }
    }
}

/// Haskell `yarnInstall`.
fn yarn_install(c: &Command) -> bool {
    shell::cmd_has_args("yarn", &["install"], c)
}

/// Haskell `yarnCacheClean`.
fn yarn_cache_clean(c: &Command) -> bool {
    shell::cmd_has_args("yarn", &["cache", "clean"], c)
}

/// Haskell `isCleanBeforeInstall`.
fn is_clean_before_install(args: &ParsedShell) -> bool {
    match (
        shell::find_command_index(yarn_cache_clean, args),
        shell::find_command_index(yarn_install, args),
    ) {
        (Some(clean_idx), Some(install_idx)) => clean_idx < install_idx,
        _ => false,
    }
}

/// This is needed as placeholder when no FROM statement has yet been encountered.
fn scratch() -> BaseImage {
    BaseImage {
        image: Image::from("scratch"),
        tag: None,
        digest: None,
        alias: None,
        platform: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn ok_with_non_yarn_commands() {
        rule_catches_not("DL3060", "RUN foo");
        on_build_rule_catches_not("DL3060", "RUN foo");
    }
    #[test]
    fn not_ok_with_no_cache_clean() {
        rule_catches("DL3060", "RUN yarn install foo");
        on_build_rule_catches("DL3060", "RUN yarn install foo");
    }
    #[test]
    fn ok_with_cache_clean() {
        rule_catches_not("DL3060", "RUN yarn install bar && yarn cache clean");
        on_build_rule_catches_not("DL3060", "RUN yarn install bar && yarn cache clean");
    }
    #[test]
    fn not_ok_with_clean_before_install() {
        rule_catches("DL3060", "RUN yarn cache clean && yarn install foo");
    }
    #[test]
    fn ok_when_cache_mount_is_used() {
        let line = "RUN --mount=type=cache,target=/root/.cache/yarn yarn install foobar";
        rule_catches_not("DL3060", line);
        on_build_rule_catches_not("DL3060", line);
    }
    #[test]
    fn ok_when_tmpfs_mount_is_used() {
        let line = "RUN --mount=type=tmpfs,target=/root/.cache/yarn yarn install foobar";
        rule_catches_not("DL3060", line);
        on_build_rule_catches_not("DL3060", line);
    }
    #[test]
    fn not_ok_when_cache_mount_is_in_wrong_location() {
        let line = "RUN --mount=type=cache,target=/var/lib/foobar yarn install foobar";
        rule_catches("DL3060", line);
        on_build_rule_catches("DL3060", line);
    }
    #[test]
    fn not_ok_when_tmpfs_mount_is_in_wrong_location() {
        let line = "RUN --mount=type=tmpfs,target=/var/lib/foobar yarn install foobar";
        rule_catches("DL3060", line);
        on_build_rule_catches("DL3060", line);
    }
    #[test]
    fn not_ok_when_yarn_install_is_in_last_stage_wo_yarn_clean() {
        let d = [
            "FROM node:lts-alpine as foo",
            "RUN hey!",
            "FROM scratch",
            "RUN yarn install",
            "",
        ]
        .join("\n");
        rule_catches("DL3060", &d);
        on_build_rule_catches("DL3060", &d);
    }
    #[test]
    fn not_ok_when_inheriting_from_stage_with_yarn_install_wo_yarn_clean() {
        let d = [
            "FROM node:lts-alpine as foo",
            "RUN yarn install",
            "FROM foo",
            "RUN hey!",
            "",
        ]
        .join("\n");
        rule_catches("DL3060", &d);
        on_build_rule_catches("DL3060", &d);
    }
    #[test]
    fn ok_when_inheriting_from_stage_with_yarn_cache_clear() {
        let d = [
            "FROM node:lts-alpine as foo",
            "RUN yarn install && yarn cache clean",
            "FROM foo",
            "RUN hey!",
            "",
        ]
        .join("\n");
        rule_catches_not("DL3060", &d);
        on_build_rule_catches_not("DL3060", &d);
    }
    #[test]
    fn ok_when_omitting_yarn_cache_clean_in_stage_that_is_not_reused_later() {
        let d = [
            "FROM node:lts-alpine as foo",
            "RUN yarn install foo",
            "FROM scratch",
            "RUN hey!",
            "",
        ]
        .join("\n");
        rule_catches_not("DL3060", &d);
        on_build_rule_catches("DL3060", &d);
    }
}
