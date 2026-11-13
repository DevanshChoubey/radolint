// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3021.hs.

use crate::rule::{DLSeverity, Rule, drop_quotes, simple_rule};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3021";
    let severity = DLSeverity::DLErrorC;
    let message = "COPY with more than 2 arguments requires the last argument to end with /";
    let check = |instr: &Instruction<_>| match instr {
        Instruction::Copy(
            CopyArgs {
                source_paths,
                target_path,
            },
            _,
        ) if source_paths.len() > 1 => ends_with_slash(target_path),
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

/// Haskell `endsWithSlash`.
pub fn ends_with_slash(t: &TargetPath) -> bool {
    let t = &t.un_target_path;
    !t.is_empty() && drop_quotes(t).ends_with('/')
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn no_warn_on_2_args() {
        rule_catches_not("DL3021", "COPY foo bar");
    }

    #[test]
    fn warn_on_3_args() {
        rule_catches("DL3021", "COPY foo bar baz");
    }

    #[test]
    fn no_warn_on_3_args_slash() {
        rule_catches_not("DL3021", "COPY foo bar baz/");
    }

    #[test]
    fn warn_on_3_args_with_quotes() {
        rule_catches("DL3021", "COPY foo bar \"baz\"");
    }

    #[test]
    fn no_warn_on_3_args_with_quotes() {
        rule_catches_not("DL3021", "COPY foo bar \"baz/\"");
    }
}
