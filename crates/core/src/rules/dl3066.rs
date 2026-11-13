// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3066.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, add_fail, custom_rule, empty_state, modify,
};
use radolint_syntax::*;
use std::collections::BTreeSet;

/// Haskell `data Acc`.
#[derive(Debug, Clone)]
enum Acc {
    Acc { args: BTreeSet<String> },
    Empty,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3066";
    let severity = DLSeverity::DLInfoC;
    let message = "Non-numeric user-id may not be resolvable by host system";

    custom_rule(
        move |line, st, instr| match instr {
            Instruction::User(u) => {
                if get_uid(u).chars().all(|c| c.is_ascii_digit()) {
                    st
                } else if uid_is_defined_arg(&st.state, u) {
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
            Instruction::Arg(arg, _) => modify(|a| register_arg(arg, a), st),
            _ => st,
        },
        empty_state(Acc::Empty),
    )
}

/// Haskell `getUid`.
fn get_uid(t: &str) -> &str {
    if t.contains(':') {
        // u (Text.splitOn ":" t): first piece (splitOn always yields at least one)
        t.split(':').next().unwrap_or("")
    } else {
        t
    }
}

/// Haskell `uidIsDefinedArg`.
fn uid_is_defined_arg(acc: &Acc, u: &str) -> bool {
    match acc {
        Acc::Empty => false,
        Acc::Acc { args } => args.iter().any(|var| var_in_uid(var, u)),
    }
}

/// Haskell `varInUid`.
fn var_in_uid(var: &str, uid: &str) -> bool {
    uid.contains(&format!("${{{var}}}")) || uid.contains(&format!("${var}"))
}

/// Haskell `registerArg`.
fn register_arg(arg: &str, acc: Acc) -> Acc {
    match acc {
        Acc::Empty => Acc::Acc {
            args: BTreeSet::from([arg.to_string()]),
        },
        Acc::Acc { mut args } => {
            args.insert(arg.to_string());
            Acc::Acc { args }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn ok_numeric_uid() {
        rule_catches_not("DL3066", "USER 12345");
    }

    #[test]
    fn ok_numeric_uid_and_gid() {
        rule_catches_not("DL3066", "USER 1234:5678");
    }

    #[test]
    fn not_ok_non_numeric_uid() {
        rule_catches("DL3066", "USER foobar");
    }

    #[test]
    fn not_ok_non_numeric_uid_and_gid() {
        rule_catches("DL3066", "USER foobar:barfoo");
    }

    #[test]
    fn ok_uid_is_non_numeric_but_is_a_defined_arg() {
        let dockerfile = "ARG APP_UID\nFROM foobar:barfoo\nUSER ${APP_UID}\n";
        rule_catches_not("DL3066", dockerfile);
    }
}
