// SPDX-License-Identifier: GPL-3.0-only
//! Conformance runner: replays `conformance/corpus.jsonl` through the in-process lint pipeline
//! and compares with what the oracle (hadolint v2.15.1) printed, exactly and order-sensitively.
//!
//! Usage: `dlint-conformance [--verbose] [--filter <id-substring>] [--corpus <path>]`
//!
//! A case passes when the JSON output (`--no-color -f json [args] -`) equals `oracle.json` and the
//! TTY output (`--no-color [args] -`) equals `oracle.tty`. Cases with `oracle_forced` are also run
//! with the forced `--error RULE` configuration against `oracle_forced.json`.
//! Panics (e.g. `todo!()` stubs) are caught per case and counted as failures.

use dlint_core::config::configuration::{apply_partial_configuration, partial_from_yaml, Configuration};
use dlint_core::formatter::format::{to_result, FormatResult};
use dlint_core::formatter::{json, tty};
use dlint_core::lint;
use dlint_core::rule::{read_either_label_type, RuleCode};
use dlint_syntax::Registry;
use serde_json::Value;
use std::panic::{catch_unwind, AssertUnwindSafe};

struct Opts {
    verbose: bool,
    filter: Option<String>,
    corpus: String,
}

fn parse_opts() -> Opts {
    let mut o = Opts { verbose: false, filter: None, corpus: "conformance/corpus.jsonl".to_string() };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--verbose" | "-v" => o.verbose = true,
            "--filter" => o.filter = it.next(),
            "--corpus" => {
                if let Some(p) = it.next() {
                    o.corpus = p
                }
            }
            other => {
                eprintln!("unknown option {other}");
                std::process::exit(2);
            }
        }
    }
    o
}

/// Build the `Configuration` equivalent to the CLI args recorded in the corpus.
fn config_from(args: &[String], extra_error: &[String], config_yaml: Option<&str>) -> Result<Configuration, String> {
    let mut config = Configuration::default();
    if let Some(y) = config_yaml {
        config = apply_partial_configuration(config, partial_from_yaml(y)?);
    }
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--strict-labels" => config.strict_labels = true,
            "--trusted-registry" => {
                let r = it.next().ok_or("--trusted-registry needs a value")?;
                config.allowed_registries.insert(Registry::from(r.as_str()));
            }
            "--require-label" => {
                let v = it.next().ok_or("--require-label needs a value")?;
                let (name, ty) = v.split_once(':').ok_or_else(|| format!("bad label schema {v}"))?;
                config.label_schema.insert(name.to_string(), read_either_label_type(ty)?);
            }
            "--error" => config.error_rules.push(RuleCode::from(it.next().ok_or("--error needs a value")?.as_str())),
            "--warning" => config.warning_rules.push(RuleCode::from(it.next().ok_or("--warning needs a value")?.as_str())),
            "--info" => config.info_rules.push(RuleCode::from(it.next().ok_or("--info needs a value")?.as_str())),
            "--style" => config.style_rules.push(RuleCode::from(it.next().ok_or("--style needs a value")?.as_str())),
            "--ignore" => config.ignore_rules.push(RuleCode::from(it.next().ok_or("--ignore needs a value")?.as_str())),
            other => return Err(format!("unsupported corpus arg {other}")),
        }
    }
    for e in extra_error {
        config.error_rules.push(RuleCode::from(e.as_str()));
    }
    Ok(config)
}

/// Run the pipeline the way the CLI does for stdin input: parse errors report "/dev/stdin",
/// checks report "-".
fn run_case(src: &[u8], config: &Configuration) -> FormatResult {
    let parsed = dlint_parser::parser::do_parse("/dev/stdin", src);
    to_result("-", parsed.map(|d| lint::analyze(config, &d)))
}

fn tty_output(r: &FormatResult) -> String {
    let mut buf = vec![];
    tty::h_write(&mut buf, std::slice::from_ref(r), true).expect("write");
    String::from_utf8_lossy(&buf).into_owned()
}

/// Minimal standard-alphabet base64 decoder (avoids a dependency).
fn b64_decode(s: &str) -> Vec<u8> {
    let mut out = vec![];
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        };
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    out
}

struct Outcome {
    ok: bool,
    detail: String,
}

fn check(rec: &Value) -> Outcome {
    let src: Vec<u8> = if let Some(b) = rec.get("dockerfile_b64").and_then(|v| v.as_str()) {
        b64_decode(b)
    } else {
        rec["dockerfile"].as_str().unwrap_or("").as_bytes().to_vec()
    };
    let args: Vec<String> = rec["args"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let yaml = rec["config_yaml"].as_str();
    let mut ok = true;
    let mut detail = String::new();

    let config = match config_from(&args, &[], yaml) {
        Ok(c) => c,
        Err(e) => return Outcome { ok: false, detail: format!("config error: {e}") },
    };
    let res = run_case(&src, &config);
    let got_json: Value = serde_json::from_str(&json::render(std::slice::from_ref(&res))).unwrap_or(Value::Null);
    let want_json = &rec["oracle"]["json"];
    if &got_json != want_json {
        ok = false;
        detail += &format!("json:\n  want {want_json}\n  got  {got_json}\n");
    }
    if let Some(want_tty) = rec["oracle"]["tty"].as_str() {
        let got_tty = tty_output(&res);
        if got_tty != want_tty {
            ok = false;
            detail += &format!("tty:\n  want {want_tty:?}\n  got  {got_tty:?}\n");
        }
    }
    if let Some(forced) = rec.get("oracle_forced").filter(|v| v.is_object()) {
        let fargs: Vec<String> = forced["args"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        match config_from(&args, &fargs.iter().filter(|a| !a.starts_with("--")).cloned().collect::<Vec<_>>(), yaml) {
            Ok(c) => {
                let r = run_case(&src, &c);
                let g: Value = serde_json::from_str(&json::render(std::slice::from_ref(&r))).unwrap_or(Value::Null);
                if g != forced["json"] {
                    ok = false;
                    detail += &format!("forced json:\n  want {}\n  got  {g}\n", forced["json"]);
                }
            }
            Err(e) => {
                ok = false;
                detail += &format!("forced config error: {e}\n");
            }
        }
    }
    Outcome { ok, detail }
}

fn main() {
    let opts = parse_opts();
    let text = match std::fs::read_to_string(&opts.corpus) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {}: {e}", opts.corpus);
            std::process::exit(2);
        }
    };
    // Stub bodies panic: keep the output readable.
    std::panic::set_hook(Box::new(|_| {}));
    let (mut pass, mut fail) = (0usize, 0usize);
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let rec: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("bad corpus line: {e}");
                continue;
            }
        };
        let id = rec["id"].as_str().unwrap_or("?").to_string();
        if let Some(f) = &opts.filter {
            if !id.contains(f.as_str()) {
                continue;
            }
        }
        let outcome = match catch_unwind(AssertUnwindSafe(|| check(&rec))) {
            Ok(o) => o,
            Err(p) => {
                let msg = p
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| "panic".to_string());
                Outcome { ok: false, detail: format!("panic: {msg}\n") }
            }
        };
        if outcome.ok {
            pass += 1;
        } else {
            fail += 1;
            if opts.verbose {
                println!("FAIL {id}\n{}", outcome.detail);
            }
        }
    }
    println!("pass {pass} / fail {fail}");
}
