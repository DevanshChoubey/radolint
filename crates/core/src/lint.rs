// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Lint.hs.

use crate::config::configuration::Configuration;
use crate::formatter::format::{FormatResult, to_result};
use crate::process;
use crate::rule::{CheckFailure, DLSeverity, Failures, RuleCode};
use radolint_parser::ParseError;
use radolint_syntax::Dockerfile;

/// hadolint's RTS runs with `-N5`, so `parList` lints on at most five cores.
const MAX_WORKERS: usize = 5;

/// Worker stack, the same as the main thread's default on Linux, so the
/// recursive parser reaches the same nesting depth on either.
const WORKER_STACK_BYTES: usize = 8 << 20;

/// Haskell `lintIO`: parse each file (or stdin for "-") and lint it.
///
/// Files are read in order first, so an unreadable file fails the run before any
/// output, as `mapM parseFile` does. Parsing and linting then run in parallel.
pub fn lint_io(config: &Configuration, d_files: &[String]) -> std::io::Result<Vec<FormatResult>> {
    let mut inputs = Vec::with_capacity(d_files.len());
    for f in d_files {
        if f == "-" {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::io::stdin().read_to_end(&mut bytes)?;
            inputs.push(("-".to_string(), "/dev/stdin".to_string(), bytes));
        } else {
            inputs.push((f.clone(), f.clone(), std::fs::read(f)?));
        }
    }
    Ok(par_map(inputs, |(name, source, bytes)| {
        let res = radolint_parser::do_parse(&source, &bytes);
        to_result(&name, res.map(|d| analyze(config, &d)))
    }))
}

/// Haskell `lint`.
pub fn lint(
    config: &Configuration,
    parsed_files: Vec<(String, Result<Dockerfile, ParseError>)>,
) -> Vec<FormatResult> {
    par_map(parsed_files, |(name, res)| {
        to_result(&name, res.map(|d| analyze(config, &d)))
    })
}

/// `parList rseq`: map `f` over `items` on up to [`MAX_WORKERS`] threads,
/// keeping the input order.
fn par_map<T: Send, R: Send>(items: Vec<T>, f: impl Fn(T) -> R + Sync) -> Vec<R> {
    let workers = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(MAX_WORKERS)
        .min(items.len());
    // Each worker checks all its shell scripts on one shelllint big-stack
    // thread, rather than shelllint starting a thread per `RUN`.
    if workers <= 1 {
        return shelllint_checker::with_check_stack(|| items.into_iter().map(&f).collect());
    }
    let n = items.len();
    let queue = std::sync::Mutex::new(items.into_iter().enumerate());
    let mut out: Vec<Option<R>> = (0..n).map(|_| None).collect();
    let done = std::sync::Mutex::new(Vec::with_capacity(n));
    let work = || {
        loop {
            let next = queue.lock().unwrap().next();
            let Some((i, item)) = next else { break };
            let r = f(item);
            done.lock().unwrap().push((i, r));
        }
    };
    std::thread::scope(|s| {
        // The calling thread works the queue too, so a helper that cannot be
        // spawned (thread or memory limits) only costs parallelism.
        for _ in 1..workers {
            let _ = std::thread::Builder::new()
                .stack_size(WORKER_STACK_BYTES)
                .spawn_scoped(s, || shelllint_checker::with_check_stack(work));
        }
        shelllint_checker::with_check_stack(work);
    });
    for (i, r) in done.into_inner().unwrap() {
        out[i] = Some(r);
    }
    out.into_iter()
        .map(|r| r.expect("every item is mapped"))
        .collect()
}

/// Haskell `analyze`.
pub fn analyze(config: &Configuration, dockerfile: &Dockerfile) -> Failures {
    fix_severity(config, process::run(config, dockerfile))
}

/// Haskell `fixSeverity`.
pub fn fix_severity(config: &Configuration, failures: Failures) -> Failures {
    let make_severity =
        |s: DLSeverity, rules: &[RuleCode], mut rule: CheckFailure| -> CheckFailure {
            if rules.contains(&rule.code) {
                rule.severity = s;
            }
            rule
        };
    let correct = |rule: CheckFailure| -> CheckFailure {
        // composed right-to-left: style, info, warning, error (error applied last, so it wins)
        let r = make_severity(DLSeverity::DLStyleC, &config.style_rules, rule);
        let r = make_severity(DLSeverity::DLInfoC, &config.info_rules, r);
        let r = make_severity(DLSeverity::DLWarningC, &config.warning_rules, r);
        make_severity(DLSeverity::DLErrorC, &config.error_rules, r)
    };
    failures
        .into_iter()
        .map(correct)
        .filter(|f| !config.ignore_rules.contains(&f.code) && f.severity != DLSeverity::DLIgnoreC)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn par_map_keeps_order() {
        let items: Vec<usize> = (0..1000).collect();
        let out = par_map(items, |i| i * 2);
        assert_eq!(out, (0..1000).map(|i| i * 2).collect::<Vec<_>>());
    }

    #[test]
    fn lint_results_follow_input_order() {
        let config = Configuration::default();
        let files: Vec<_> = (0..40)
            .map(|i| {
                let src = if i % 3 == 0 {
                    "FROM a\nRUN cd /x\n"
                } else {
                    "FROM a:1\n"
                };
                (format!("f{i}"), radolint_parser::parse_text(src))
            })
            .collect();
        let expected: Vec<_> = files
            .iter()
            .map(|(n, r)| to_result(n, r.clone().map(|d| analyze(&config, &d))))
            .collect();
        assert_eq!(
            format!("{:?}", lint(&config, files)),
            format!("{expected:?}")
        );
    }
}
