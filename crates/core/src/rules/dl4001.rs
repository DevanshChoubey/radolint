// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL4001.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, State, add_fail, combine, custom_rule, empty_state,
    fold_arguments, modify, onbuild, pipe, replace_with,
};
use crate::shell::{ParsedShell, find_command_names};
use dlint_syntax::{Instruction, Linenumber, RunArgs};
use std::collections::BTreeSet;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl4001(), onbuild(dl4001()))
}

/// Haskell `dl4001`.
fn dl4001() -> Box<dyn Rule> {
    let code = "DL4001";
    let severity = DLSeverity::DLWarningC;
    let message = "Either use Wget or Curl but not both";

    let check = move |line: Linenumber,
                      st: State<BTreeSet<i64>>,
                      instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => {
            let new_args = fold_arguments(extract_commands, args);
            let new_state = pipe(st, |s| modify(|a| a.union(&new_args).cloned().collect(), s));
            if !new_args.is_empty() && new_state.state.len() >= 2 {
                pipe(new_state, |s| {
                    add_fail(
                        CheckFailure {
                            code: RuleCode::from(code),
                            severity,
                            message: message.to_string(),
                            line,
                        },
                        s,
                    )
                })
            } else {
                new_state
            }
        }
        // Reset the state for each stage
        Instruction::From(_) => pipe(st, |s| replace_with(BTreeSet::new(), s)),
        _ => st,
    };
    custom_rule(check, empty_state(BTreeSet::new()))
}

/// Haskell `extractCommands`.
fn extract_commands(args: &ParsedShell) -> BTreeSet<i64> {
    find_command_names(args)
        .into_iter()
        .filter(|w| w == "curl" || w == "wget")
        .map(|w| if w == "curl" { 0 } else { 1 })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::rule::RuleCode;
    use crate::testing::*;

    fn df(lines: &[&str]) -> String {
        lines.iter().map(|l| format!("{l}\n")).collect()
    }

    #[test]
    fn warns_when_using_both_wget_and_curl() {
        rule_catches(
            "DL4001",
            &df(&["FROM node as foo", "RUN wget my.xyz", "RUN curl localhost"]),
        );
    }

    #[test]
    fn warns_when_using_both_wget_and_curl_in_same_instruction() {
        rule_catches(
            "DL4001",
            &df(&["FROM node as foo", "RUN wget my.xyz && curl localhost"]),
        );
    }

    #[test]
    fn does_not_warn_when_using_only_wget() {
        rule_catches_not("DL4001", &df(&["FROM node as foo", "RUN wget my.xyz"]));
    }

    #[test]
    fn does_not_warn_when_using_both_in_different_stages() {
        rule_catches_not(
            "DL4001",
            &df(&[
                "FROM node as foo",
                "RUN wget my.xyz",
                "FROM scratch",
                "RUN curl localhost",
            ]),
        );
    }

    #[test]
    fn does_not_warns_when_using_both_on_a_single_stage() {
        rule_catches(
            "DL4001",
            &df(&[
                "FROM node as foo",
                "RUN wget my.xyz",
                "RUN curl localhost",
                "FROM scratch",
                "RUN curl localhost",
            ]),
        );
    }

    #[test]
    fn only_warns_on_the_relevant_run_instruction() {
        assert_checks(
            &df(&[
                "FROM node as foo",
                "RUN wget my.xyz",
                "RUN curl my.xyz",
                "RUN echo hello",
            ]),
            |f| fails_with(1, &RuleCode::from("DL4001"), &f),
        );
    }

    #[test]
    fn only_warns_on_many_relevant_run_instructions() {
        assert_checks(
            &df(&[
                "FROM node as foo",
                "RUN wget my.xyz",
                "RUN curl my.xyz",
                "RUN echo hello",
                "RUN wget foo.com",
            ]),
            |f| fails_with(2, &RuleCode::from("DL4001"), &f),
        );
    }
}
