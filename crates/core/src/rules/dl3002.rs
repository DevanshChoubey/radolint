// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3002.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Failures, Rule, RuleCode, State, empty_state, modify, pipe,
    very_custom_rule,
};
use dlint_syntax::{Instruction, Linenumber};
use std::collections::BTreeMap;

type StageLine = Linenumber;

type UserLine = Linenumber;

#[derive(Debug, Clone)]
enum Acc {
    /// `Acc StageLine (Map.IntMap UserLine)`
    Acc(StageLine, BTreeMap<i64, UserLine>),
    Empty,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3002";
    let severity = DLSeverity::DLWarningC;
    let message = "Last USER should not be root";

    let is_root = |user: &str| -> bool {
        user.starts_with("root:") || user.starts_with("0:") || user == "root" || user == "0"
    };

    let check = move |line: Linenumber, st: State<Acc>, instr: &Instruction<_>| match instr {
        Instruction::From(_) => pipe(st, |s| modify(|a| remember_stage(line, a), s)),
        Instruction::User(user) => {
            if !is_root(user) {
                pipe(st, |s| modify(forget_stage, s))
            } else {
                pipe(st, |s| modify(|a| remember_line(line, a), s))
            }
        }
        _ => st,
    };

    let mark_failures = move |st: State<Acc>| -> Failures {
        let State {
            failures: mut fails,
            state,
        } = st;
        match state {
            Acc::Acc(_, m) => {
                // Map.foldl' (Seq.|>) fails (fmap makeFail st): in ascending key order
                for (_, line) in m {
                    fails.push(CheckFailure {
                        code: RuleCode::from(code),
                        severity,
                        message: message.to_string(),
                        line,
                    });
                }
                fails
            }
            Acc::Empty => fails,
        }
    };

    very_custom_rule(check, empty_state(Acc::Empty), mark_failures)
}

fn remember_stage(from: StageLine, acc: Acc) -> Acc {
    match acc {
        Acc::Acc(_, m) => Acc::Acc(from, m),
        Acc::Empty => Acc::Acc(from, BTreeMap::new()),
    }
}

fn forget_stage(acc: Acc) -> Acc {
    match acc {
        Acc::Acc(from, mut m) => {
            m.remove(&from);
            Acc::Acc(from, m)
        }
        Acc::Empty => Acc::Empty,
    }
}

fn remember_line(line: StageLine, acc: Acc) -> Acc {
    match acc {
        Acc::Acc(from, mut m) => {
            m.insert(from, line);
            Acc::Acc(from, m)
        }
        Acc::Empty => Acc::Empty,
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    fn df(lines: &[&str]) -> String {
        lines.iter().map(|l| format!("{l}\n")).collect()
    }

    #[test]
    fn last_user_should_not_be_root() {
        rule_catches("DL3002", &df(&["FROM scratch", "USER root"]));
    }

    #[test]
    fn no_root() {
        rule_catches_not("DL3002", &df(&["FROM scratch", "USER foo"]));
    }

    #[test]
    fn no_root_uid() {
        rule_catches("DL3002", &df(&["FROM scratch", "USER 0"]));
    }

    #[test]
    fn no_root_root() {
        rule_catches("DL3002", &df(&["FROM scratch", "USER root:root"]));
    }

    #[test]
    fn no_uid_gid() {
        rule_catches("DL3002", &df(&["FROM scratch", "USER 0:0"]));
    }

    #[test]
    fn can_switch_back_to_non_root() {
        rule_catches_not(
            "DL3002",
            &df(&["FROM scratch", "USER root", "RUN something", "USER foo"]),
        );
    }

    #[test]
    fn warns_on_transitive_root_user() {
        rule_catches(
            "DL3002",
            &df(&[
                "FROM debian as base",
                "USER root",
                "RUN something",
                "FROM base",
                "RUN something else",
            ]),
        );
    }

    #[test]
    fn warns_on_multiple_stages() {
        rule_catches(
            "DL3002",
            &df(&[
                "FROM debian as base",
                "USER root",
                "RUN something",
                "FROM scratch",
                "USER foo",
                "RUN something else",
            ]),
        );
    }

    #[test]
    fn does_not_warn_when_switching_in_multiple_stages() {
        rule_catches_not(
            "DL3002",
            &df(&[
                "FROM debian as base",
                "USER root",
                "RUN something",
                "USER foo",
                "FROM scratch",
                "RUN something else",
            ]),
        );
    }
}
