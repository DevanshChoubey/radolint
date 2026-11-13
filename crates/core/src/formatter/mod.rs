// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter.hs.

pub mod checkstyle;
pub mod codacy;
pub mod codeclimate;
pub mod format;
pub mod gnu;
pub mod json;
pub mod junit;
pub mod sarif;
pub mod sonarqube;
pub mod tty;

pub use format::{read_maybe_output_format, FormatResult, OutputFormat};

use std::io::Write;

/// Haskell `Formatter.hWrite`.
pub fn h_write(
    handle: &mut dyn Write,
    format: OutputFormat,
    nocolor: bool,
    file_path_in_report: Option<&str>,
    all_results: &[FormatResult],
) -> std::io::Result<()> {
    match format {
        OutputFormat::Checkstyle => checkstyle::h_write(handle, all_results, file_path_in_report),
        OutputFormat::Codacy => codacy::h_write(handle, all_results),
        OutputFormat::CodeclimateJson => codeclimate::h_write(handle, all_results, file_path_in_report),
        OutputFormat::GitLabCodeclimateJson => codeclimate::h_write_gitlab(handle, all_results, file_path_in_report),
        OutputFormat::Gnu => gnu::h_write(handle, all_results),
        OutputFormat::JUnit => junit::h_write(handle, all_results, file_path_in_report),
        OutputFormat::Json => json::h_write(handle, all_results),
        OutputFormat::Sarif => sarif::h_write(handle, all_results),
        OutputFormat::SonarQube => sonarqube::h_write(handle, all_results, file_path_in_report),
        OutputFormat::TTY => tty::h_write(handle, all_results, nocolor),
    }
}

/// Haskell `Formatter.write`.
pub fn write(
    paths: &[String],
    formats: &[OutputFormat],
    nocolor: bool,
    file_path_in_report: Option<&str>,
    all_results: &[FormatResult],
) -> std::io::Result<()> {
    for (p, f) in zip_with_def("-", OutputFormat::TTY, paths, formats) {
        if p == "-" {
            let stdout = std::io::stdout();
            let mut h = stdout.lock();
            h_write(&mut h, f, nocolor, file_path_in_report, all_results)?;
            h.flush()?;
        } else {
            let mut h = std::fs::File::create(&p)?;
            h_write(&mut h, f, nocolor, file_path_in_report, all_results)?;
        }
    }
    Ok(())
}

/// Haskell `zipWithDef`.
pub fn zip_with_def(
    def_p: &str,
    def_f: OutputFormat,
    paths: &[String],
    formats: &[OutputFormat],
) -> Vec<(String, OutputFormat)> {
    match (paths, formats) {
        ([], []) => vec![(def_p.to_string(), def_f)],
        ([], [f]) => vec![(def_p.to_string(), *f)],
        ([p], []) => vec![(p.clone(), def_f)],
        ([p], [f]) => vec![(p.clone(), *f)],
        ([], [f, fs @ ..]) => {
            let mut v = vec![(def_p.to_string(), *f)];
            v.extend(zip_with_def(def_p, def_f, &[], fs));
            v
        }
        ([p, ps @ ..], []) => {
            let mut v = vec![(p.clone(), def_f)];
            v.extend(zip_with_def(def_p, def_f, ps, &[]));
            v
        }
        ([p, ps @ ..], [f, fs @ ..]) => {
            let mut v = vec![(p.clone(), *f)];
            v.extend(zip_with_def(def_p, def_f, ps, fs));
            v
        }
    }
}
