// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL4000.hs.

use crate::rule::{simple_rule, DLSeverity, Rule};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL4000";
    let severity = DLSeverity::DLErrorC;
    let message = "MAINTAINER is deprecated";
    let check = |instr: &Instruction<_>| !matches!(instr, Instruction::Maintainer(_));
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn has_deprecated_maintainer() {
        rule_catches("DL4000", "FROM busybox\nMAINTAINER hudu@mail.com");
    }

    #[test]
    fn has_maintainer() {
        rule_catches("DL4000", "FROM debian\nMAINTAINER Lukas");
    }

    #[test]
    fn has_maintainer_first() {
        rule_catches("DL4000", "MAINTAINER Lukas\nFROM DEBIAN");
    }

    #[test]
    fn has_no_maintainer() {
        rule_catches_not("DL4000", "FROM debian");
    }
}
