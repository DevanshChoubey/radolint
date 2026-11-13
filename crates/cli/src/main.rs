// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/app/Main.hs.

use radolint_core::config::commandline::{
    CommandlineConfig, CommandlineOutcome, error_text, parse_commandline_as, prog_name,
};
use radolint_core::config::configfile::get_config_from_file;
use radolint_core::config::configuration::{
    Configuration, PartialConfiguration, apply_partial_configuration,
};
use radolint_core::config::environment::get_config_from_environment;
use radolint_core::formatter::format::{FormatResult, OutputFormat};
use radolint_core::formatter::write;
use radolint_core::lint::lint_io;
use radolint_core::meta::get_version;
use radolint_core::rule::DLSeverity;
use std::process::exit;

/// Haskell `noFailure`.
fn no_failure(r: &FormatResult, cutoff: DLSeverity) -> bool {
    if !r.errors.is_empty() {
        return false;
    }
    r.checks.iter().all(|f| f.severity > cutoff)
}

/// Haskell `exitProgram`.
fn exit_program(conf: &Configuration, res: &[FormatResult]) -> ! {
    if conf.no_fail
        || conf.formats.contains(&OutputFormat::CodeclimateJson)
        || conf.formats.contains(&OutputFormat::Codacy)
        || res.iter().all(|r| no_failure(r, conf.failure_threshold))
    {
        exit(0)
    }
    exit(1)
}

/// Haskell `runLint`.
fn run_lint(cmd: &CommandlineConfig, conf: &Configuration) -> ! {
    let res = match lint_io(conf, &cmd.dockerfiles) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        }
    };
    if let Err(e) = write(
        &cmd.output,
        &conf.formats,
        conf.no_color,
        cmd.file_path_in_report_option.as_deref(),
        &res,
    ) {
        eprintln!("{e}");
        exit(1)
    }
    exit_program(conf, &res)
}

/// Haskell `execute`.
fn execute(cmd: &CommandlineConfig, config: &Configuration) -> ! {
    if cmd.show_version {
        println!("{}", get_version());
        exit(0)
    }
    if cmd.dockerfiles.is_empty() {
        println!("Please provide a Dockerfile");
        exit(1)
    }
    run_lint(cmd, config)
}

fn main() {
    let mut argv = std::env::args();
    let prog = prog_name(argv.next().as_deref());
    let args: Vec<String> = argv.collect();
    let invoked_with = match parse_commandline_as(&prog, &args) {
        CommandlineOutcome::Config(c) => *c,
        CommandlineOutcome::Help(t) => {
            print!("{t}");
            exit(0)
        }
        CommandlineOutcome::Error(e) => {
            eprint!("{}", error_text(&prog, &e));
            exit(1)
        }
    };
    let from_environment = get_config_from_environment();
    let from_commandline = invoked_with.configuration.clone();
    let verbose = from_commandline.partial_verbose == Some(true);
    let from_configfile = match get_config_from_file(invoked_with.config_file.as_deref(), verbose) {
        Ok(Ok(c)) => c,
        Ok(Err(err)) => {
            eprintln!("{err}");
            PartialConfiguration::mempty()
        }
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        }
    };
    let running_config = apply_partial_configuration(
        Configuration::default(),
        from_environment
            .append(from_configfile)
            .append(from_commandline),
    );
    if running_config.verbose {
        eprintln!("{}", running_config.pretty());
    }
    execute(&invoked_with, &running_config)
}
