// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3051.hs.

use crate::rule::LabelSchema;
use crate::rule::{DLSeverity, Rule, mconcat, simple_rule};
use dlint_syntax::{Instruction, Pairs};

/// Haskell `rule`.
pub fn rule(label_schema: &LabelSchema) -> Box<dyn Rule> {
    mconcat(
        label_schema
            .keys()
            .map(|k| label_is_not_empty_rule(k))
            .collect(),
    )
}

/// Haskell `labelIsNotEmptyRule`.
fn label_is_not_empty_rule(label: &str) -> Box<dyn Rule> {
    let code = "DL3051";
    let severity = DLSeverity::DLWarningC;
    let message = format!("label `{}` is empty.", label);
    let label = label.to_string();
    simple_rule(code, severity, &message, move |instr| match instr {
        Instruction::Label(pairs) => get_empty_labels(&label, pairs).is_empty(),
        _ => true,
    })
}

/// Haskell `getEmptyLabels`.
fn get_empty_labels(lbl: &str, prs: &Pairs) -> Pairs {
    prs.iter()
        .filter(|(l, v)| l == lbl && v.is_empty())
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::config::configuration::Configuration;
    use crate::rule::LabelType;
    use crate::testing::*;

    fn cfg() -> Configuration {
        let mut c = Configuration::default();
        c.label_schema
            .insert("emptylabel".to_string(), LabelType::RawText);
        c
    }

    #[test]
    fn not_ok_with_label_empty() {
        let c = cfg();
        rule_catches_with(&c, "DL3051", "LABEL emptylabel=\"\"");
        on_build_rule_catches_with(&c, "DL3051", "LABEL emptylabel=\"\"");
    }

    #[test]
    fn ok_with_label_not_empty() {
        let c = cfg();
        rule_catches_not_with(&c, "DL3051", "LABEL emptylabel=\"foo\"");
        on_build_rule_catches_not_with(&c, "DL3051", "LABEL emptylabel=\"bar\"");
    }

    #[test]
    fn ok_with_other_label_empty() {
        let c = cfg();
        rule_catches_not_with(&c, "DL3051", "LABEL other=\"\"");
        on_build_rule_catches_not_with(&c, "DL3051", "LABEL other=\"\"");
    }
}
