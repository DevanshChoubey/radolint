// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3044.hs.

use crate::rule::{
    add_fail, custom_rule, empty_state, modify, CheckFailure, DLSeverity, Rule, RuleCode, State,
};
use dlint_syntax::{Instruction, Pairs};
use std::collections::BTreeSet;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3044";
    let severity = DLSeverity::DLErrorC;
    let message =
        "Do not refer to an environment variable within the same `ENV` statement where it is defined.";

    let check = move |line, st: State<BTreeSet<String>>, instr: &Instruction<_>| match instr {
        Instruction::Env(pairs) => {
            let defined: BTreeSet<String> = pairs.iter().map(|p| p.0.clone()).collect();
            let bad = list_of_references(pairs)
                .iter()
                .any(|env| !st.state.contains(env));
            let new_state = modify(|s: BTreeSet<String>| s.union(&defined).cloned().collect(), st);
            if !bad {
                new_state
            } else {
                add_fail(
                    CheckFailure {
                        code: RuleCode::from(code),
                        severity,
                        message: message.to_string(),
                        line,
                    },
                    new_state,
                )
            }
        }
        Instruction::Arg(arg, _) => modify(
            |mut s: BTreeSet<String>| {
                s.insert(arg.clone());
                s
            },
            st,
        ),
        _ => st,
    };
    custom_rule(check, empty_state(BTreeSet::new()))
}

/// generates a list of references to variable names referenced on the right
/// hand side of a variable definition, except when the variable is
/// referenced on its own right hand side.
fn list_of_references(prs: &Pairs) -> Vec<String> {
    let mut out = vec![];
    for (idx, (var, _)) in prs.iter().enumerate() {
        let others: Vec<&str> = prs
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, p)| p.1.as_str())
            .collect();
        if is_substring_of_any(var, &others) {
            out.push(var.clone());
        }
    }
    out
}

/// is a reference of a variable substring of any text?
/// matches ${var_name} and $var_name, but not $var_nameblafoo
fn is_substring_of_any(t: &str, l: &[&str]) -> bool {
    l.iter()
        .any(|v| v.contains(&format!("${{{t}}}")) || bare_variable_in_text(t, v))
}

/// we find a 'bare' variable with name v in a text, if '$v' is in the text at
/// any place and any text following after that occurence would terminate a
/// variable name.
fn bare_variable_in_text(v: &str, t: &str) -> bool {
    let var = format!("${v}");
    let rest = t.split(var.as_str()).skip(1);
    // x would terminate a variable name if it was appended directly to that name
    let terminates_var_name = |x: &str| match x.chars().next() {
        None => true,
        Some(c) => !(c.is_ascii_alphanumeric() || c == '_'),
    };
    t.contains(&var) && rest.into_iter().any(terminates_var_name)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn ok_with_normal_env() {
        rule_catches_not("DL3044", "ENV BLA=\"blubb\"\nENV BLUBB=\"${BLA}/blubb\"");
    }
    #[test]
    fn ok_with_partial_match_1() {
        rule_catches_not("DL3044", "ENV BLA=\"blubb\" BLUBB=\"${FOOBLA}/blubb\"");
    }
    #[test]
    fn ok_with_partial_match_2() {
        rule_catches_not("DL3044", "ENV BLA=\"blubb\" BLUBB=\"${BLAFOO}/blubb\"");
    }
    #[test]
    fn ok_with_partial_match_3() {
        rule_catches_not("DL3044", "ENV BLA=\"blubb\" BLUBB=\"$FOOBLA/blubb\"");
    }
    #[test]
    fn ok_with_partial_match_4() {
        rule_catches_not("DL3044", "ENV BLA=\"blubb\" BLUBB=\"$BLAFOO/blubb\"");
    }
    #[test]
    fn fail_with_partial_match_5() {
        rule_catches("DL3044", "ENV BLA=\"blubb\" BLUBB=\"$BLA/$BLAFOO/blubb\"");
    }
    #[test]
    fn ok_with_parial_match_6() {
        rule_catches_not("DL3044", "ENV BLA=\"blubb\" BLUBB=\"BLA/$BLAFOO/BLA\"");
    }
    #[test]
    fn ok_when_previously_defined_in_arg() {
        rule_catches_not("DL3044", "ARG BLA\nENV BLA=${BLA}");
    }
    #[test]
    fn ok_when_previously_defined_in_env() {
        rule_catches_not("DL3044", "ENV BLA blubb\nENV BLA=${BLA}");
    }
    #[test]
    fn ok_with_referencing_a_variable_on_its_own_right_hand_side() {
        rule_catches_not("DL3044", "ENV PATH=/bla:${PATH}");
    }
    #[test]
    fn ok_with_referencing_own_right_side_twice_in_different_envs() {
        rule_catches_not("DL3044", "ENV PATH=/bla:${PATH}\nENV PATH=/blubb:${PATH}");
    }
    #[test]
    fn fail_when_referencing_own_right_side_twice_within_same_env() {
        rule_catches("DL3044", "ENV PATH=/bla:${PATH} PATH=/blubb:${PATH}");
    }
    #[test]
    fn fail_with_selfreferencing_with_curly_braces_env() {
        rule_catches("DL3044", "ENV BLA=\"blubb\" BLUBB=\"${BLA}/blubb\"");
    }
    #[test]
    fn fail_with_selfreferencing_without_curly_braces_env() {
        rule_catches("DL3044", "ENV BLA=\"blubb\" BLUBB=\"$BLA/blubb\"");
    }
    #[test]
    fn fail_with_full_match_1() {
        rule_catches("DL3044", "ENV BLA=\"blubb\" BLUBB=\"$BLA\"");
    }
    #[test]
    fn fail_with_full_match_2() {
        rule_catches("DL3044", "ENV BLA=\"blubb\" BLUBB=\"${BLA}\"");
    }
}
