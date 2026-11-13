// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3045.hs.

use crate::rule::{
    add_fail, custom_rule, empty_state, modify, CheckFailure, DLSeverity, Rule, RuleCode, State,
};
use dlint_syntax::{BaseImage, CopyArgs, Instruction, Linenumber, TargetPath};
use std::collections::BTreeMap;

/// This data encapsulates the name of a build stage. It may be None within an
/// `ONBUILD` context. (Derived `Ord`: `Stage` < `None`.)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    Stage { stage: String },
    None,
}

/// The state here keeps the image name/alias of the current build stage
/// and a map from image names/aliases to a Bool, saving whether or
/// not a `WORKDIR` has been set in a build stage.
#[derive(Debug, Clone)]
enum Acc {
    Acc { current: Stage, workdir_set: BTreeMap<Stage, bool> },
    Empty,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3045";
    let severity = DLSeverity::DLWarningC;
    let message = "`COPY` to a relative destination without `WORKDIR` set.";

    let check = move |line: Linenumber, st: State<Acc>, instr: &Instruction<_>| match instr {
        Instruction::From(from) => modify(|a| remember_stage(from, a), st),
        Instruction::Workdir(_) => modify(remember_workdir, st),
        Instruction::Copy(CopyArgs { target_path: TargetPath { un_target_path: dest }, .. }, _) => {
            let d = dest.trim_matches(quote_predicate);
            let workdir_known = match &st.state {
                Acc::Acc { current, workdir_set } => workdir_set.get(current) == Some(&true),
                Acc::Empty => false,
            };
            if workdir_known {
                st // workdir has been set
            } else if d.starts_with('/') {
                st // absolute dest. normal
            } else if is_windows_absolute(d) {
                st // absolute dest. windows
            } else if d.starts_with('$') {
                st // dest is a variable
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
        _ => st,
    };
    custom_rule(check, empty_state(Acc::Empty))
}

fn remember_stage(from: &BaseImage, acc: Acc) -> Acc {
    match (&from.alias, acc) {
        (Some(als), Acc::Empty) => Acc::Acc {
            current: Stage::Stage { stage: als.un_image_alias.clone() },
            workdir_set: BTreeMap::new(),
        },
        (None, Acc::Empty) => Acc::Acc {
            current: Stage::Stage { stage: from.image.image_name.clone() },
            workdir_set: BTreeMap::new(),
        },
        (alias, Acc::Acc { mut workdir_set, .. }) => {
            let parent_value = workdir_set
                .get(&Stage::Stage { stage: from.image.image_name.clone() })
                .copied()
                .unwrap_or(false);
            let name = match alias {
                Some(als) => als.un_image_alias.clone(),
                None => from.image.image_name.clone(),
            };
            workdir_set.insert(Stage::Stage { stage: name.clone() }, parent_value);
            Acc::Acc { current: Stage::Stage { stage: name }, workdir_set }
        }
    }
}

fn remember_workdir(acc: Acc) -> Acc {
    match acc {
        Acc::Empty => {
            let mut m = BTreeMap::new();
            m.insert(Stage::None, true);
            Acc::Acc { current: Stage::None, workdir_set: m }
        }
        Acc::Acc { current, mut workdir_set } => {
            workdir_set.insert(current.clone(), true);
            Acc::Acc { current, workdir_set }
        }
    }
}

fn quote_predicate(c: char) -> bool {
    c == '"' || c == '\''
}

fn is_windows_absolute(path: &str) -> bool {
    let mut it = path.chars();
    match (it.next(), it.next()) {
        (Some(d), Some(c)) => d.is_alphabetic() && c == ':',
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    fn both_not(code: &str, df: &str) {
        rule_catches_not(code, df);
        on_build_rule_catches_not(code, df);
    }
    fn both(code: &str, df: &str) {
        rule_catches(code, df);
        on_build_rule_catches(code, df);
    }

    #[test]
    fn ok_copy_with_absolute_destination_and_no_workdir_set() {
        both_not("DL3045", "COPY bla.sh /usr/local/bin/blubb.sh");
    }
    #[test]
    fn ok_copy_absolute_quotes() {
        rule_catches_not("DL3045", "COPY bla.sh \"/usr/local/bin/blubb.s\"");
        on_build_rule_catches_not("DL3045", "COPY bla.sh \"/usr/local/bin/blubb.sh\"");
    }
    #[test]
    fn ok_copy_absolute_windows() {
        rule_catches_not("DL3045", "COPY bla.sh c:\\system32\\blubb.sh");
        on_build_rule_catches_not("DL3045", "COPY bla.sh d:\\mypath\\blubb.sh");
    }
    #[test]
    fn ok_copy_absolute_windows_quotes() {
        rule_catches_not("DL3045", "COPY bla.sh \"c:\\system32\\blubb.sh\"");
        on_build_rule_catches_not("DL3045", "COPY bla.sh \"d:\\mypath\\blubb.sh\"");
    }
    #[test]
    fn ok_copy_absolute_windows_alternative_paths() {
        rule_catches_not("DL3045", "COPY bla.sh c:/system32/blubb.sh");
        on_build_rule_catches_not("DL3045", "COPY bla.sh d:/mypath/blubb.sh");
    }
    #[test]
    fn ok_copy_relative_and_workdir_set() {
        rule_catches_not("DL3045", "FROM scratch\nWORKDIR /usr\nCOPY bla.sh blubb.sh");
        // sic: upstream swaps the code and the Dockerfile's first line here
        on_build_rule_catches_not("FROM scratch\nDL3045", "WORKDIR /usr\nCOPY bla.sh blubb.sh");
    }
    #[test]
    fn ok_copy_relative_and_workdir_set_windows() {
        both_not("DL3045", "FROM scratch\nWORKDIR c:\\system32\nCOPY bla.sh blubb.sh");
    }
    #[test]
    fn ok_copy_destination_env_var_1() {
        both_not("DL3045", "COPY src.sh ${SRC_BASE_ENV}");
    }
    #[test]
    fn ok_copy_destination_env_var_2() {
        both_not("DL3045", "COPY src.sh $SRC_BASE_ENV");
    }
    #[test]
    fn ok_copy_destination_env_var_3() {
        both_not("DL3045", "COPY src.sh \"${SRC_BASE_ENV}\"");
    }
    #[test]
    fn ok_copy_destination_env_var_4() {
        both_not("DL3045", "COPY src.sh \"$SRC_BASE_ENV\"");
    }
    #[test]
    fn not_ok_copy_relative_and_no_workdir() {
        both("DL3045", "COPY bla.sh blubb.sh");
    }
    #[test]
    fn not_ok_copy_relative_and_no_workdir_quotes() {
        both("DL3045", "COPY bla.sh \"blubb.sh\"");
    }
    #[test]
    fn not_ok_workdir_in_previous_stage_not_inherited() {
        let df = ["FROM debian:buster as stage1", "WORKDIR /usr", "FROM debian:buster", "COPY foo bar", ""].join("\n");
        both("DL3045", &df);
    }
    #[test]
    fn not_ok_workdir_in_previous_stage_not_inherited_windows() {
        let df = [
            "FROM microsoft/windowsservercore as stage1",
            "WORKDIR c:\\system32",
            "FROM microsoft/windowsservercore",
            "COPY foo bar",
            "",
        ]
        .join("\n");
        both("DL3045", &df);
    }
    #[test]
    fn ok_workdir_set_in_base_image() {
        let df = [
            "FROM debian:buster as base",
            "WORKDIR /usr",
            "FROM debian:buster as stage-inbetween",
            "RUN foo",
            "FROM base",
            "COPY foo bar",
            "",
        ]
        .join("\n");
        both_not("DL3045", &df);
    }
    #[test]
    fn ok_workdir_set_in_base_image_windows() {
        let df = [
            "FROM microsoft/windowsservercore as base",
            "WORKDIR c:\\system32",
            "FROM microsoft/windowsservercore as stage-inbetween",
            "RUN foo",
            "FROM base",
            "COPY foo bar",
            "",
        ]
        .join("\n");
        both_not("DL3045", &df);
    }
    #[test]
    fn ok_workdir_set_in_previous_stage_deep_case() {
        let df = [
            "FROM debian:buster as base1",
            "WORKDIR /usr",
            "FROM base1 as base2",
            "RUN foo",
            "FROM base2",
            "COPY foo bar",
            "",
        ]
        .join("\n");
        both_not("DL3045", &df);
    }
    #[test]
    fn ok_workdir_set_in_previous_stage_deep_case_windows() {
        let df = [
            "FROM microsoft/windowsservercore as base1",
            "WORKDIR c:\\system32",
            "FROM base1 as base2",
            "RUN foo",
            "FROM base2",
            "COPY foo bar",
            "",
        ]
        .join("\n");
        both_not("DL3045", &df);
    }
    #[test]
    fn ok_workdir_set_both_within_onbuild() {
        let df = ["FROM debian:buster", "ONBUILD WORKDIR /usr/local/lib", "ONBUILD COPY foo bar", ""].join("\n");
        both_not("DL3045", &df);
    }
    #[test]
    fn regression_dont_crash_with_single_character_paths() {
        both("DL3045", "COPY a b");
    }
}
