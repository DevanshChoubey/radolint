// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Process.hs.

use crate::config::configuration::Configuration;
use crate::pragma;
use crate::rule::{CheckFailure, Failures, RuleCode};
use crate::rules;
use crate::shell::{self, ParsedShell};
use radolint_syntax::InstructionPos;
use std::collections::{BTreeMap, BTreeSet};

/// Haskell `AnalisisResult`.
pub struct AnalisisResult {
    /// The set of ignored rules per line.
    pub ignored: BTreeMap<i64, BTreeSet<RuleCode>>,
    /// The set of ignored rules per build stage.
    pub stage_ignored: BTreeMap<i64, BTreeSet<RuleCode>>,
    /// The set of globally ignored rules.
    pub global_ignored: BTreeSet<RuleCode>,
    /// A set of failures collected for each rule.
    pub failed: Failures,
}

/// Haskell `run`.
pub fn run(config: &Configuration, dockerfile: &[InstructionPos<String>]) -> Failures {
    let AnalisisResult {
        ignored,
        stage_ignored,
        global_ignored,
        failed,
    } = analyze(config, dockerfile);

    let ignores =
        |line: i64| -> BTreeSet<RuleCode> { ignored.get(&line).cloned().unwrap_or_default() };
    let stage_ignores = |line: i64| -> BTreeSet<RuleCode> {
        // `ls = 0 : [x | x <- keys, x < line]`; look up `last ls`.
        let last = stage_ignored
            .keys()
            .copied()
            .rfind(|x| *x < line)
            .unwrap_or(0);
        stage_ignored.get(&last).cloned().unwrap_or_default()
    };
    let should_keep = |f: &CheckFailure| -> bool {
        if config.disable_ignore_pragma {
            return true;
        }
        let code = &f.code;
        !(ignores(f.line).contains(code)
            || stage_ignores(f.line).contains(code)
            || global_ignored.contains(code))
    };
    failed.into_iter().filter(|f| should_keep(f)).collect()
}

/// Haskell `analyze`: run the pragma folds and the combined rule fold over the dockerfile.
pub fn analyze(config: &Configuration, dockerfile: &[InstructionPos<String>]) -> AnalisisResult {
    let ignored = pragma::ignored(dockerfile);
    let stage_ignored = pragma::stage_ignored(dockerfile);
    let global_ignored = pragma::global_ignored(dockerfile);
    let mut rule = rules::failures(config);
    for ip in dockerfile {
        rule.step(&parse_shell(ip));
    }
    AnalisisResult {
        ignored,
        stage_ignored,
        global_ignored,
        failed: rule.done(),
    }
}

/// Haskell `parseShell` (`fmap Shell.parseShell`).
pub fn parse_shell(ip: &InstructionPos<String>) -> InstructionPos<ParsedShell> {
    ip.clone().fmap(&|s: String| shell::parse_shell(&s))
}
