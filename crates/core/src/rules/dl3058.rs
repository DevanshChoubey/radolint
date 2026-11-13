// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3058.hs.

use crate::rule::LabelSchema;
use crate::rule::{DLSeverity, LabelName, LabelType, Rule, mconcat, simple_rule};
use radolint_syntax::Instruction;

/// Haskell `rule`.
pub fn rule(label_schema: &LabelSchema) -> Box<dyn Rule> {
    // Map.keys (Map.filter (== Email) labelschema); BTreeMap iterates in key order.
    mconcat(
        label_schema
            .iter()
            .filter(|(_, t)| **t == LabelType::Email)
            .map(|(k, _)| label_is_not_email_rule(k.clone()))
            .collect(),
    )
}

/// Haskell `labelIsNotEmailRule`.
fn label_is_not_email_rule(label: LabelName) -> Box<dyn Rule> {
    let code = "DL3058";
    let severity = DLSeverity::DLWarningC;
    let message = format!(
        "Label `{}` is not a valid email format - must conform to RFC5322.",
        label
    );
    let check = move |instr: &Instruction<_>| match instr {
        Instruction::Label(ls) => get_bad_emailformat_labels(&label, ls).is_empty(),
        _ => true,
    };
    simple_rule(code, severity, &message, check)
}

/// Haskell `getBadEmailformatLabels`.
fn get_bad_emailformat_labels(lbl: &str, pairs: &[(String, String)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .filter(|(l, v)| l == lbl && !is_valid_email(v.as_bytes()))
        .cloned()
        .collect()
}

// DIVERGENCE: no email-validate crate is available; `Email.isValid` is re-implemented as an
// addr-spec check (dot-atom or quoted-string local part, dot-atom or domain-literal domain).
// Comments and folding whitespace (CFWS) are not supported.
fn is_atext(c: u8) -> bool {
    c.is_ascii_alphanumeric() || b"!#$%&'*+/=?^_`{|}~-".contains(&c)
}

fn is_dot_atom(s: &[u8]) -> bool {
    !s.is_empty()
        && s.split(|&c| c == b'.')
            .all(|part| !part.is_empty() && part.iter().all(|&c| is_atext(c)))
}

fn is_quoted_string(s: &[u8]) -> bool {
    if s.len() < 2 || s[0] != b'"' || s[s.len() - 1] != b'"' {
        return false;
    }
    let inner = &s[1..s.len() - 1];
    let mut i = 0;
    while i < inner.len() {
        let c = inner[i];
        if c == b'\\' {
            if i + 1 >= inner.len()
                || !(inner[i + 1] == b' '
                    || inner[i + 1] == b'\t'
                    || (33..=126).contains(&inner[i + 1]))
            {
                return false;
            }
            i += 2;
            continue;
        }
        if !(c == b' '
            || c == b'\t'
            || c == 33
            || (35..=91).contains(&c)
            || (93..=126).contains(&c))
        {
            return false;
        }
        i += 1;
    }
    true
}

fn is_domain_literal(s: &[u8]) -> bool {
    s.len() >= 2
        && s[0] == b'['
        && s[s.len() - 1] == b']'
        && s[1..s.len() - 1]
            .iter()
            .all(|&c| c == b' ' || (33..=90).contains(&c) || (94..=126).contains(&c))
}

/// Haskell `Email.isValid`.
fn is_valid_email(s: &[u8]) -> bool {
    let at = match s.iter().rposition(|&c| c == b'@') {
        Some(i) => i,
        None => return false,
    };
    let (local, domain) = (&s[..at], &s[at + 1..]);
    if local.len() > 64 || domain.len() > 255 {
        return false;
    }
    (is_dot_atom(local) || is_quoted_string(local))
        && (is_dot_atom(domain) || is_domain_literal(domain))
}

#[cfg(test)]
mod tests {
    use crate::config::configuration::Configuration;
    use crate::rule::LabelType;
    use crate::testing::*;

    fn cfg() -> Configuration {
        let mut c = Configuration::default();
        c.label_schema.insert("maintainer".into(), LabelType::Email);
        c
    }

    #[test]
    fn not_ok_with_label_not_containing_valid_email() {
        let c = cfg();
        rule_catches_with(&c, "DL3058", "LABEL maintainer=\"not-email\"");
        on_build_rule_catches_with(&c, "DL3058", "LABEL maintainer=\"not-email\"");
    }

    #[test]
    fn ok_with_label_containing_valid_email() {
        let c = cfg();
        rule_catches_not_with(&c, "DL3058", "LABEL maintainer=\"abcd@google.com\"");
        on_build_rule_catches_not_with(&c, "DL3058", "LABEL maintainer=\"abcd@google.com\"");
    }

    #[test]
    fn ok_with_other_label_not_containing_valid_email() {
        let c = cfg();
        rule_catches_not_with(&c, "DL3058", "LABEL other=\"doo\"");
        on_build_rule_catches_not_with(&c, "DL3058", "LABEL other=\"bar\"");
    }
}
