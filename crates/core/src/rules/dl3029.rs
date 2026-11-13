// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3029.hs.

use crate::rule::{simple_rule, DLSeverity, Rule};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3029";
    let severity = DLSeverity::DLWarningC;
    let message = "Do not use --platform flag with FROM";
    simple_rule(code, severity, message, |instr| match instr {
        Instruction::From(BaseImage { platform: Some(p), .. }) => {
            p.contains("BUILDPLATFORM") || p.contains("TARGETPLATFORM")
        }
        _ => true,
    })
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn explicit_platform_flag() {
        rule_catches("DL3029", "FROM --platform=linux debian:jessie");
    }
    #[test]
    fn no_platform_flag() {
        rule_catches_not("DL3029", "FROM debian:jessie");
    }
    #[test]
    fn allows_platform_buildplatform_flag() {
        rule_catches_not("DL3029", "FROM --platform=$BUILDPLATFORM debian:jessie");
    }
    #[test]
    fn allows_platform_quoted_buildplatform_flag() {
        rule_catches_not("DL3029", "FROM --platform=\"$BUILDPLATFORM\" debian:jessie");
    }
    #[test]
    fn allows_platform_braced_buildplatform_flag() {
        rule_catches_not("DL3029", "FROM --platform=${BUILDPLATFORM} debian:jessie");
    }
    #[test]
    fn allows_platform_braced_default_buildplatform_flag() {
        rule_catches_not("DL3029", "FROM --platform=${BUILDPLATFORM:-} debian:jessie");
    }
    #[test]
    fn allows_platform_quoted_braced_default_buildplatform_flag() {
        rule_catches_not("DL3029", "FROM --platform=\"${BUILDPLATFORM:-}\" debian:jessie");
    }
    #[test]
    fn allows_platform_targetplatform_flag() {
        rule_catches_not("DL3029", "FROM --platform=$TARGETPLATFORM debian:jessie");
    }
}
