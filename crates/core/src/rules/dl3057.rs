// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3057.hs.

use crate::rule::*;
use crate::shell::ParsedShell;
use dlint_syntax::*;
use std::collections::BTreeSet;

/// Haskell `data StageID` (derived `Ord`: src, name, line).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StageID {
    src: String,
    name: String,
    line: Linenumber,
}

/// Haskell `data Acc`.
#[derive(Debug, Clone)]
enum Acc {
    Acc(StageID, BTreeSet<StageID>, BTreeSet<StageID>),
    Empty,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3057";
    let severity = DLSeverity::DLIgnoreC;
    let message = "`HEALTHCHECK` instruction missing.";

    let check =
        |line: Linenumber, state: State<Acc>, instr: &Instruction<ParsedShell>| -> State<Acc> {
            match instr {
                Instruction::From(BaseImage {
                    image,
                    alias: Some(als),
                    ..
                }) => {
                    let sid = StageID {
                        src: image.image_name.clone(),
                        name: als.un_image_alias.clone(),
                        line,
                    };
                    let src = image.image_name.clone();
                    modify(move |a| current_stage(&src, sid, a), state)
                }
                Instruction::From(BaseImage {
                    image, alias: None, ..
                }) => {
                    let sid = StageID {
                        src: image.image_name.clone(),
                        name: image.image_name.clone(),
                        line,
                    };
                    let src = image.image_name.clone();
                    modify(move |a| current_stage(&src, sid, a), state)
                }
                Instruction::Healthcheck(_) => modify(good_stage, state),
                _ => state,
            }
        };

    let mark_failures = move |st: State<Acc>| -> Failures {
        match st.state {
            Acc::Acc(_, _, b) => {
                let mut fails = st.failures;
                // Haskell `Set.map makeFail b`: CheckFailure's Ord compares only the line,
                // so the set dedupes by line and is ordered by line.
                let lines: BTreeSet<Linenumber> = b.iter().map(|s| s.line).collect();
                for line in lines {
                    fails.push(CheckFailure {
                        code: RuleCode::from(code),
                        severity,
                        message: message.to_string(),
                        line,
                    });
                }
                fails
            }
            Acc::Empty => st.failures,
        }
    };

    very_custom_rule(check, empty_state(Acc::Empty), mark_failures)
}

/// Haskell `currentStage`.
fn current_stage(src: &str, stageid: StageID, acc: Acc) -> Acc {
    match acc {
        Acc::Acc(_, mut g, mut b) => {
            if g.iter().any(|o| o.name == src) {
                g.insert(stageid.clone());
                Acc::Acc(stageid, g, b)
            } else {
                b.insert(stageid.clone());
                Acc::Acc(stageid, g, b)
            }
        }
        Acc::Empty => {
            let mut b = BTreeSet::new();
            b.insert(stageid.clone());
            Acc::Acc(stageid, BTreeSet::new(), b)
        }
    }
}

/// Haskell `goodStage`.
fn good_stage(acc: Acc) -> Acc {
    match acc {
        Acc::Acc(stageid, g, b) => {
            let now_good = recurse_good(&b, &stageid);
            let mut good: BTreeSet<StageID> = g.union(&now_good).cloned().collect();
            good.insert(stageid.clone());
            let mut bad: BTreeSet<StageID> = b.difference(&now_good).cloned().collect();
            bad.remove(&stageid);
            Acc::Acc(stageid, good, bad)
        }
        Acc::Empty => Acc::Empty,
    }
}

/// Haskell `recurseGood` (local to `goodStage`).
fn recurse_good(bad: &BTreeSet<StageID>, sid: &StageID) -> BTreeSet<StageID> {
    // bad stages to be marked good
    let g1: BTreeSet<StageID> = bad.iter().filter(|e| e.name == sid.src).cloned().collect();
    // bad stages not to be marked good
    let b1: BTreeSet<StageID> = bad.difference(&g1).cloned().collect();
    if g1.is_empty() {
        g1
    } else {
        let mut out = g1.clone();
        for s in g1.iter() {
            out.extend(recurse_good(&b1, s));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    fn unlines(ls: &[&str]) -> String {
        ls.iter().map(|l| format!("{l}\n")).collect()
    }

    #[test]
    fn warn_with_no_healthcheck_instructions() {
        rule_catches("DL3057", &unlines(&["FROM scratch"]));
    }

    #[test]
    fn ok_when_healthcheck_is_explicitly_disabled() {
        rule_catches_not("DL3057", &unlines(&["FROM scratch", "HEALTHCHECK NONE"]));
    }

    #[test]
    fn ok_with_one_healthcheck_instruction() {
        rule_catches_not(
            "DL3057",
            &unlines(&["FROM scratch", "HEALTHCHECK CMD /bin/bla"]),
        );
    }

    #[test]
    fn ok_with_inheriting_healthcheck_instruction() {
        rule_catches_not(
            "DL3057",
            &unlines(&[
                "FROM scratch AS base",
                "HEALTHCHECK CMD /bin/bla",
                "FROM base",
            ]),
        );
    }

    #[test]
    fn ok_with_ending_inheritance_chain_with_healthcheck() {
        rule_catches_not(
            "DL3057",
            &unlines(&[
                "FROM scratch AS base1",
                "FROM base1 AS base2",
                "FROM base2 AS base3",
                "FROM base3 AS end",
                "HEALTHCHECK NONE",
            ]),
        );
    }

    #[test]
    fn warn_when_inheritance_chain_bifurcates() {
        rule_catches(
            "DL3057",
            &unlines(&[
                "FROM scratch AS base1",
                "FROM base1 AS base2",
                "FROM base2 AS base3.1",
                "FROM base2 AS base3.2",
                "",
                "FROM base3.1 AS end1",
                "HEALTHCHECK NONE",
                "",
                "FROM base3.2 AS end2",
            ]),
        );
    }

    #[test]
    fn warn_when_not_inheriting_with_no_healthcheck_instruction() {
        rule_catches(
            "DL3057",
            &unlines(&[
                "FROM scratch AS base",
                "HEALTHCHECK CMD /bin/bla",
                "FROM scratch",
            ]),
        );
    }
}
