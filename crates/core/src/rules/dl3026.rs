// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3026.hs.
//!
//! DIVERGENCE (two, both in docs/divergences.md):
//! - A trusted registry may include a path, such as `gcr.io/distroless`, and then allows the
//!   images under that path (hadolint/hadolint#401). hadolint only compares the host part, so
//!   such an entry never matches and `gcr.io` was the only way to trust those images.
//! - A FROM image that uses variables, such as `FROM $BASE` or `FROM ${REG}/app`, is expanded
//!   with the defaults of the ARGs declared before the first FROM, as Docker does. When the
//!   registry depends on an ARG without a default, it is unknown and the image is not reported. hadolint takes the variable text as a Docker Hub image name and
//!   reports it whenever docker.io is not trusted.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, State, add_fail, custom_rule, empty_state, modify,
};
use radolint_syntax::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Default)]
struct St {
    aliases: BTreeSet<Option<ImageAlias>>,
    /// ARGs declared before the first FROM, with their defaults.
    global_args: BTreeMap<String, Option<String>>,
    seen_from: bool,
}

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

    let is_registry_allowed_from = is_registry_allowed.clone();
    let allowed_from = allowed.clone();

    let is_path_allowed = {
        let allowed = allowed.clone();
        move |paths: &[String]| {
            allowed
                .iter()
                .any(|p| paths.iter().any(|path| match_path(&p.un_registry, path)))
        }
    };

    let is_allowed = move |img: &Image| {
        let registry_ok = match &img.registry_name {
            Some(registry) => is_registry_allowed(&registry.un_registry),
            None => {
                img.image_name == "scratch"
                    || is_registry_allowed("docker.io")
                    || is_registry_allowed("hub.docker.com")
            }
        };
        registry_ok || is_path_allowed(&image_paths(img))
    };

    let to_image_alias = |img: &Image| {
        Some(ImageAlias {
            un_image_alias: img.image_name.clone(),
        })
    };

    let do_check = {
        let allowed = allowed.clone();
        move |st: &St, img: &Image| {
            st.aliases.contains(&to_image_alias(img)) || allowed.is_empty() || is_allowed(img)
        }
    };

    let check_mount = {
        let do_check = do_check.clone();
        move |st: &St, m: &RunMount| match m {
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

    let check =
        move |line: Linenumber, st: State<St>, instr: &Instruction<crate::shell::ParsedShell>| {
            match instr {
                Instruction::Arg(name, value) if !st.state.seen_from => modify(
                    |mut s: St| {
                        record_args(&mut s.global_args, name, value.as_deref());
                        s
                    },
                    st,
                ),
                Instruction::From(BaseImage { image, alias, .. }) => {
                    let ok = match resolve_image(image, &st.state.global_args) {
                        Some(img) => do_check(&st.state, &img),
                        None => match &image.registry_name {
                            // Only the path or name is unknown (`evil.com/${ORG}/img`): the
                            // literal registry still decides, unless a trusted entry with a
                            // path under it could match once the variable is known.
                            Some(r) if !r.un_registry.contains('$') => {
                                allowed_from.is_empty()
                                    || is_registry_allowed_from(&r.un_registry)
                                    || allowed_from.iter().any(|a| {
                                        a.un_registry.contains('/')
                                            && (a.un_registry.contains('*')
                                                || a.un_registry
                                                    .starts_with(&format!("{}/", r.un_registry)))
                                    })
                            }
                            // The registry depends on a build argument we cannot know.
                            _ => true,
                        },
                    };
                    let new_state = modify(
                        |mut s: St| {
                            s.aliases.insert(alias.clone());
                            s.seen_from = true;
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
                    if m.iter().all(|x| check_mount(&st.state, x)) {
                        st
                    } else {
                        add_fail(fail(line), st)
                    }
                }
                _ => st,
            }
        };

    custom_rule(check, empty_state(St::default()))
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

/// Whether a trusted entry with a path (`gcr.io/distroless`, `reg.io/team/*`) allows the image
/// at `path` (`gcr.io/distroless/base`). A plain entry matches whole path components only, so
/// `gcr.io/dist` does not allow `gcr.io/distroless/base`.
fn match_path(allow: &str, path: &str) -> bool {
    if !allow.contains('/') {
        return false;
    }
    if allow.contains('*') {
        return match_registry(allow, path);
    }
    let allow = allow.trim_end_matches('/');
    path == allow || path.starts_with(&format!("{allow}/"))
}

/// The full paths of an image without its tag or digest: `gcr.io/distroless/base`, or for a
/// Docker Hub image both `docker.io/library/debian` and `docker.io/debian`.
fn image_paths(img: &Image) -> Vec<String> {
    let mut name = img.image_name.as_str();
    if let Some(at) = name.find('@') {
        name = &name[..at];
    }
    let last = name.rfind('/').map_or(0, |i| i + 1);
    if let Some(colon) = name[last..].find(':') {
        name = &name[..last + colon];
    }
    match &img.registry_name {
        Some(r) => vec![format!("{}/{name}", r.un_registry)],
        None if name.contains('/') => vec![format!("docker.io/{name}")],
        None => vec![
            format!("docker.io/library/{name}"),
            format!("docker.io/{name}"),
        ],
    }
}

/// The image with its variables expanded from the global ARG defaults, or `None` when that is
/// not possible. An image without variables is returned as it is.
fn resolve_image(img: &Image, args: &BTreeMap<String, Option<String>>) -> Option<Image> {
    let full = match &img.registry_name {
        Some(r) => format!("{}/{}", r.un_registry, img.image_name),
        None => img.image_name.clone(),
    };
    if !full.contains('$') {
        return Some(img.clone());
    }
    let expanded = expand(&full, args)?;
    if expanded.is_empty() || expanded.contains('$') {
        return None;
    }
    // A default can carry its own tag or digest (`ARG BASE=foo.com/img:1`).
    let mut out = Image::from(expanded.as_str());
    if let Some(at) = out.image_name.find('@') {
        out.image_name.truncate(at);
    }
    let last = out.image_name.rfind('/').map_or(0, |i| i + 1);
    if let Some(colon) = out.image_name[last..].find(':') {
        out.image_name.truncate(last + colon);
    }
    Some(out)
}

/// Expands `$NAME`, `${NAME}`, `${NAME:-word}`, `${NAME-word}`, `${NAME:+word}` and
/// `${NAME+word}` the way BuildKit does for FROM. `None` means a value is unknown: the variable is
/// an ARG without a default (it can only come from `--build-arg`). Other names are unset.
fn expand(s: &str, args: &BTreeMap<String, Option<String>>) -> Option<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '$' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        i += 1;
        if chars.get(i) == Some(&'{') {
            // Find the matching close brace.
            let start = i + 1;
            let mut depth = 1;
            let mut j = start;
            while j < chars.len() && depth > 0 {
                match chars[j] {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                j += 1;
            }
            if depth != 0 {
                return None;
            }
            let body: String = chars[start..j - 1].iter().collect();
            out.push_str(&expand_braced(&body, args)?);
            i = j;
        } else {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            if start == i {
                out.push('$');
                continue;
            }
            let name: String = chars[start..i].iter().collect();
            out.push_str(&lookup(&name, args)?.unwrap_or_default());
        }
    }
    Some(out)
}

/// The value of a variable in FROM. `None`: unknown (an ARG declared without a default, which
/// only `--build-arg` can set). `Some(None)`: unset (not a global ARG, so nothing can set it).
fn lookup(name: &str, args: &BTreeMap<String, Option<String>>) -> Option<Option<String>> {
    match args.get(name) {
        Some(v) => v.clone().map(Some),
        None => Some(None),
    }
}

/// Records the variables of one global `ARG` line. The parser keeps everything after the first
/// `=` as one value, so `ARG A=x B=y` arrives as `A` = `x B=y` and is split here. Defaults that
/// refer to earlier ARGs (`ARG IMG=${REG}/app`) are expanded now, as Docker does.
fn record_args(args: &mut BTreeMap<String, Option<String>>, name: &str, value: Option<&str>) {
    let mut pairs: Vec<(String, Option<String>)> = Vec::new();
    match value {
        None => pairs.push((name.to_string(), None)),
        Some(v) => {
            let mut words = split_words(v).into_iter();
            pairs.push((name.to_string(), Some(words.next().unwrap_or_default())));
            for w in words {
                match w.split_once('=') {
                    Some((k, v)) => pairs.push((k.to_string(), Some(unquote(v).to_string()))),
                    None => pairs.push((w, None)),
                }
            }
        }
    }
    for (k, v) in pairs {
        let v = match v {
            Some(v) if v.contains('$') => expand(&v, args),
            v => v,
        };
        args.insert(k, v);
    }
}

/// Splits an ARG value on unquoted whitespace, removing the quotes.
fn split_words(v: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut in_word = false;
    for c in v.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => cur.push(c),
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                in_word = true;
            }
            None if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut cur));
                    in_word = false;
                }
            }
            None => {
                cur.push(c);
                in_word = true;
            }
        }
    }
    if in_word {
        words.push(cur);
    }
    words
}

fn unquote(v: &str) -> &str {
    // Values are unquoted by `split_words`; this handles `K="v"` pairs split on `=`.
    for q in ['"', '\''] {
        if v.len() >= 2 && v.starts_with(q) && v.ends_with(q) {
            return &v[1..v.len() - 1];
        }
    }
    v
}

fn expand_braced(body: &str, args: &BTreeMap<String, Option<String>>) -> Option<String> {
    let name_len = body
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(body.len());
    let (name, rest) = body.split_at(name_len);
    let value = lookup(name, args)?;
    let (op, word) = if let Some(w) = rest.strip_prefix(":-") {
        (":-", w)
    } else if let Some(w) = rest.strip_prefix(":+") {
        (":+", w)
    } else if let Some(w) = rest.strip_prefix('-') {
        ("-", w)
    } else if let Some(w) = rest.strip_prefix('+') {
        ("+", w)
    } else if rest.is_empty() {
        return Some(value.unwrap_or_default());
    } else {
        // `${NAME:?msg}` and other forms: leave the value unknown.
        return None;
    };
    match (op, value) {
        (":-", None) | ("-", None) => expand(word, args),
        (":-", Some(v)) if v.is_empty() => expand(word, args),
        (":+", Some(v)) if !v.is_empty() => expand(word, args),
        ("+", Some(_)) => expand(word, args),
        (":+", _) | ("+", None) => Some(String::new()),
        (_, None) => None,
        (_, Some(v)) => Some(v),
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

    // hadolint/hadolint#401: a trusted entry can include a path.
    #[test]
    fn allows_image_under_trusted_path() {
        rule_catches_not_with(
            &cfg(&["gcr.io/distroless"]),
            "DL3026",
            "FROM gcr.io/distroless/base-debian10:latest\n",
        );
    }

    #[test]
    fn trusted_path_matches_whole_components() {
        rule_catches_with(
            &cfg(&["gcr.io/dist"]),
            "DL3026",
            "FROM gcr.io/distroless/base-debian10:1\n",
        );
    }

    #[test]
    fn warns_on_image_outside_trusted_path() {
        rule_catches_with(
            &cfg(&["gcr.io/distroless"]),
            "DL3026",
            "FROM gcr.io/my-project/app:1\n",
        );
    }

    #[test]
    fn trusted_path_with_wildcard() {
        rule_catches_not_with(
            &cfg(&["reg.io/approved/*"]),
            "DL3026",
            "FROM reg.io/approved/app:1\n",
        );
    }

    #[test]
    fn trusted_docker_hub_namespace() {
        rule_catches_not_with(&cfg(&["docker.io/library"]), "DL3026", "FROM debian:12\n");
        rule_catches_with(
            &cfg(&["docker.io/library"]),
            "DL3026",
            "FROM someone/debian:12\n",
        );
    }

    #[test]
    fn copy_from_trusted_path() {
        rule_catches_not_with(
            &cfg(&["reg.io/team"]),
            "DL3026",
            "COPY --from=reg.io/team/tools:1 /a /b\n",
        );
    }

    #[test]
    fn localhost_with_port_is_a_registry() {
        rule_catches_not_with(
            &cfg(&["localhost:5000"]),
            "DL3026",
            "FROM localhost:5000/bash:1\n",
        );
        rule_catches_with(
            &cfg(&["docker.io"]),
            "DL3026",
            "FROM localhost:5000/bash:1\n",
        );
    }

    // hadolint/hadolint#339: FROM images built from ARGs.
    #[test]
    fn expands_global_arg_default() {
        let cfg = cfg(&["foo.com"]);
        let ok = "ARG TOGETHER=foo.com/image:tag\nFROM $TOGETHER\n\nARG IMAGE=foo.com/image\nARG VERSION=tag\nFROM $IMAGE:$VERSION\n";
        rule_catches_not_with(&cfg, "DL3026", ok);
    }

    #[test]
    fn expands_braced_default_and_reports_untrusted() {
        rule_catches_with(
            &cfg(&["foo.com"]),
            "DL3026",
            "ARG REG=bar.com\nFROM ${REG}/app:1\n",
        );
        rule_catches_with(
            &cfg(&["foo.com"]),
            "DL3026",
            "ARG BASE=\"\"\nFROM ${BASE:-debian}:12\n",
        );
    }

    #[test]
    fn unknown_build_arg_is_not_reported() {
        let cfg = cfg(&["foo.com"]);
        rule_catches_not_with(&cfg, "DL3026", "ARG BASE\nFROM $BASE\n");
        rule_catches_not_with(&cfg, "DL3026", "FROM $UNDECLARED\n");
        rule_catches_not_with(&cfg, "DL3026", "ARG BASE\nFROM ${BASE:-debian:9}\n");
    }

    #[test]
    fn arg_inside_a_stage_does_not_feed_from() {
        rule_catches_not_with(
            &cfg(&["foo.com"]),
            "DL3026",
            "FROM foo.com/a:1\nARG NEXT=bar.com/b:1\nFROM $NEXT\n",
        );
    }

    #[test]
    fn expanded_previous_stage_alias() {
        rule_catches_not_with(
            &cfg(&["foo.com"]),
            "DL3026",
            "ARG STAGE=build\nFROM foo.com/a:1 AS build\nFROM ${STAGE}\n",
        );
    }

    #[test]
    fn literal_registry_is_checked_when_the_path_is_a_variable() {
        let gcr = cfg(&["gcr.io"]);
        rule_catches_with(&gcr, "DL3026", "ARG ORG\nFROM evil.com/${ORG}/img:1\n");
        rule_catches_not_with(&gcr, "DL3026", "ARG ORG\nFROM gcr.io/${ORG}/img:1\n");
        // A trusted path under the registry could match once ORG is known.
        rule_catches_not_with(
            &cfg(&["gcr.io/distroless"]),
            "DL3026",
            "ARG ORG\nFROM gcr.io/${ORG}/img:1\n",
        );
    }

    #[test]
    fn several_pairs_on_one_arg_line() {
        let gcr = cfg(&["gcr.io"]);
        rule_catches_not_with(&gcr, "DL3026", "ARG A=gcr.io B=evil.com\nFROM ${A}/a:1\n");
        rule_catches_with(&gcr, "DL3026", "ARG A=gcr.io B=evil.com\nFROM ${B}/a:1\n");
        rule_catches_with(
            &gcr,
            "DL3026",
            "ARG A=\"gcr.io\" B='evil.com'\nFROM ${B}/a:1\n",
        );
        rule_catches_not_with(&gcr, "DL3026", "ARG A=gcr.io B\nFROM ${B}/a:1\n");
    }

    #[test]
    fn undeclared_variable_is_unset() {
        let gcr = cfg(&["gcr.io"]);
        rule_catches_with(&gcr, "DL3026", "FROM ${X:-evil.com}/a:1\n");
        rule_catches_not_with(&gcr, "DL3026", "FROM ${X:-gcr.io}/a:1\n");
        rule_catches_with(&gcr, "DL3026", "FROM ${X-evil.com}/a:1\n");
        rule_catches_with(&gcr, "DL3026", "FROM evil.com/a${X:+gcr.io}:1\n");
    }

    #[test]
    fn arg_defaults_refer_to_earlier_args() {
        let gcr = cfg(&["gcr.io"]);
        rule_catches_with(
            &gcr,
            "DL3026",
            "ARG R=evil.com\nARG IMG=${R}/x\nFROM $IMG\n",
        );
        rule_catches_not_with(&gcr, "DL3026", "ARG R=gcr.io\nARG IMG=${R}/x\nFROM $IMG\n");
        rule_catches_not_with(&gcr, "DL3026", "ARG R\nARG IMG=${R}/x\nFROM $IMG\n");
    }
}
