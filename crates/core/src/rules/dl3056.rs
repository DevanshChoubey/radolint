// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3056.hs.

use crate::rule::{DLSeverity, LabelName, LabelSchema, LabelType, Rule, mconcat, simple_rule};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule(label_schema: &LabelSchema) -> Box<dyn Rule> {
    // Map.keys (Map.filter (== SemVer) labelschema): BTreeMap iterates in key order.
    mconcat(
        label_schema
            .iter()
            .filter(|(_, t)| **t == LabelType::SemVer)
            .map(|(k, _)| label_is_not_sem_ver_rule(k.clone()))
            .collect(),
    )
}

fn label_is_not_sem_ver_rule(label: LabelName) -> Box<dyn Rule> {
    let code = "DL3056";
    let severity = DLSeverity::DLWarningC;
    let message = format!("Label `{}` does not conform to semantic versioning.", label);
    simple_rule(code, severity, &message, move |instr| match instr {
        Instruction::Label(pairs) => has_no_bad_versioning(&label, pairs),
        _ => true,
    })
}

fn has_no_bad_versioning(lbl: &str, prs: &Pairs) -> bool {
    !prs.iter().any(|(l, v)| l == lbl && !sem_ver_from_text(v))
}

// DIVERGENCE: Data.SemVer.fromText has no Rust equivalent here; this is a hand-written
// check of the semver 2.0 grammar (true when `fromText` would return Right).
fn sem_ver_from_text(t: &str) -> bool {
    fn numeric(s: &str) -> bool {
        !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) && (s == "0" || !s.starts_with('0'))
    }
    fn alnum(s: &str) -> bool {
        !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    }
    fn release_id(s: &str) -> bool {
        if s.bytes().all(|b| b.is_ascii_digit()) {
            numeric(s)
        } else {
            alnum(s)
        }
    }
    let (rest, meta) = match t.split_once('+') {
        Some((a, m)) => (a, Some(m)),
        None => (t, None),
    };
    if let Some(m) = meta
        && !m.split('.').all(alnum)
    {
        return false;
    }
    let (core, pre) = match rest.split_once('-') {
        Some((c, p)) => (c, Some(p)),
        None => (rest, None),
    };
    if let Some(p) = pre
        && !p.split('.').all(release_id)
    {
        return false;
    }
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3 && parts.iter().all(|p| numeric(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Configuration;
    use crate::testing::*;

    fn config() -> Configuration {
        Configuration {
            label_schema: [("semver".to_string(), LabelType::SemVer)]
                .into_iter()
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn not_ok_with_label_not_containing_semantic_version() {
        let c = config();
        rule_catches_with(&c, "DL3056", "LABEL semver=\"not-sem-ver\"");
        on_build_rule_catches_with(&c, "DL3056", "LABEL semver=\"not-sem-ver\"");
    }

    #[test]
    fn ok_with_label_containing_semantic_version() {
        let c = config();
        rule_catches_not_with(&c, "DL3056", "LABEL semver=\"1.0.0\"");
        on_build_rule_catches_not_with(&c, "DL3056", "LABEL semver=\"2.0.1-rc1\"");
    }

    #[test]
    fn ok_with_other_label_not_containing_semantic_version() {
        let c = config();
        rule_catches_not_with(&c, "DL3056", "LABEL other=\"foo\"");
        on_build_rule_catches_not_with(&c, "DL3056", "LABEL other=\"bar\"");
    }
}
