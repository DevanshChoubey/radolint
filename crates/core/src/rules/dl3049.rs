// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3049.hs.

use crate::rule::*;
use crate::shell::ParsedShell;
use dlint_syntax::*;
use std::collections::BTreeSet;

/// Haskell `rule`.
pub fn rule(label_schema: &LabelSchema) -> Box<dyn Rule> {
    mconcat(label_schema.keys().map(|k| missing_label_rule(k.clone())).collect())
}

/// Haskell `data StageID`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StageID {
    name: BaseImage,
    line: Linenumber,
}

/// Haskell `data Acc`.
#[derive(Debug, Clone)]
enum Acc {
    Acc(StageID, BTreeSet<StageID>, BTreeSet<StageID>, BTreeSet<StageID>),
    Empty,
}

/// Haskell `missingLabelRule`.
///
/// triggers on a `FROM` instruction when label `label` is not defined within
/// that stage. Tracks defined labels through multi stage builds
fn missing_label_rule(label: LabelName) -> Box<dyn Rule> {
    let code = "DL3049";
    let severity = DLSeverity::DLInfoC;
    let message = format!("Label `{}` is missing.", label);

    let check = move |line: Linenumber, state: State<Acc>, instr: &Instruction<ParsedShell>| -> State<Acc> {
        match instr {
            Instruction::From(img) => {
                let img = img.clone();
                modify(move |a| current_stage(StageID { name: img, line }, a), state)
            }
            Instruction::Copy(_, flags) if matches!(flags.source_flag, CopySource::CopySource(_)) => {
                let src = match &flags.source_flag {
                    CopySource::CopySource(s) => s.clone(),
                    _ => unreachable!(),
                };
                modify(move |a| mark_silent_by_alias(&src, a), state)
            }
            Instruction::Label(pairs) => {
                if pairs.iter().any(|(k, _)| *k == label) {
                    let name = get_current_stage_name(&state);
                    let state = modify(move |a| mark_silent_by_alias(&name, a), state);
                    modify(mark_good, state)
                } else {
                    state
                }
            }
            _ => state,
        }
    };

    let mark_failure = move |st: State<Acc>| -> Failures {
        match st.state {
            Acc::Acc(_, _, _, b) => {
                let mut fails = st.failures;
                // Haskell `Set.map markFail b`: CheckFailure's Ord compares only the line,
                // so the set dedupes by line and is ordered by line.
                let lines: BTreeSet<Linenumber> = b.iter().map(|s| s.line).collect();
                for line in lines {
                    fails.push(CheckFailure {
                        code: RuleCode::from(code),
                        severity,
                        message: message.clone(),
                        line,
                    });
                }
                fails
            }
            Acc::Empty => st.failures,
        }
    };

    very_custom_rule(check, empty_state(Acc::Empty), mark_failure)
}

/// Haskell `currentStage`.
fn current_stage(stageid: StageID, acc: Acc) -> Acc {
    match acc {
        Acc::Empty => {
            let mut b = BTreeSet::new();
            b.insert(stageid.clone());
            Acc::Acc(stageid, BTreeSet::new(), BTreeSet::new(), b)
        }
        Acc::Acc(_, mut g, s, mut b) => {
            let predicate = |sid: &StageID, other: &StageID| -> bool {
                match &other.name.alias {
                    None => false,
                    Some(als) => als.un_image_alias == sid.name.image.image_name,
                }
            };
            if g.iter().any(|o| predicate(&stageid, o)) {
                g.insert(stageid.clone());
                Acc::Acc(stageid, g, s, b)
            } else {
                b.insert(stageid.clone());
                Acc::Acc(stageid, g, s, b)
            }
        }
    }
}

/// Haskell `markGood`.
fn mark_good(acc: Acc) -> Acc {
    match acc {
        Acc::Empty => Acc::Empty,
        Acc::Acc(stageid, mut good, mut silent, mut bad) => {
            good.insert(stageid.clone());
            silent.remove(&stageid);
            bad.remove(&stageid);
            Acc::Acc(stageid, good, silent, bad)
        }
    }
}

/// Haskell `markSilentByAlias`.
fn mark_silent_by_alias(silentname: &str, acc: Acc) -> Acc {
    match acc {
        Acc::Empty => Acc::Empty,
        Acc::Acc(stageid, good, mut silent, bad) => {
            let stages: BTreeSet<StageID> = bad
                .iter()
                .filter(|s| match &s.name.alias {
                    None => false,
                    Some(als) => als.un_image_alias == silentname,
                })
                .cloned()
                .collect();
            silent.extend(stages.iter().cloned());
            let bad: BTreeSet<StageID> = bad.difference(&stages).cloned().collect();
            Acc::Acc(stageid, good, silent, bad)
        }
    }
}

/// Haskell `getCurrentStageName`.
fn get_current_stage_name(st: &State<Acc>) -> String {
    match &st.state {
        Acc::Acc(StageID { name, .. }, _, _, _) => match &name.alias {
            None => name.image.image_name.clone(),
            Some(als) => als.un_image_alias.clone(),
        },
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::config::configuration::Configuration;
    use crate::rule::LabelType;
    use crate::testing::*;

    fn cfg() -> Configuration {
        let mut c = Configuration::default();
        c.label_schema.insert("foo".to_string(), LabelType::RawText);
        c
    }

    fn fails_n(n: usize, lines: &[&str]) {
        let c = cfg();
        let df = lines.join("\n") + "\n";
        assert_checks_with(&c, &df, |f| fails_with(n, &"DL3049".into(), &f));
    }

    #[test]
    fn not_ok_single_stage_no_label() {
        let c = cfg();
        rule_catches_with(&c, "DL3049", "FROM baseimage");
        on_build_rule_catches_with(&c, "DL3049", "FROM baseimage");
    }
    #[test]
    fn not_ok_single_stage_wrong_label() {
        let c = cfg();
        rule_catches_with(&c, "DL3049", "FROM baseimage\nLABEL bar=\"baz\"");
        on_build_rule_catches_with(&c, "DL3049", "FROM baseimage\nLABEL bar=\"baz\"");
    }
    #[test]
    fn ok_single_stage_label_present() {
        let c = cfg();
        rule_catches_not_with(&c, "DL3049", "FROM baseimage\nLABEL foo=\"bar\"");
        on_build_rule_catches_not_with(&c, "DL3049", "FROM baseimage\nLABEL foo=\"bar\"");
    }
    #[test]
    fn warn_twice_two_stages_no_labels() {
        fails_n(2, &["FROM stage1", "FROM stage2"]);
    }
    #[test]
    fn warn_twice_two_stages_wrong_labels_only() {
        fails_n(2, &["FROM stage1", "LABEL bar=\"baz\"", "FROM stage2", "LABEL buzz=\"fuzz\""]);
    }
    #[test]
    fn warn_once_two_stages_label_present_in_first_only() {
        fails_n(1, &["FROM baseimage", "LABEL foo=\"bar\"", "FROM newimage"]);
    }
    #[test]
    fn warn_once_two_stages_label_present_in_second_only() {
        fails_n(1, &["FROM baseimage", "FROM newimage", "LABEL foo=\"bar\""]);
    }
    #[test]
    fn warn_once_two_stages_no_inheritance_wrong_label_in_one() {
        fails_n(1, &["FROM baseimage", "LABEL baz=\"bar\"", "FROM newimage", "LABEL foo=\"bar\""]);
    }
    #[test]
    fn warn_twice_two_stages_inheritance_no_labels() {
        fails_n(2, &["FROM baseimage as base", "FROM base"]);
    }
    #[test]
    fn dont_warn_two_stages_inheritance_label_defined_in_second_stage() {
        fails_n(0, &["FROM baseimage as base", "FROM base", "LABEL foo=\"bar\""]);
    }
    #[test]
    fn dont_warn_two_stages_no_inheritance_label_in_last_stage_with_copy() {
        fails_n(
            0,
            &["FROM baseimage as base", "FROM anotherimage", "LABEL foo=\"bar\"", "COPY --from=base /bar /baz"],
        );
    }
    #[test]
    fn warn_once_two_stages_no_inheritance_label_in_last_stage_no_connection() {
        fails_n(1, &["FROM baseimage as base", "FROM anotherimage", "LABEL foo=\"bar\""]);
    }
    #[test]
    fn dont_warn_two_stages_inheritance() {
        fails_n(0, &["FROM baseimage as base", "LABEL foo=\"bar\"", "FROM base"]);
    }
    #[test]
    fn warn_once_multiple_results_no_inheritance_label_in_one_result() {
        fails_n(
            1,
            &[
                "FROM baseimage as base",
                "FROM newimage",
                "LABEL foo=\"bar\"",
                "COPY --from=base /bar /baz",
                "",
                "FROM newimage",
                "COPY --from=base /bar /baz",
            ],
        );
    }
    #[test]
    fn warn_once_multiple_results_inheritance_label_in_one_result() {
        fails_n(1, &["FROM baseimage as base", "FROM base", "LABEL foo=\"bar\"", "", "FROM base"]);
    }
    #[test]
    fn warn_once_multiple_results_inheritance_label_in_one_result_2() {
        fails_n(1, &["FROM baseimage as base", "FROM base", "", "FROM base", "LABEL foo=\"bar\""]);
    }
}
