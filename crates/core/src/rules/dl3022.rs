// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3022.hs.

use crate::rule::{
    add_fail, custom_rule, drop_quotes, empty_state, modify, pipe, CheckFailure, DLSeverity, Rule,
    RuleCode, State,
};
use dlint_syntax::{BaseImage, CopySource, Instruction, Linenumber};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Acc {
    /// `Acc {count :: Int, names :: Set Text}`
    Acc { count: i64, names: BTreeSet<String> },
    Empty,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3022";
    let severity = DLSeverity::DLWarningC;
    let message = "`COPY --from` should reference a previously defined `FROM` alias";

    let check = move |line: Linenumber, st: State<Acc>, instr: &Instruction<_>| match instr {
        Instruction::From(BaseImage { alias: Some(als), .. }) => {
            let als = als.un_image_alias.clone();
            pipe(st, |s| modify(|a| inc_and_add_name(als, a), s))
        }
        Instruction::From(BaseImage { .. }) => pipe(st, |s| modify(inc_count, s)),
        Instruction::Copy(_, flags) => match &flags.source_flag {
            CopySource::CopySource(s) => {
                if drop_quotes(s).contains(':') {
                    st
                } else if is_member(s, &st.state) {
                    st
                } else {
                    match read_decimal(s) {
                        Some(v) if v < name_count(&st.state) => st,
                        _ => add_fail(
                            CheckFailure {
                                code: RuleCode::from(code),
                                severity,
                                message: message.to_string(),
                                line,
                            },
                            st,
                        ),
                    }
                }
            }
            CopySource::NoSource => st,
        },
        _ => st,
    };
    custom_rule(check, empty_state(Acc::Empty))
}

/// `Data.Text.Read.decimal` at `Int`: leading digits, wrapping on overflow.
fn read_decimal(s: &str) -> Option<i64> {
    let digits: Vec<u8> = s.bytes().take_while(|b| b.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let mut v: i64 = 0;
    for d in digits {
        v = v.wrapping_mul(10).wrapping_add((d - b'0') as i64);
    }
    Some(v)
}

fn inc_and_add_name(s: String, acc: Acc) -> Acc {
    match acc {
        Acc::Empty => Acc::Acc { count: 1, names: BTreeSet::from([s]) },
        Acc::Acc { count, mut names } => {
            names.insert(s);
            Acc::Acc { count: count + 1, names }
        }
    }
}

fn inc_count(acc: Acc) -> Acc {
    match acc {
        Acc::Empty => Acc::Acc { count: 1, names: BTreeSet::new() },
        Acc::Acc { count, names } => Acc::Acc { count: count + 1, names },
    }
}

fn is_member(s: &str, acc: &Acc) -> bool {
    match acc {
        Acc::Empty => false,
        Acc::Acc { names, .. } => names.contains(s),
    }
}

fn name_count(acc: &Acc) -> i64 {
    match acc {
        Acc::Empty => 0,
        Acc::Acc { count, .. } => *count,
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn warn_on_missing_alias() {
        rule_catches("DL3022", "COPY --from=foo bar .");
    }

    #[test]
    fn warn_on_alias_defined_after() {
        let docker_file = ["FROM scratch", "COPY --from=build foo .", "FROM node as build", "RUN baz"];
        rule_catches("DL3022", &(docker_file.join("\n") + "\n"));
    }

    #[test]
    fn dont_warn_on_correctly_defined_aliases() {
        let docker_file =
            ["FROM scratch as build", "RUN foo", "FROM node", "COPY --from=build foo .", "RUN baz"];
        rule_catches_not("DL3022", &(docker_file.join("\n") + "\n"));
    }

    #[test]
    fn dont_warn_on_external_images() {
        rule_catches_not("DL3022", "COPY --from=haskell:latest bar .");
    }

    #[test]
    fn warn_on_invalid_stage_count_as_from_count() {
        rule_catches("DL3022", "COPY --from=0 bar .");
    }

    #[test]
    fn dont_warn_on_valid_stage_count_as_from_count() {
        let docker_file = ["FROM scratch as build", "RUN foo", "FROM node", "COPY --from=0 foo ."];
        rule_catches_not("DL3022", &(docker_file.join("\n") + "\n"));
    }

    #[test]
    fn dont_warn_on_valid_stage_count_as_from_count_unaliased() {
        let docker_file = ["FROM scratch", "RUN foo", "FROM node", "COPY --from=0 foo ."];
        rule_catches_not("DL3022", &(docker_file.join("\n") + "\n"));
    }
}
