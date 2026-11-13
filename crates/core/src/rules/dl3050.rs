// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3050.hs.

use crate::rule::LabelSchema;
use crate::rule::{DLSeverity, Rule, simple_rule};
use crate::shell::ParsedShell;
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule(label_schema: &LabelSchema, strict_labels: bool) -> Box<dyn Rule> {
    let code = "DL3050";
    let severity = DLSeverity::DLInfoC;
    let message = "Superfluous label(s) present.";
    let label_schema = label_schema.clone();
    let check = move |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Label(pairs) if strict_labels => {
            pairs.iter().all(|(k, _)| label_schema.contains_key(k))
        }
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::config::configuration::Configuration;
    use crate::rule::LabelType;
    use crate::testing::*;

    fn config() -> Configuration {
        let mut c = Configuration::default();
        c.label_schema
            .insert("required".to_string(), LabelType::RawText);
        c.strict_labels = true;
        c
    }

    #[test]
    fn ok_with_no_label() {
        let c = config();
        rule_catches_not_with(&c, "DL3050", "");
        on_build_rule_catches_not_with(&c, "DL3050", "");
    }

    #[test]
    fn ok_with_required_label() {
        let c = config();
        rule_catches_not_with(&c, "DL3050", "LABEL required=\"foo\"");
        on_build_rule_catches_not_with(&c, "DL3050", "LABEL required=\"bar\"");
    }

    #[test]
    fn not_ok_with_just_other_label() {
        let c = config();
        rule_catches_with(&c, "DL3050", "LABEL other=\"bar\"");
        on_build_rule_catches_with(&c, "DL3050", "LABEL other=\"bar\"");
    }

    #[test]
    fn not_ok_with_other_label_and_required_label() {
        let c = config();
        rule_catches_with(&c, "DL3050", "LABEL required=\"foo\" other=\"bar\"");
        on_build_rule_catches_with(&c, "DL3050", "LABEL required=\"foo\" other=\"bar\"");
    }
}
