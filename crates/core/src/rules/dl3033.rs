// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3033.hs.

use crate::rule::*;
use crate::shell;
use crate::shell::ParsedShell;
use radolint_syntax::*;
use std::collections::{BTreeMap, BTreeSet};

/// Haskell `data Acc`.
#[derive(Debug, Clone)]
enum Acc {
    Acc {
        stage_idx: i64,
        stage_aliases: BTreeMap<i64, String>,
        envs: BTreeMap<i64, BTreeSet<String>>,
        args: BTreeSet<String>,
    },
    Empty,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3033(), onbuild(dl3033()))
}

/// Haskell `dl3033`.
fn dl3033() -> Box<dyn Rule> {
    let code = "DL3033";
    let severity = DLSeverity::DLWarningC;
    let message = "Specify version with `yum install -y <package>-<version>`.";

    let check =
        move |line: Linenumber, st: State<Acc>, instr: &Instruction<ParsedShell>| -> State<Acc> {
            match instr {
                Instruction::Run(RunArgs(a, _)) => {
                    if fold_arguments(
                        |s| {
                            yum_packages(s)
                                .iter()
                                .all(|p| package_version_fixed(&st.state, p))
                        },
                        a,
                    ) && fold_arguments(
                        |s| yum_modules(s).iter().all(|m| module_version_fixed(m)),
                        a,
                    ) {
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
                Instruction::Env(pairs) => modify(|a| register_envs(pairs, a), st),
                Instruction::Arg(arg, _) => modify(|a| register_arg(arg, a), st),
                Instruction::From(bi) => modify(|a| new_stage(bi, a), st),
                _ => st,
            }
        };
    custom_rule(check, empty_state(Acc::Empty))
}

/// Haskell `yumPackages`.
fn yum_packages(args: &ParsedShell) -> Vec<String> {
    let mut out = vec![];
    for cmd in &args.present_commands {
        if !shell::cmd_has_args("yum", &["module"], cmd) {
            out.extend(install_filter(cmd));
        }
    }
    out
}

/// Haskell `packageVersionFixed`.
fn package_version_fixed(acc: &Acc, package: &str) -> bool {
    let parts: Vec<&str> = package.split('-').collect();
    if parts.len() <= 1 {
        false // No dashes, definitively no version
    } else if package.ends_with(".rpm") {
        true // rpm files always have a version
    } else if package.contains('$') {
        env_defined(acc, package)
    } else {
        is_version_like(&parts[1..])
    }
}

/// Haskell `envDefined`.
fn env_defined(acc: &Acc, package: &str) -> bool {
    match acc {
        Acc::Empty => false,
        Acc::Acc {
            stage_idx,
            envs,
            args,
            ..
        } => {
            let empty = BTreeSet::new();
            let this_stage = envs.get(stage_idx).unwrap_or(&empty);
            this_stage.iter().any(|v| var_in_text(v, package))
                || args.iter().any(|v| var_in_text(v, package))
        }
    }
}

/// Haskell `varInText`.
fn var_in_text(var: &str, txt: &str) -> bool {
    txt.contains(&format!("${{{var}}}"))
}

/// Haskell `isVersionLike`.
fn is_version_like(parts: &[&str]) -> bool {
    if parts.is_empty() {
        return false; // No parts after splitting by hyphen
    }
    let part_is_valid = |p: &&str| p.chars().all(is_version_char);
    let part_starts_with_digit = |p: &&str| p.chars().next().is_some_and(|c| c.is_ascii_digit());
    parts.iter().all(part_is_valid) && parts.iter().any(part_starts_with_digit)
}

/// Haskell `isVersionChar`.
fn is_version_char(c: char) -> bool {
    c.is_ascii_digit()
        || c.is_ascii_uppercase()
        || c.is_ascii_lowercase()
        || ['.', '~', '^', '_', ':', '+'].contains(&c)
}

/// Haskell `yumModules`.
fn yum_modules(args: &ParsedShell) -> Vec<String> {
    let mut out = vec![];
    for cmd in &args.present_commands {
        if shell::cmd_has_args("yum", &["module"], cmd) {
            out.extend(install_filter(cmd));
        }
    }
    out
}

/// Haskell `moduleVersionFixed`.
fn module_version_fixed(t: &str) -> bool {
    t.contains(':')
}

/// Haskell `installFilter`.
fn install_filter(cmd: &shell::Command) -> Vec<String> {
    if !shell::cmd_has_args("yum", &["install"], cmd) {
        return vec![];
    }
    shell::get_args_no_flags(cmd)
        .into_iter()
        .filter(|a| a != "install" && a != "module")
        .collect()
}

/// Haskell `registerEnvs`.
fn register_envs(pairs: &Pairs, acc: Acc) -> Acc {
    let names: BTreeSet<String> = pairs.iter().map(|p| p.0.clone()).collect();
    match acc {
        Acc::Empty => Acc::Acc {
            stage_idx: 0,
            stage_aliases: BTreeMap::from([(0, String::new())]),
            envs: BTreeMap::from([(0, names)]),
            args: BTreeSet::new(),
        },
        Acc::Acc {
            stage_idx,
            stage_aliases,
            mut envs,
            args,
        } => {
            // Map.adjust: no-op when the key is absent
            if let Some(s) = envs.get_mut(&stage_idx) {
                s.extend(names);
            }
            Acc::Acc {
                stage_idx,
                stage_aliases,
                envs,
                args,
            }
        }
    }
}

/// Haskell `registerArg`.
fn register_arg(arg: &str, acc: Acc) -> Acc {
    match acc {
        Acc::Empty => Acc::Acc {
            stage_idx: 0,
            stage_aliases: BTreeMap::from([(0, String::new())]),
            envs: BTreeMap::from([(0, BTreeSet::new())]),
            args: BTreeSet::from([arg.to_string()]),
        },
        Acc::Acc {
            stage_idx,
            stage_aliases,
            envs,
            mut args,
        } => {
            args.insert(arg.to_string());
            Acc::Acc {
                stage_idx,
                stage_aliases,
                envs,
                args,
            }
        }
    }
}

/// Haskell `newStage`.
fn new_stage(bi: &BaseImage, acc: Acc) -> Acc {
    match acc {
        Acc::Empty => {
            let alias = match &bi.alias {
                None => String::new(),
                Some(a) => a.un_image_alias.clone(),
            };
            Acc::Acc {
                stage_idx: 1,
                stage_aliases: BTreeMap::from([(1, alias)]),
                envs: BTreeMap::from([(1, BTreeSet::new())]),
                args: BTreeSet::new(),
            }
        }
        Acc::Acc {
            stage_idx,
            mut stage_aliases,
            mut envs,
            args,
        } => {
            let als = |a: &Option<ImageAlias>| match a {
                None => String::new(),
                Some(a) => a.un_image_alias.clone(),
            };
            // stageIdxOfAlias: first (lowest key) stage whose alias equals the image name
            let stage_idx_of_alias = stage_aliases
                .iter()
                .find(|(_, v)| **v == bi.image.image_name)
                .map(|(k, _)| *k)
                .unwrap_or(stage_idx + 1);
            let stage_envs = envs.get(&stage_idx_of_alias).cloned().unwrap_or_default();
            stage_aliases.insert(stage_idx + 1, als(&bi.alias));
            envs.insert(stage_idx + 1, stage_envs);
            Acc::Acc {
                stage_idx: stage_idx + 1,
                stage_aliases,
                envs,
                args,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn not_ok_without_yum_version_pinning() {
        rule_catches("DL3033", "RUN yum install -y tomcat && yum clean all");
        on_build_rule_catches("DL3033", "RUN yum install -y tomcat && yum clean all");
    }

    #[test]
    fn ok_with_yum_version_pinning() {
        rule_catches_not("DL3033", "RUN yum install -y tomcat-9.2 && yum clean all");
        rule_catches_not("DL3033", "RUN bash -c `# not even a yum command`");
        on_build_rule_catches_not("DL3033", "RUN yum install -y tomcat-9.2 && yum clean all");
        on_build_rule_catches_not("DL3033", "RUN bash -c `# not even a yum command`");
    }

    #[test]
    fn ok_with_yum_version_pinning_version_contains_epoch() {
        rule_catches_not("DL3033", "RUN yum install -y openssl-1:1.1.1k");
        on_build_rule_catches_not("DL3033", "RUN yum install -y openssl-1:1.1.1k");
    }

    #[test]
    fn ok_with_yum_version_pinning_package_name_contains_dash() {
        rule_catches_not("DL3033", "RUN yum install -y rpm-sign-4.16.1.3");
        on_build_rule_catches_not("DL3033", "RUN yum install -y rpm-sign-4.16.1.3");
    }

    #[test]
    fn ok_with_yum_version_pinning_package_name_contains_dash_and_plus() {
        rule_catches_not("DL3033", "RUN yum install -y gcc-c++-1.1.1");
        on_build_rule_catches_not("DL3033", "RUN yum install -y gcc-c++-1.1.1");
    }

    #[test]
    fn not_ok_without_yum_version_pinning_modules() {
        rule_catches(
            "DL3033",
            "RUN yum module install -y tomcat && yum clean all",
        );
        on_build_rule_catches(
            "DL3033",
            "RUN yum module install -y tomcat && yum clean all",
        );
    }

    #[test]
    fn ok_with_yum_version_pinning_modules() {
        rule_catches_not(
            "DL3033",
            "RUN yum module install -y tomcat:9 && yum clean all",
        );
        rule_catches_not("DL3033", "RUN bash -c `# not even a yum command`");
        on_build_rule_catches_not(
            "DL3033",
            "RUN yum module install -y tomcat:9 && yum clean all",
        );
        on_build_rule_catches_not("DL3033", "RUN bash -c `# not even a yum command`");
    }

    #[test]
    fn ok_with_version_as_variable_braced() {
        let snippet = [
            "ENV version=2.51.0-2.fc42",
            "RUN yum -y install git-core-${version}",
            "",
        ]
        .join("\n");
        rule_catches_not("DL3033", &snippet);
    }

    #[test]
    fn ok_with_version_as_arg_unbraced() {
        let snippet = [
            "ARG version=2.51.0-2.fc42",
            "RUN yum -y install git-core-$version",
            "",
        ]
        .join("\n");
        rule_catches_not("DL3033", &snippet);
    }

    #[test]
    fn ok_with_version_as_arg_different_stages() {
        let snippet = [
            "FROM fedora:fc42 AS build",
            "ARG version=2.51.0-2.fc42",
            "FROM fedora:fc42",
            "RUN yum -y install git-core-${version}",
            "",
        ]
        .join("\n");
        rule_catches_not("DL3033", &snippet);
    }

    #[test]
    fn ok_with_version_as_env_different_stages_reused_stage() {
        let snippet = [
            "FROM fedora:fc42 AS build",
            "ENV version=2.51.0-2.fc42",
            "FROM build",
            "RUN yum -y install git-core-${version}",
            "",
        ]
        .join("\n");
        rule_catches_not("DL3033", &snippet);
    }

    #[test]
    fn not_ok_with_version_as_variable_different_stages_new_stage() {
        let snippet = [
            "FROM fedora:fc42 AS build",
            "ENV version=2.51.0-2.fc42",
            "FROM fedora:fc42",
            "RUN yum -y install git-core-${version}",
            "",
        ]
        .join("\n");
        rule_catches("DL3033", &snippet);
    }
}
