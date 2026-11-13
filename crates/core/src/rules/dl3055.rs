// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3055.hs.

use crate::rule::{DLSeverity, LabelName, LabelSchema, LabelType, Rule, mconcat, simple_rule};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule(label_schema: &LabelSchema) -> Box<dyn Rule> {
    // Map.keys (Map.filter (== GitHash) labelschema): BTreeMap iterates in key order.
    mconcat(
        label_schema
            .iter()
            .filter(|(_, t)| **t == LabelType::GitHash)
            .map(|(k, _)| label_is_not_git_hash_rule(k.clone()))
            .collect(),
    )
}

/// Haskell `labelIsNotGitHashRule`.
fn label_is_not_git_hash_rule(label: LabelName) -> Box<dyn Rule> {
    let code = "DL3055";
    let severity = DLSeverity::DLWarningC;
    let message = format!("Label `{}` is not a valid git hash.", label);
    let check = move |instr: &Instruction<_>| match instr {
        Instruction::Label(ls) => get_bad_hash_labels(&label, ls).is_empty(),
        _ => true,
    };
    simple_rule(code, severity, &message, check)
}

/// Haskell `getBadHashLabels`.
fn get_bad_hash_labels(lbl: &str, prs: &Pairs) -> Pairs {
    prs.iter()
        .filter(|(l, v)| l == lbl && is_bad_hash(v))
        .cloned()
        .collect()
}

/// Haskell `isBadHash`.
fn is_bad_hash(h: &str) -> bool {
    h.chars().any(|c| !valid_hash(c)) || (h.chars().count() != 40 && h.chars().count() != 7)
}

/// Haskell `validHash`: ['0'..'9'] ++ ['a'..'f'].
fn valid_hash(c: char) -> bool {
    c.is_ascii_digit() || ('a'..='f').contains(&c)
}

#[cfg(test)]
mod tests {
    use crate::config::configuration::Configuration;
    use crate::rule::LabelType;
    use crate::testing::*;

    fn cfg() -> Configuration {
        let mut c = Configuration::default();
        c.label_schema
            .insert("githash".to_string(), LabelType::GitHash);
        c
    }

    #[test]
    fn not_ok_with_label_not_containing_git_hash() {
        let c = cfg();
        rule_catches_with(&c, "DL3055", "LABEL githash=\"not-git-hash\"");
        on_build_rule_catches_with(&c, "DL3055", "LABEL githash=\"not-git-hash\"");
    }

    #[test]
    fn ok_with_label_containing_short_git_hash() {
        let c = cfg();
        rule_catches_not_with(&c, "DL3055", "LABEL githash=\"2dbfae9\"");
        on_build_rule_catches_not_with(&c, "DL3055", "LABEL githash=\"2dbfae9\"");
    }

    #[test]
    fn ok_with_label_containing_long_git_hash() {
        let c = cfg();
        rule_catches_not_with(
            &c,
            "DL3055",
            "LABEL githash=\"43c572f1272b6b3171dd1db9e41b7027128ce080\"",
        );
        on_build_rule_catches_not_with(
            &c,
            "DL3055",
            "LABEL githash=\"43c572f1272b6b3171dd1db9e41b7027128ce080\"",
        );
    }

    #[test]
    fn ok_with_other_label_not_containing_git_hash() {
        let c = cfg();
        rule_catches_not_with(&c, "DL3055", "LABEL other=\"foo\"");
        on_build_rule_catches_not_with(&c, "DL3055", "LABEL other=\"bar\"");
    }
}
