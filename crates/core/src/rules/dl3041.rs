// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3041.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, State, add_fail, combine, custom_rule, empty_state,
    fold_arguments, modify, onbuild,
};
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
    combine(dl3041(), onbuild(dl3041()))
}

/// Haskell `dl3041`.
fn dl3041() -> Box<dyn Rule> {
    let code = "DL3041";
    let severity = DLSeverity::DLWarningC;
    let message = "Specify version with `dnf install -y <package>-<version>`.";

    let check =
        move |line: Linenumber, st: State<Acc>, instr: &Instruction<ParsedShell>| -> State<Acc> {
            match instr {
                Instruction::Run(RunArgs(a, _)) => {
                    let ok = fold_arguments(
                        |s: &ParsedShell| {
                            dnf_packages(s)
                                .iter()
                                .all(|p| package_version_fixed(&st.state, p))
                        },
                        a,
                    ) && fold_arguments(
                        |s: &ParsedShell| dnf_modules(s).iter().all(|m| module_version_fixed(m)),
                        a,
                    );
                    if ok {
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

const DNF_CMDS: [&str; 2] = ["dnf", "microdnf"];

/// Haskell `dnfPackages`.
fn dnf_packages(args: &ParsedShell) -> Vec<String> {
    let mut out = vec![];
    for cmd in &args.present_commands {
        if !shell::cmds_have_args(&DNF_CMDS, &["module", "group"], cmd) {
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
            envs.get(stage_idx)
                .is_some_and(|s| s.iter().any(|v| var_in_text(v, package)))
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

/// Haskell `dnfModules`.
fn dnf_modules(args: &ParsedShell) -> Vec<String> {
    let mut out = vec![];
    for cmd in &args.present_commands {
        if shell::cmds_have_args(&DNF_CMDS, &["module"], cmd) {
            out.extend(install_filter(cmd));
        }
    }
    out
}

/// Haskell `moduleVersionFixed`.
fn module_version_fixed(m: &str) -> bool {
    m.contains(':')
}

/// Haskell `installFilter`.
fn install_filter(cmd: &shell::Command) -> Vec<String> {
    if !shell::cmds_have_args(&DNF_CMDS, &["install"], cmd) {
        return vec![];
    }
    shell::get_args_no_flags(cmd)
        .into_iter()
        .filter(|a| a != "install" && a != "module")
        .collect()
}

/// Haskell `registerEnvs`.
fn register_envs(pairs: &Pairs, acc: Acc) -> Acc {
    let names: BTreeSet<String> = pairs.iter().map(|(k, _)| k.clone()).collect();
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
            let stage_idx_of_alias: i64 = match stage_aliases
                .iter()
                .find(|(_, v)| **v == bi.image.image_name)
            {
                None => stage_idx + 1,
                Some((k, _)) => *k,
            };
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

    const R: &str = "DL3041";

    fn both_catch(df: &str) {
        rule_catches(R, df);
    }

    #[test]
    fn not_ok_without_dnf_version_pinning() {
        rule_catches(R, "RUN dnf install -y tomcat && dnf clean all");
        rule_catches(R, "RUN microdnf install -y tomcat && microdnf clean all");
        on_build_rule_catches(R, "RUN dnf install -y tomcat && dnf clean all");
    }

    #[test]
    fn not_ok_without_dnf_version_pinning_package_name_with_dash() {
        rule_catches(R, "RUN dnf install -y rpm-sign && dnf clean all");
        rule_catches(R, "RUN microdnf install -y rpm-sign && microdnf clean all");
        on_build_rule_catches(R, "RUN dnf install -y rpm-sign && dnf clean all");
    }

    #[test]
    fn ok_with_dnf_version_pinning() {
        rule_catches_not(R, "RUN dnf install -y tomcat-9.0.1 && dnf clean all");
        rule_catches_not(
            R,
            "RUN microdnf install -y tomcat-9.0.1 && microdnf clean all",
        );
        on_build_rule_catches_not(R, "RUN dnf install -y tomcat-9.0.1 && dnf clean all");
        on_build_rule_catches_not(
            R,
            "RUN microdnf install -y tomcat-9.0.1 && microdnf clean all",
        );
    }

    #[test]
    fn ok_with_version_pinning_if_command_is_not_dnf_or_microdnf() {
        rule_catches_not(R, "RUN notdnf install openssl-1:1.1.1k");
        on_build_rule_catches_not(R, "RUN notdnf install openssl-1:1.1.1k");
    }

    #[test]
    fn ok_without_version_pinning_if_command_is_not_dnf_or_microdnf() {
        rule_catches_not(R, "RUN notdnf install tomcat");
        on_build_rule_catches_not(R, "RUN notdnf install tomcat");
    }

    #[test]
    fn ok_with_dnf_version_pinning_package_name_with_dash() {
        rule_catches_not(R, "RUN dnf install -y rpm-sign-4.16.1.3 && dnf clean all");
        rule_catches_not(
            R,
            "RUN microdnf install -y rpm-sign-4.16.1.3 && microdnf clean all",
        );
        on_build_rule_catches_not(R, "RUN dnf install -y rpm-sign-4.16.1.3 && dnf clean all");
        on_build_rule_catches_not(
            R,
            "RUN microdnf install -y rpm-sign-4.16.1.3 && microdnf clean all",
        );
    }

    #[test]
    fn ok_with_dnf_version_pinning_package_name_with_dash_and_plus() {
        rule_catches_not(R, "RUN dnf install -y gcc-c++-1.1.1");
        rule_catches_not(R, "RUN microdnf install -y gcc-c++-1.1.1");
        on_build_rule_catches_not(R, "RUN dnf install -y gcc-c++-1.1.1");
        on_build_rule_catches_not(R, "RUN microdnf install -y gcc-c++-1.1.1");
    }

    #[test]
    fn ok_with_dnf_version_pinning_package_version_with_epoch() {
        rule_catches_not(R, "RUN dnf install -y openssl-1:1.1.1k");
        rule_catches_not(R, "RUN microdnf install -y openssl-1:1.1.1k");
        on_build_rule_catches_not(R, "RUN dnf install -y openssl-1:1.1.1k");
        on_build_rule_catches_not(R, "RUN microdnf install -y openssl-1:1.1.1k");
    }

    #[test]
    fn not_ok_without_dnf_version_pinning_modules() {
        rule_catches(R, "RUN dnf module install -y tomcat && dnf clean all");
        rule_catches(
            R,
            "RUN microdnf module install -y tomcat && microdnf clean all",
        );
        on_build_rule_catches(R, "RUN dnf module install -y tomcat && dnf clean all");
    }

    #[test]
    fn ok_with_dnf_version_pinning_modules() {
        rule_catches_not(R, "RUN dnf module install -y tomcat:9 && dnf clean all");
        rule_catches_not(
            R,
            "RUN microdnf module install -y tomcat:9 && microdnf clean all",
        );
        rule_catches_not(R, "RUN notdnf module install tomcat");
        on_build_rule_catches_not(R, "RUN dnf module install -y tomcat:9 && dnf clean all");
        on_build_rule_catches_not(
            R,
            "RUN microdnf module install -y tomcat:9 && microdnf clean all",
        );
        on_build_rule_catches_not(R, "RUN notdnf module install tomcat");
    }

    #[test]
    fn ok_with_dnf_group_install() {
        rule_catches_not(R, "RUN dnf -y group install \"Development Tools\"");
        rule_catches_not(
            R,
            "RUN dnf -y --setopt=group_package_types=\"mandatory\" group install \"Development Tools\"",
        );
        rule_catches_not(
            R,
            "RUN dnf group install -y \"Development Tools\" && dnf clean all",
        );
        rule_catches_not(
            R,
            "RUN microdnf group install -y \"Development Tools\" && microdnf clean all",
        );
        on_build_rule_catches_not(R, "RUN dnf -y group install \"Development Tools\"");
    }

    #[test]
    fn ok_with_version_as_variable_braced() {
        rule_catches_not(
            R,
            "ENV version=2.51.0-2.fc42\nRUN dnf -y install git-core-${version}\n",
        );
    }

    #[test]
    fn ok_with_version_as_arg_unbraced() {
        // upstream spec expects no failure even though only `${var}` form is matched
        rule_catches_not(
            R,
            "ARG version=2.51.0-2.fc42\nRUN dnf -y install git-core-$version\n",
        );
    }

    #[test]
    fn ok_with_version_as_arg_different_stages() {
        rule_catches_not(
            R,
            "FROM fedora:fc42 AS build\nARG version=2.51.0-2.fc42\nFROM fedora:fc42\nRUN dnf -y install git-core-${version}\n",
        );
    }

    #[test]
    fn ok_with_version_as_env_different_stages_reused_stage() {
        rule_catches_not(
            R,
            "FROM fedora:fc42 AS build\nENV version=2.51.0-2.fc42\nFROM build\nRUN dnf -y install git-core-${version}\n",
        );
    }

    #[test]
    fn not_ok_with_version_as_variable_different_stages_new_stage() {
        both_catch(
            "FROM fedora:fc42 AS build\nENV version=2.51.0-2.fc42\nFROM fedora:fc42\nRUN dnf -y install git-core-${version}\n",
        );
    }
}
