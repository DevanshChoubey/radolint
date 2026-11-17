// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3020.hs.
//!
//! DIVERGENCE (docs/divergences.md, hadolint #769): a source that uses variables, such as
//! `ADD ${URL} /app/`, is expanded with the ARG and ENV values of its build stage before it is
//! checked, as Docker does. When a value is unknown (an ARG without a default, or a variable
//! that may come from the base image), the source is not reported. hadolint checks the
//! variable text itself, so `ADD $URL` is always reported.

use crate::arg_vars::Vars;
use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, State, add_fail, archive_file_format_extensions,
    custom_rule, drop_quotes, empty_state, modify,
};
use radolint_syntax::{Instruction, Linenumber};

#[derive(Clone, Default)]
struct St {
    /// ARGs declared before the first FROM.
    global: Vars,
    /// ARG and ENV values in the current build stage.
    stage: Vars,
    seen_from: bool,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3020";
    let severity = DLSeverity::DLErrorC;
    let message = "Use COPY instead of ADD for files and folders";
    let check = move |line: Linenumber, st: State<St>, instr: &Instruction<_>| match instr {
        Instruction::Arg(name, value) => modify(
            |mut s: St| {
                if s.seen_from {
                    let global = s.global.clone();
                    s.stage.record_arg(name, value.as_deref(), Some(&global));
                } else {
                    s.global.record_arg(name, value.as_deref(), None);
                }
                s
            },
            st,
        ),
        Instruction::Env(pairs) => modify(
            |mut s: St| {
                for (k, v) in pairs {
                    let v = if v.contains('$') {
                        s.stage.expand(v)
                    } else {
                        Some(v.clone())
                    };
                    s.stage.set(k, v);
                }
                s
            },
            st,
        ),
        Instruction::From(_) => modify(
            |mut s: St| {
                s.seen_from = true;
                s.stage = Vars::stage();
                s
            },
            st,
        ),
        Instruction::Add(add_args, _) => {
            let ok = add_args.source_paths.iter().all(|src| {
                let path = &src.un_source_path;
                if is_archive(path) || is_url(path) {
                    return true;
                }
                if !path.contains('$') {
                    return false;
                }
                let vars = if st.state.seen_from {
                    &st.state.stage
                } else {
                    &st.state.global
                };
                let path = drop_quotes(path);
                match vars.expand(&path) {
                    // An unknown value may be a URL or an archive, unless the
                    // literal text around the variables rules both out.
                    None => !literal_rules_out_url_and_archive(&path),
                    Some(p) => is_archive(&p) || is_url(&p),
                }
            });
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
        _ => st,
    };
    custom_rule(check, empty_state(St::default()))
}

/// Whether the literal text of a source with an unknown variable already
/// shows that it is neither a URL nor an archive: it starts with text that is
/// not `http://` or `https://`, and ends with text that is not an archive
/// extension (`conf/${ENV}.json`). `$URL`, `$BASE/x.txt` and `files/$F` stay
/// unknown.
fn literal_rules_out_url_and_archive(path: &str) -> bool {
    let Some(first) = path.find('$') else {
        return false;
    };
    let prefix = &path[..first];
    let suffix = text_after_last_variable(path);
    let could_be_url = ["https://", "http://"]
        .iter()
        .any(|proto| prefix.starts_with(proto) || proto.starts_with(prefix));
    let could_be_archive = archive_file_format_extensions()
        .iter()
        .any(|ext| suffix.ends_with(ext) || ext.ends_with(suffix));
    !prefix.is_empty() && !suffix.is_empty() && !could_be_url && !could_be_archive
}

/// The text after the last `$NAME` or `${...}` in a path.
fn text_after_last_variable(path: &str) -> &str {
    let Some(last) = path.rfind('$') else {
        return path;
    };
    let rest = &path[last + 1..];
    if let Some(body) = rest.strip_prefix('{') {
        match body.find('}') {
            Some(end) => &body[end + 1..],
            None => "",
        }
    } else {
        let len = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        &rest[len..]
    }
}

/// Haskell `isArchive`.
fn is_archive(path: &str) -> bool {
    let p = drop_quotes(path);
    archive_file_format_extensions()
        .iter()
        .any(|ftype| p.ends_with(ftype))
}

/// Haskell `isUrl`.
fn is_url(path: &str) -> bool {
    let p = drop_quotes(path);
    ["https://", "http://"]
        .iter()
        .any(|proto| p.starts_with(proto))
}

#[cfg(test)]
mod tests {
    use crate::testing::{rule_catches, rule_catches_not};

    fn df(lines: &[&str]) -> String {
        lines.join("\n") + "\n"
    }

    #[test]
    fn add_for_tar() {
        rule_catches_not("DL3020", "ADD file.tar /usr/src/app/");
    }
    #[test]
    fn add_for_gzip() {
        rule_catches_not("DL3020", "ADD file.gz /usr/src/app/");
    }
    #[test]
    fn add_for_bz2() {
        rule_catches_not("DL3020", "ADD file.bz2 /usr/src/app/");
    }
    #[test]
    fn add_for_xz() {
        rule_catches_not("DL3020", "ADD file.xz /usr/src/app/");
    }
    #[test]
    fn add_for_tgz() {
        rule_catches_not("DL3020", "ADD file.tgz /usr/src/app/");
    }
    #[test]
    fn add_for_url() {
        rule_catches_not("DL3020", "ADD http://file.com /usr/src/app/");
    }
    #[test]
    fn using_add() {
        rule_catches("DL3020", "ADD file /usr/src/app/");
    }
    #[test]
    fn warn_for_zip() {
        rule_catches("DL3020", "ADD file.zip /usr/src/app/");
    }
    #[test]
    fn add_for_tgz_with_quotes() {
        rule_catches_not("DL3020", "ADD \"file.tgz\" /usr/src/app/");
    }
    #[test]
    fn add_for_url_with_quotes() {
        rule_catches_not("DL3020", "ADD \"http://file.com\" /usr/src/app/");
    }
    #[test]
    fn warn_for_zip_with_quotes() {
        rule_catches("DL3020", "ADD \"file.zip\" /usr/src/app/");
    }

    // DIVERGENCE: hadolint #769. Sources are expanded with the stage's ARG
    // and ENV values.
    #[test]
    fn add_for_url_from_a_stage_arg() {
        rule_catches_not(
            "DL3020",
            &df(&[
                "FROM scratch",
                "ARG URL=https://example.com/a.bin",
                "ADD ${URL} /app/",
            ]),
        );
    }
    #[test]
    fn add_for_archive_from_an_env() {
        rule_catches_not(
            "DL3020",
            &df(&["FROM scratch", "ENV V=1.2", "ADD app-$V.tar.gz /app/"]),
        );
    }
    #[test]
    fn add_for_url_from_a_redeclared_global_arg() {
        rule_catches_not(
            "DL3020",
            &df(&[
                "ARG URL=https://example.com/a.bin",
                "FROM scratch",
                "ARG URL",
                "ADD $URL /app/",
            ]),
        );
    }
    #[test]
    fn add_for_a_plain_file_from_an_arg() {
        rule_catches(
            "DL3020",
            &df(&["FROM scratch", "ARG F=config.json", "ADD $F /app/"]),
        );
    }
    #[test]
    fn global_arg_not_redeclared_is_not_used_in_the_stage() {
        // Without `ARG F` in the stage the global value does not apply, and
        // F may come from the base image, so it is unknown.
        rule_catches_not(
            "DL3020",
            &df(&["ARG F=config.json", "FROM scratch", "ADD $F /app/"]),
        );
    }
    #[test]
    fn arg_without_a_default_is_not_reported() {
        rule_catches_not(
            "DL3020",
            &df(&["FROM scratch", "ARG URL", "ADD $URL /app/"]),
        );
    }
    #[test]
    fn variable_from_the_base_image_is_not_reported() {
        rule_catches_not("DL3020", &df(&["FROM node", "ADD $SOMEWHERE /app/"]));
    }
    #[test]
    fn stage_values_reset_at_from() {
        rule_catches(
            "DL3020",
            &df(&[
                "FROM scratch",
                "ARG F=https://example.com/a.bin",
                "FROM scratch",
                "ARG F=local.txt",
                "ADD $F /app/",
            ]),
        );
    }

    #[test]
    fn literal_text_that_rules_out_url_and_archive_is_reported() {
        rule_catches(
            "DL3020",
            &df(&["FROM debian", "ARG ENV", "ADD conf/${ENV}.json /etc/"]),
        );
        rule_catches("DL3020", &df(&["FROM debian", "ADD src/$NAME.txt /app/"]));
    }
    #[test]
    fn literal_text_that_may_be_a_url_or_archive_is_not_reported() {
        for src in [
            "$URL",
            "$BASE/x.txt",
            "files/$F",
            "https://$HOST/a.json",
            "src/$NAME.tar.gz",
            "src/${NAME}gz",
        ] {
            rule_catches_not("DL3020", &df(&["FROM debian", &format!("ADD {src} /app/")]));
        }
    }
}
