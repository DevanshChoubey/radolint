// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3020.hs.

use crate::rule::{DLSeverity, Rule, archive_file_format_extensions, drop_quotes, simple_rule};
use dlint_syntax::Instruction;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3020";
    let severity = DLSeverity::DLErrorC;
    let message = "Use COPY instead of ADD for files and folders";
    let check = |instr: &Instruction<_>| match instr {
        Instruction::Add(add_args, _) => add_args
            .source_paths
            .iter()
            .all(|src| is_archive(&src.un_source_path) || is_url(&src.un_source_path)),
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

/// Haskell `isArchive`.
fn is_archive(path: &str) -> bool {
    let p = drop_quotes(path);
    archive_file_format_extensions()
        .iter()
        .any(|ftype| p.ends_with(ftype))
}

/// Haskell `isUrl`.
fn is_url(path: &str) -> bool {
    let p = drop_quotes(path);
    ["https://", "http://"]
        .iter()
        .any(|proto| p.starts_with(proto))
}

#[cfg(test)]
mod tests {
    use crate::testing::{rule_catches, rule_catches_not};

    #[test]
    fn add_for_tar() {
        rule_catches_not("DL3020", "ADD file.tar /usr/src/app/");
    }
    #[test]
    fn add_for_gzip() {
        rule_catches_not("DL3020", "ADD file.gz /usr/src/app/");
    }
    #[test]
    fn add_for_bz2() {
        rule_catches_not("DL3020", "ADD file.bz2 /usr/src/app/");
    }
    #[test]
    fn add_for_xz() {
        rule_catches_not("DL3020", "ADD file.xz /usr/src/app/");
    }
    #[test]
    fn add_for_tgz() {
        rule_catches_not("DL3020", "ADD file.tgz /usr/src/app/");
    }
    #[test]
    fn add_for_url() {
        rule_catches_not("DL3020", "ADD http://file.com /usr/src/app/");
    }
    #[test]
    fn using_add() {
        rule_catches("DL3020", "ADD file /usr/src/app/");
    }
    #[test]
    fn warn_for_zip() {
        rule_catches("DL3020", "ADD file.zip /usr/src/app/");
    }
    #[test]
    fn add_for_tgz_with_quotes() {
        rule_catches_not("DL3020", "ADD \"file.tgz\" /usr/src/app/");
    }
    #[test]
    fn add_for_url_with_quotes() {
        rule_catches_not("DL3020", "ADD \"http://file.com\" /usr/src/app/");
    }
    #[test]
    fn warn_for_zip_with_quotes() {
        rule_catches("DL3020", "ADD \"file.zip\" /usr/src/app/");
    }
}
