// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3024.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, add_fail, custom_rule, empty_state, modify,
};
use dlint_syntax::{ImageAlias, Instruction};
use std::collections::BTreeSet;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3024";
    let severity = DLSeverity::DLErrorC;
    let message = "FROM aliases (stage names) must be unique";
    custom_rule(
        move |line, st, instr| match instr {
            Instruction::From(base) => match &base.alias {
                Some(ImageAlias {
                    un_image_alias: als,
                }) => {
                    if st.state.contains(als) {
                        add_fail(
                            CheckFailure {
                                code: RuleCode::from(code),
                                severity,
                                message: message.to_string(),
                                line,
                            },
                            st,
                        )
                    } else {
                        let als = als.clone();
                        modify(
                            move |mut s: BTreeSet<String>| {
                                s.insert(als);
                                s
                            },
                            st,
                        )
                    }
                }
                None => st,
            },
            _ => st,
        },
        empty_state(BTreeSet::<String>::new()),
    )
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn warn_on_duplicate_aliases() {
        rule_catches(
            "DL3024",
            "FROM node as foo\nRUN something\nFROM scratch as foo\nRUN something\n",
        );
    }
    #[test]
    fn dont_warn_on_unique_aliases() {
        rule_catches_not(
            "DL3024",
            "FROM scratch as build\nRUN foo\nFROM node as run\nRUN baz\n",
        );
    }
}
