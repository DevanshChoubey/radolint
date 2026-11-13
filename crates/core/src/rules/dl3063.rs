// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3063.hs.

use crate::rule::{DLSeverity, Rule, simple_rule};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3063";
    let severity = DLSeverity::DLWarningC;
    let message = "stage name should not be a reserved word";

    let check = |instr: &Instruction<_>| match instr {
        Instruction::From(base) => match &base.alias {
            Some(ImageAlias {
                un_image_alias: als,
            }) => !(als == "scratch" || als == "context"),
            _ => true,
        },
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::{rule_catches, rule_catches_not};

    #[test]
    fn not_ok_stage_name_scratch() {
        rule_catches("DL3063", "FROM foo:bar AS scratch");
    }

    #[test]
    fn not_ok_stage_name_context() {
        rule_catches("DL3063", "FROM foo:bar AS context");
    }

    #[test]
    fn ok_stage_name_foobar() {
        rule_catches_not("DL3063", "FROM foo:bar AS foobar");
    }
}
