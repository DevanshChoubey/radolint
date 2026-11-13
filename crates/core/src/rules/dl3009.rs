// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3009.hs.

use crate::rule::*;
use crate::shell;
use crate::shell::ParsedShell;
use crate::utils;
use radolint_syntax::*;
use std::collections::BTreeMap;

/// Haskell `data Acc`.
#[derive(Debug, Clone, PartialEq)]
enum Acc {
    Acc {
        last_from: BaseImage,
        docker_clean: bool,
        stages: BTreeMap<String, Linenumber>,
        forgets: BTreeMap<Linenumber, BaseImage>,
    },
    Empty,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3009(), onbuild(dl3009()))
}

/// Haskell `dl3009`.
fn dl3009() -> Box<dyn Rule> {
    let code = "DL3009";
    let severity = DLSeverity::DLInfoC;
    let message = "Delete the apt lists (/var/lib/apt/lists) after installing something";

    let check =
        |line: Linenumber, st: State<Acc>, instr: &Instruction<ParsedShell>| -> State<Acc> {
            match instr {
                Instruction::From(from) => modify(|a| remember_stage(line, from, a), st),
                Instruction::Run(RunArgs(args, flags)) => {
                    if !fold_arguments(forgot_to_cleanup, args) {
                        if fold_arguments(disabled_docker_clean, args) {
                            modify(remember_docker_clean, st)
                        } else {
                            st
                        }
                    } else if utils::has_cache_or_tmpfs_mount_with("/var/lib/apt/lists", flags) {
                        st
                    } else if utils::has_cache_or_tmpfs_mount_with("/var/lib/apt", flags)
                        && utils::has_cache_or_tmpfs_mount_with("/var/cache/apt", flags)
                    {
                        st
                    } else {
                        modify(|a| remember_line(line, a), st)
                    }
                }
                _ => st,
            }
        };

    // Convert the final state into failures
    let mark_failures = move |st: State<Acc>| -> Failures {
        match st.state {
            Acc::Empty => st.failures,
            Acc::Acc {
                last_from,
                stages,
                forgets,
                ..
            } => {
                let mut out = Vec::new();
                for (line, from) in forgets.iter() {
                    let fail = CheckFailure {
                        code: RuleCode::from(code),
                        severity,
                        message: message.to_string(),
                        line: *line,
                    };
                    if *from == last_from {
                        out.push(fail);
                    } else if let Some(ImageAlias {
                        un_image_alias: als,
                    }) = &from.alias
                    {
                        // Check if this stage is used later
                        if stages.contains_key(als) {
                            // If the same alias is used in another stage, fail
                            out.push(fail);
                        }
                    }
                }
                out
            }
        }
    };

    very_custom_rule(check, empty_state(Acc::Empty), mark_failures)
}

/// Haskell `rememberStage`.
fn remember_stage(line: Linenumber, from: &BaseImage, acc: Acc) -> Acc {
    // Haskell pattern `Image _ als` binds the image name.
    let als = from.image.image_name.clone();
    match acc {
        Acc::Acc {
            stages, forgets: o, ..
        } => {
            let mut stages = stages;
            stages.insert(als, line);
            Acc::Acc {
                last_from: from.clone(),
                docker_clean: true,
                stages,
                forgets: o,
            }
        }
        Acc::Empty => {
            let mut stages = BTreeMap::new();
            stages.insert(als, line);
            Acc::Acc {
                last_from: from.clone(),
                docker_clean: true,
                stages,
                forgets: BTreeMap::new(),
            }
        }
    }
}

/// Haskell `rememberLine`.
fn remember_line(line: Linenumber, acc: Acc) -> Acc {
    match acc {
        Acc::Acc {
            last_from,
            docker_clean,
            stages,
            mut forgets,
        } => {
            forgets.insert(line, last_from.clone());
            Acc::Acc {
                last_from,
                docker_clean,
                stages,
                forgets,
            }
        }
        Acc::Empty => {
            let mut forgets = BTreeMap::new();
            forgets.insert(line, empty_image());
            Acc::Acc {
                last_from: empty_image(),
                docker_clean: true,
                stages: BTreeMap::new(),
                forgets,
            }
        }
    }
}

/// Haskell `rememberDockerClean`.
fn remember_docker_clean(acc: Acc) -> Acc {
    match acc {
        Acc::Empty => Acc::Acc {
            last_from: empty_image(),
            docker_clean: false,
            stages: BTreeMap::new(),
            forgets: BTreeMap::new(),
        },
        Acc::Acc {
            last_from,
            stages,
            forgets,
            ..
        } => Acc::Acc {
            last_from,
            docker_clean: false,
            stages,
            forgets,
        },
    }
}

/// Haskell `forgotToCleanup`.
fn forgot_to_cleanup(args: &ParsedShell) -> bool {
    let has_cleanup = args
        .present_commands
        .iter()
        .any(|c| shell::cmd_has_args("rm", &["-rf", "/var/lib/apt/lists/*"], c));
    has_update(args) && !has_cleanup
}

/// Haskell `hasUpdate`.
fn has_update(args: &ParsedShell) -> bool {
    let is_package_update = |cmd: &shell::Command| {
        shell::cmd_has_args("apt", &["update"], cmd)
            || shell::cmd_has_args("apt-get", &["update"], cmd)
            || shell::cmd_has_args("aptitude", &["update"], cmd)
    };
    args.present_commands.iter().any(is_package_update)
}

/// Haskell `disabledDockerClean`.
fn disabled_docker_clean(args: &ParsedShell) -> bool {
    let removes_script = args
        .present_commands
        .iter()
        .any(|c| shell::cmd_has_args("rm", &["/etc/apt/apt.conf.d/docker-clean"], c));
    let keeps_packages = args.present_commands.iter().any(|c| {
        shell::cmd_has_args(
            "echo",
            &["'Binary::apt::APT::Keep-Downloaded-Packages \"true\";'"],
            c,
        )
    });
    removes_script || keeps_packages
}

/// Haskell `emptyImage`: even though dockerfiles without a FROM are not valid, we still want to
/// provide some feedback for this rule so we pretend there is a base image at the start.
fn empty_image() -> BaseImage {
    BaseImage {
        image: Image {
            registry_name: None,
            image_name: "scratch".to_string(),
        },
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
    fn apt_get_no_cleanup() {
        let docker_file = [
            "FROM scratch",
            "RUN apt-get update && apt-get install python",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches("DL3009", &docker_file);
        on_build_rule_catches("DL3009", &docker_file);
    }

    #[test]
    fn apt_get_cleanup_in_stage_image() {
        let docker_file = [
            "FROM ubuntu as foo",
            "RUN apt-get update && apt-get install python",
            "FROM scratch",
            "RUN echo hey!",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches("DL3009", &docker_file);
    }

    #[test]
    fn apt_get_no_cleanup_in_last_stage() {
        let docker_file = [
            "FROM ubuntu as foo",
            "RUN hey!",
            "FROM scratch",
            "RUN apt-get update && apt-get install python",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches("DL3009", &docker_file);
        on_build_rule_catches("DL3009", &docker_file);
    }

    #[test]
    fn apt_get_no_cleanup_in_intermediate_stage() {
        let docker_file = [
            "FROM ubuntu as foo",
            "RUN apt-get update && apt-get install python",
            "FROM foo",
            "RUN hey!",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches("DL3009", &docker_file);
        on_build_rule_catches("DL3009", &docker_file);
    }

    #[test]
    fn no_warn_apt_get_cleanup_in_intermediate_stage_that_cleans_lists() {
        let docker_file = [
            "FROM ubuntu as foo",
            "RUN apt-get update && apt-get install python && rm -rf /var/lib/apt/lists/*",
            "FROM foo",
            "RUN hey!",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches_not("DL3009", &docker_file);
    }

    #[test]
    fn no_warn_apt_get_cleanup_in_intermediate_stage_when_stage_not_used_later() {
        let docker_file = [
            "FROM ubuntu as foo",
            "RUN apt-get update && apt-get install python",
            "FROM scratch",
            "RUN hey!",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches("DL3009", &docker_file);
    }

    #[test]
    fn apt_get_cleanup() {
        let docker_file = [
            "FROM scratch",
            "RUN apt-get update && apt-get install python && rm -rf /var/lib/apt/lists/*",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches_not("DL3009", &docker_file);
    }

    #[test]
    fn dont_warn_buildkit_cache_mount_to_apt_lists_directory() {
        let docker_file = [
            "RUN --mount=type=cache,target=/var/lib/apt/lists     apt-get update && apt-get install python",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches_not("DL3009", &docker_file);
    }

    #[test]
    fn dont_warn_buildkit_cache_mount_to_apt_directories_1() {
        let docker_file = [
            "RUN --mount=type=cache,target=/var/lib/apt     --mount=type=cache,target=/var/cache/apt     rm -f /etc/apt/apt.conf.d/docker-clean &&     apt-get update && apt-get install python",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches_not("DL3009", &docker_file);
    }

    #[test]
    fn dont_warn_buildkit_cache_mount_to_apt_directories_2() {
        let docker_file = [
            "RUN rm -f /etc/apt/apt.conf.d/docker-clean",
            "RUN --mount=type=cache,target=/var/cache/apt \\",
            "    --mount=type=cache,target=/var/lib/apt \\",
            "    apt-get update && apt-get install foo",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches_not("DL3009", &docker_file);
    }

    #[test]
    fn warn_buildkit_cache_mount_to_apt_cache_directory_only() {
        let docker_file = [
            "RUN --mount=type=cache,target=/var/cache/apt \\",
            "    rm -f /etc/apt/apt.conf.d/docker-clean && \\",
            "    apt-get update && apt-get install python",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches("DL3009", &docker_file);
        on_build_rule_catches("DL3009", &docker_file);
    }

    #[test]
    fn warn_buildkit_cache_mount_to_apt_lists_directory_only() {
        let docker_file = [
            "RUN rm -f /etc/apt/apt.conf.d/docker-clean",
            "RUN --mount=type=cache,target=/var/lib/apt \\",
            "    apt-get update && apt-get install python",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches("DL3009", &docker_file);
        on_build_rule_catches("DL3009", &docker_file);
    }

    #[test]
    fn dont_warn_tmpfs_mount_to_apt_cache_and_lists_directory() {
        let docker_file = [
            "RUN \\",
            "  --mount=type=tmpfs,target=/var/cache/apt \\",
            "  --mount=type=tmpfs,target=/var/lib/apt \\",
            "  apt-get update",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches_not("DL3009", &docker_file);
    }

    #[test]
    fn dont_warn_tmpfs_mount_to_apt_cache_and_cache_mount_to_lists_directory() {
        let docker_file = [
            "RUN \\",
            "  --mount=type=tmpfs,target=/var/cache/apt \\",
            "  --mount=type=cache,target=/var/lib/apt \\",
            "  apt-get update",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches_not("DL3009", &docker_file);
    }

    #[test]
    fn dont_warn_cache_mount_to_apt_cache_and_tmpfs_mount_to_lists_directory() {
        let docker_file = [
            "RUN \\",
            "  --mount=type=cache,target=/var/cache/apt \\",
            "  --mount=type=tmpfs,target=/var/lib/apt \\",
            "  apt-get update",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches_not("DL3009", &docker_file);
    }

    #[test]
    fn dont_warn_cache_mount_to_apt_cache_and_lists_directory() {
        let docker_file = [
            "RUN \\",
            "  --mount=type=cache,target=/var/cache/apt \\",
            "  --mount=type=cache,target=/var/lib/apt \\",
            "  apt-get update",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches_not("DL3009", &docker_file);
    }

    #[test]
    fn apt_no_cleanup() {
        let docker_file = ["FROM scratch", "RUN apt update && apt install python"];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches("DL3009", &docker_file);
        on_build_rule_catches("DL3009", &docker_file);
    }

    #[test]
    fn apt_cleanup() {
        let docker_file = [
            "FROM scratch",
            "RUN apt update && apt install python && rm -rf /var/lib/apt/lists/*",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches_not("DL3009", &docker_file);
    }

    #[test]
    fn aptitude_no_cleanup() {
        let docker_file = [
            "FROM scratch",
            "RUN aptitude update && aptitude install python",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches("DL3009", &docker_file);
        on_build_rule_catches("DL3009", &docker_file);
    }

    #[test]
    fn aptitude_cleanup() {
        let docker_file = [
            "FROM scratch",
            "RUN aptitude update && aptitude install python && rm -rf /var/lib/apt/lists/*",
        ];
        let docker_file = docker_file
            .iter()
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        rule_catches_not("DL3009", &docker_file);
        on_build_rule_catches_not("DL3009", &docker_file);
    }
}
