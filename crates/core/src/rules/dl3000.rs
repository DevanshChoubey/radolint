// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3000.hs.

use crate::rule::{DLSeverity, Rule, drop_quotes, simple_rule};
use radolint_syntax::Instruction;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3000";
    let severity = DLSeverity::DLErrorC;
    let message = "Use absolute WORKDIR";
    let check = |instr: &Instruction<_>| match instr {
        Instruction::Workdir(loc) => {
            let d = drop_quotes(loc);
            if d.starts_with('$') {
                true
            } else if d.starts_with('/') {
                true
            } else {
                is_windows_absolute(&d)
            }
        }
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

/// Haskell `isWindowsAbsolute`.
// DIVERGENCE: Haskell `Text.index` errors on out-of-range; here a too-short path yields False.
fn is_windows_absolute(path: &str) -> bool {
    let mut it = path.chars();
    match (it.next(), it.next()) {
        (Some(a), Some(b)) => a.is_alphabetic() && b == ':',
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::{rule_catches, rule_catches_not};

    #[test]
    fn workdir_relative() {
        rule_catches("DL3000", "WORKDIR relative/dir");
    }
    #[test]
    fn workdir_absolute() {
        rule_catches_not("DL3000", "WORKDIR /usr/local");
    }
    #[test]
    fn workdir_variable() {
        rule_catches_not("DL3000", "WORKDIR ${work}");
    }
    #[test]
    fn workdir_relative_single_quotes() {
        rule_catches("DL3000", "WORKDIR 'relative/dir'");
    }
    #[test]
    fn workdir_absolute_single_quotes() {
        rule_catches_not("DL3000", "WORKDIR '/usr/local'");
    }
    #[test]
    fn workdir_relative_double_quotes() {
        rule_catches("DL3000", "WORKDIR \"relative/dir\"");
    }
    #[test]
    fn workdir_absolute_double_quotes() {
        rule_catches_not("DL3000", "WORKDIR \"/usr/local\"");
    }
    #[test]
    fn workdir_variable_double_quotes() {
        rule_catches_not("DL3000", "WORKDIR \"${dir}\"");
    }
    #[test]
    fn workdir_absolute_windows() {
        rule_catches_not("DL3000", "WORKDIR 'C:\\'");
    }
    #[test]
    fn workdir_absolute_windows_quotes() {
        rule_catches_not("DL3000", "WORKDIR \"C:\\\"");
    }
    #[test]
    fn workdir_absolute_windows_alternative() {
        rule_catches_not("DL3000", "WORKDIR C:/");
    }
    #[test]
    fn workdir_absolute_windows_quotes_alternative() {
        rule_catches_not("DL3000", "WORKDIR \"C:/\"");
    }
}
