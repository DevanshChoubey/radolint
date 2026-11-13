// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3059.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, State, add_fail, custom_rule, empty_state,
    fold_arguments, modify, pipe,
};
use crate::shell::ParsedShell;
use dlint_syntax::{Instruction, Linenumber, RunArgs, RunFlags};

#[derive(Debug, Clone, PartialEq)]
enum Acc {
    /// `Acc { flags :: RunFlags, count :: Int }`
    Acc {
        flags: RunFlags,
        count: i64,
    },
    Empty,
}

/// Haskell `rule`.
///
/// This Rule catches multiple consecutive `RUN` instructions.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3059";
    let severity = DLSeverity::DLInfoC;
    let message = "Multiple consecutive `RUN` instructions. Consider consolidation.";

    let check =
        move |line: Linenumber, st: State<Acc>, instr: &Instruction<ParsedShell>| match instr {
            Instruction::Run(RunArgs(ar, fl)) => {
                let n = fold_arguments(count_commands, ar);
                if st.state == Acc::Empty {
                    pipe(st, |s| modify(|a| remember(fl.clone(), n, a), s))
                } else if flags_of(&st.state) != Some(fl) {
                    pipe(st, |s| modify(|a| remember(fl.clone(), n, a), s))
                } else if n > 2 || count_of(&st.state) > 2 {
                    pipe(st, |s| modify(|a| remember(fl.clone(), n, a), s))
                } else {
                    pipe(st, |s| {
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
                }
            }
            Instruction::Comment(_) => st,
            _ => pipe(st, |s| modify(reset, s)),
        };
    custom_rule(check, empty_state(Acc::Empty))
}

fn flags_of(a: &Acc) -> Option<&RunFlags> {
    match a {
        Acc::Acc { flags, .. } => Some(flags),
        Acc::Empty => None,
    }
}

fn count_of(a: &Acc) -> i64 {
    match a {
        Acc::Acc { count, .. } => *count,
        Acc::Empty => 0, // unreachable: Empty is handled first
    }
}

fn remember(fl: RunFlags, cn: i64, _: Acc) -> Acc {
    Acc::Acc {
        flags: fl,
        count: cn,
    }
}

fn reset(_: Acc) -> Acc {
    Acc::Empty
}

fn count_commands(script: &ParsedShell) -> i64 {
    script.present_commands.len() as i64
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn ok_with_no_run_at_all() {
        rule_catches_not("DL3059", "FROM debian:10");
    }
    #[test]
    fn ok_with_one_run() {
        rule_catches_not("DL3059", "RUN /foo.sh");
    }
    #[test]
    fn ok_with_two_not_consecutive_run() {
        rule_catches_not("DL3059", "RUN /foo.sh\nWORKDIR /\nRUN /bar.sh");
    }
    #[test]
    fn not_ok_with_two_consecutive_runs() {
        rule_catches("DL3059", "RUN /foo.sh\nRUN /bar.sh");
    }
    #[test]
    fn not_ok_with_two_runs_separated_by_a_comment() {
        rule_catches("DL3059", "RUN /foo.sh\n# a comment\nRUN /bar.sh");
    }
    #[test]
    fn not_ok_with_two_runs_separated_by_two_comment() {
        rule_catches(
            "DL3059",
            "RUN /foo.sh\n# a comment\n# another comment\nRUN /bar.sh",
        );
    }
    #[test]
    fn ok_with_one_run_after_a_comment() {
        rule_catches_not("DL3059", "# a comment\nRUN /foo.sh");
    }
    #[test]
    fn ok_with_two_consecutive_runs_when_flags_are_different_1() {
        rule_catches_not(
            "DL3059",
            "RUN --mount=type=secret,id=foo /foo.sh\nRUN /bar.sh",
        );
    }
    #[test]
    fn ok_with_two_consecutive_runs_when_flags_are_different_2() {
        let dfile = [
            "RUN --mount=type=secret,id=foo /foo.sh",
            "RUN --mount=type=secret,id=bar /bar.sh",
        ];
        rule_catches_not("DL3059", &(dfile.join("\n") + "\n"));
    }
    #[test]
    fn not_ok_with_two_consecutive_runs_when_flags_are_equal() {
        let dfile = [
            "RUN --mount=type=secret,id=foo /foo.sh",
            "RUN --mount=type=secret,id=foo /bar.sh",
        ];
        rule_catches("DL3059", &(dfile.join("\n") + "\n"));
    }
    #[test]
    fn ok_when_commands_are_chained_1() {
        rule_catches_not("DL3059", "RUN foo && bar\nRUN foobar\n");
    }
    #[test]
    fn ok_when_commands_are_chained_2() {
        rule_catches_not("DL3059", "RUN foo; bar\nRUN foobar\n");
    }
    #[test]
    fn ok_when_commands_are_chained_3() {
        rule_catches_not("DL3059", "RUN foobar\nRUN foo && bar\n");
    }
    #[test]
    fn ok_when_commands_are_chained_4() {
        rule_catches_not("DL3059", "RUN foobar\nRUN foo; bar\n");
    }
    #[test]
    fn ok_when_commands_are_chained_5() {
        rule_catches_not("DL3059", "RUN foo && bar\nRUN foo; bar\n");
    }
    #[test]
    fn not_ok_when_more_than_one_run_has_just_one_command_in_a_row() {
        rule_catches("DL3059", "RUN foo && bar\nRUN foo\nRUN bar\n");
    }
}
