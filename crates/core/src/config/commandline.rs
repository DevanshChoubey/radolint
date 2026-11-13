// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Config/Commandline.hs (optparse-applicative parser),
//! hand-written so error texts can be matched to optparse-applicative's.

use crate::config::configuration::PartialConfiguration;
use crate::formatter::format::read_maybe_output_format;
use crate::rule::{read_either_label_type, read_either_severity, RuleCode};
use dlint_syntax::Registry;

/// Haskell `CommandlineConfig`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandlineConfig {
    pub show_version: bool,
    pub config_file: Option<String>,
    pub dockerfiles: Vec<String>,
    pub file_path_in_report_option: Option<String>,
    pub output: Vec<String>,
    pub configuration: PartialConfiguration,
}

/// Result of parsing argv: a config, `--help` (usage text, exit 0) or an error (message, exit 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandlineOutcome {
    Config(Box<CommandlineConfig>),
    Help(String),
    Error(String),
}

/// optparse-applicative usage/help text.
pub fn usage_text() -> String {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Commandline.hs (help texts as rendered by
    // optparse-applicative's `execParser`, header "hadolint - Dockerfile Linter written in Haskell").
    String::new()
}

/// Haskell `parseCommandline` run by `execParser` over `args` (without the program name).
pub fn parse_commandline(args: &[String]) -> CommandlineOutcome {
    // TODO(port): error texts / usage layout of optparse-applicative (e.g. "Invalid option `--bogus'"
    // followed by the usage block); the option semantics below are complete.
    let mut c = CommandlineConfig::default();
    let mut it = args.iter().peekable();
    let mut only_files = false;
    while let Some(a) = it.next() {
        if only_files || !a.starts_with('-') || a == "-" {
            c.dockerfiles.push(a.clone());
            continue;
        }
        if a == "--" {
            only_files = true;
            continue;
        }
        // split `--long=value`
        let (name, inline_val) = match a.split_once('=') {
            Some((n, v)) if n.starts_with("--") => (n.to_string(), Some(v.to_string())),
            _ => (a.clone(), None),
        };
        macro_rules! value {
            () => {
                match inline_val.clone().or_else(|| it.next().cloned()) {
                    Some(v) => v,
                    None => return CommandlineOutcome::Error(format!("The option `{name}' expects an argument.")),
                }
            };
        }
        let p = &mut c.configuration;
        match name.as_str() {
            "--version" | "-v" => c.show_version = true,
            "-h" | "--help" => return CommandlineOutcome::Help(usage_text()),
            "--config" | "-c" => c.config_file = Some(value!()),
            "--file-path-in-report" => c.file_path_in_report_option = Some(value!()),
            "--output" | "-o" => c.output.push(value!()),
            "--no-fail" => p.partial_no_fail = Some(true),
            "--no-color" => p.partial_no_color = Some(true),
            "--verbose" | "-V" => p.partial_verbose = Some(true),
            "--format" | "-f" => {
                let v = value!();
                match read_maybe_output_format(&v) {
                    Some(f) => p.partial_formats.push(f),
                    None => return CommandlineOutcome::Error(format!("option {name}: cannot parse value `{v}'")),
                }
            }
            "--error" => p.partial_error_rules.push(RuleCode::from(value!())),
            "--warning" => p.partial_warning_rules.push(RuleCode::from(value!())),
            "--info" => p.partial_info_rules.push(RuleCode::from(value!())),
            "--style" => p.partial_style_rules.push(RuleCode::from(value!())),
            "--ignore" => p.partial_ignore_rules.push(RuleCode::from(value!())),
            "--trusted-registry" => {
                p.partial_allowed_registries.insert(Registry::from(value!().as_str()));
            }
            "--require-label" => {
                let v = value!();
                match label_parser(&v) {
                    Ok((k, t)) => {
                        p.partial_label_schema.insert(k, t); // Map.fromList: later duplicates win
                    }
                    Err(e) => return CommandlineOutcome::Error(format!("option --require-label: {e}")),
                }
            }
            "--strict-labels" => p.partial_strict_labels = Some(true),
            "--disable-ignore-pragma" => p.partial_disable_ignore_pragma = Some(true),
            "--failure-threshold" | "-t" => {
                let v = value!();
                match read_either_severity(&v) {
                    Ok(s) => p.partial_failure_threshold = Some(s),
                    Err(e) => return CommandlineOutcome::Error(format!("option {name}: {e}")),
                }
            }
            _ => return CommandlineOutcome::Error(format!("Invalid option `{a}'")),
        }
    }
    CommandlineOutcome::Config(Box::new(c))
}

/// Haskell `labelParser` (`breakOn ":"`, then `readEitherLabelType . drop 1`).
pub fn label_parser(l: &str) -> Result<(String, crate::rule::LabelType), String> {
    let (ln, rest) = match l.find(':') {
        Some(i) => (&l[..i], &l[i + 1..]),
        None => (l, ""),
    };
    read_either_label_type(rest).map(|lt| (ln.to_string(), lt))
}
