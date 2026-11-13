// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3026.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, State, add_fail, custom_rule, empty_state, modify,
};
use radolint_syntax::*;
use std::collections::BTreeSet;

/// Haskell `rule`.
pub fn rule(allowed: &BTreeSet<Registry>) -> Box<dyn Rule> {
    let allowed = allowed.clone();
    let code = "DL3026";
    let severity = DLSeverity::DLErrorC;
    let message = "Use only an allowed registry in the FROM image";

    let is_registry_allowed = {
        let allowed = allowed.clone();
        move |registry: &str| {
            allowed
                .iter()
                .any(|p| match_registry(&p.un_registry, registry))
        }
    };

    let is_allowed = {
        let is_registry_allowed = is_registry_allowed.clone();
        move |img: &Image| match &img.registry_name {
            Some(registry) => is_registry_allowed(&registry.un_registry),
            None => {
                img.image_name == "scratch"
                    || is_registry_allowed("docker.io")
                    || is_registry_allowed("hub.docker.com")
            }
        }
    };

    let to_image_alias = |img: &Image| {
        Some(ImageAlias {
            un_image_alias: img.image_name.clone(),
        })
    };

    let do_check = {
        let allowed = allowed.clone();
        move |st: &BTreeSet<Option<ImageAlias>>, img: &Image| {
            st.contains(&to_image_alias(img)) || allowed.is_empty() || is_allowed(img)
        }
    };

    let check_mount = {
        let do_check = do_check.clone();
        move |st: &BTreeSet<Option<ImageAlias>>, m: &RunMount| match m {
            RunMount::CacheMount(c) => {
                let fi = c
                    .c_from_image
                    .clone()
                    .unwrap_or_else(|| "scratch".to_string());
                do_check(st, &Image::from(fi.as_str()))
            }
            RunMount::BindMount(b) => {
                let fi = b
                    .b_from_image
                    .clone()
                    .unwrap_or_else(|| "scratch".to_string());
                do_check(st, &Image::from(fi.as_str()))
            }
            _ => true,
        }
    };

    let fail = move |line| CheckFailure {
        code: RuleCode::from(code),
        severity,
        message: message.to_string(),
        line,
    };

    let check = move |line: Linenumber,
                      st: State<BTreeSet<Option<ImageAlias>>>,
                      instr: &Instruction<crate::shell::ParsedShell>| {
        match instr {
            Instruction::From(BaseImage { image, alias, .. }) => {
                let ok = do_check(&st.state, image);
                let new_state = modify(
                    |mut s: BTreeSet<Option<ImageAlias>>| {
                        s.insert(alias.clone());
                        s
                    },
                    st,
                );
                if ok {
                    new_state
                } else {
                    add_fail(fail(line), new_state)
                }
            }
            Instruction::Copy(
                _,
                CopyFlags {
                    source_flag: CopySource::CopySource(src),
                    ..
                },
            ) => {
                let img = Image::from(src.as_str());
                if do_check(&st.state, &img) {
                    st
                } else {
                    add_fail(fail(line), st)
                }
            }
            Instruction::Run(RunArgs(_, RunFlags { mount: m, .. })) => {
                if m.iter().find(|x| !check_mount(&st.state, x)).is_none() {
                    st
                } else {
                    add_fail(fail(line), st)
                }
            }
            _ => st,
        }
    };

    custom_rule(check, empty_state(BTreeSet::new()))
}

/// Haskell `matchRegistry`.
fn match_registry(allow: &str, registry: &str) -> bool {
    let star = "*";
    if allow == star {
        true
    } else if allow.starts_with(star) {
        registry.ends_with(&allow[1..])
    } else if allow.ends_with(star) {
        registry.starts_with(&allow[..allow.len() - 1])
    } else {
        registry == allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::configuration::Configuration;
    use crate::testing::*;

    fn cfg(regs: &[&str]) -> Configuration {
        Configuration {
            allowed_registries: regs.iter().map(|r| Registry::from(*r)).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn does_not_warn_on_empty_allowed_registries() {
        rule_catches_not("DL3026", "FROM random.com/debian\n");
    }

    #[test]
    fn warn_on_non_allowed_registry() {
        rule_catches_with(&cfg(&["docker.io"]), "DL3026", "FROM random.com/debian\n");
    }

    #[test]
    fn does_not_warn_on_allowed_registries() {
        rule_catches_not_with(
            &cfg(&["x.com", "random.com"]),
            "DL3026",
            "FROM random.com/debian\n",
        );
    }

    #[test]
    fn doesnt_warn_on_scratch_image() {
        rule_catches_not_with(&cfg(&["x.com", "random.com"]), "DL3026", "FROM scratch\n");
    }

    const THREE: &str = "FROM ubuntu:18.04 AS builder1\nFROM zemanlx/ubuntu:18.04 AS builder2\nFROM docker.io/zemanlx/ubuntu:18.04 AS builder3\n";

    #[test]
    fn allows_all_forms_of_docker_io() {
        rule_catches_not_with(&cfg(&["docker.io"]), "DL3026", THREE);
    }

    #[test]
    fn allows_using_previous_stages() {
        rule_catches_not_with(
            &cfg(&["random.com"]),
            "DL3026",
            "FROM random.com/foo AS builder1\nFROM builder1 AS builder2\n",
        );
    }

    #[test]
    fn warn_on_non_allowed_wildcard_registry() {
        rule_catches_with(&cfg(&["*.random.com"]), "DL3026", "FROM x.com/debian\n");
    }

    #[test]
    fn does_not_warn_on_allowed_wildcard_registries() {
        rule_catches_not_with(
            &cfg(&["x.com", "*.random.com"]),
            "DL3026",
            "FROM foo.random.com/debian\n",
        );
    }

    #[test]
    fn does_not_warn_on_star_registry() {
        rule_catches_not_with(&cfg(&["*"]), "DL3026", THREE);
    }

    #[test]
    fn does_warn_on_copy_from_untrusted_registry() {
        rule_catches_with(
            &cfg(&["trusted.com"]),
            "DL3026",
            "COPY --from=untrusted.com/repo/image:tag /foo /bar\n",
        );
    }

    #[test]
    fn does_not_warn_on_copy_from_trusted_registry() {
        rule_catches_not_with(
            &cfg(&["trusted.com"]),
            "DL3026",
            "COPY --from=trusted.com/repo/image:tag /foo /bar\n",
        );
    }

    #[test]
    fn warn_on_run_with_bind_mount_from_untrusted_registry() {
        rule_catches_with(
            &cfg(&["trusted.com"]),
            "DL3026",
            "RUN --mount=type=bind,from=untrusted.com/repo/image:tag,target=/foo foobar\n",
        );
    }

    #[test]
    fn warn_on_run_with_cache_mount_from_untrusted_registry() {
        rule_catches_with(
            &cfg(&["trusted.com"]),
            "DL3026",
            "RUN --mount=type=cache,from=untrusted.com/repo/image:tag,target=/foo foobar\n",
        );
    }

    #[test]
    fn dont_warn_on_run_with_bind_mount_from_trusted_registry() {
        rule_catches_not_with(
            &cfg(&["trusted.com"]),
            "DL3026",
            "RUN --mount=type=bind,from=trusted.com/repo/image:tag,target=/foo foobar\n",
        );
    }

    #[test]
    fn dont_warn_on_run_with_cache_mount_from_trusted_registry() {
        rule_catches_not_with(
            &cfg(&["trusted.com"]),
            "DL3026",
            "RUN --mount=type=cache,from=trusted.com/repo/image:tag,target=/foo foobar\n",
        );
    }

    #[test]
    fn distrust_all_forms_of_docker_io_if_trusted_registries_are_given() {
        assert_checks_with(&cfg(&["trusted.com"]), THREE, |f| {
            fails_with(3, &RuleCode::from("DL3026"), &f)
        });
    }
}
