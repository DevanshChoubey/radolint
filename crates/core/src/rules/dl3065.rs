// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3065.hs.

use crate::rule::{simple_rule, DLSeverity, Rule};
use dlint_syntax::Instruction;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3065";
    let severity = DLSeverity::DLWarningC;
    let message = "Setting `FROM --platform` to predefined `$TARGETPLATFORM` in is redundant as this is the default behavior";

    let check = |instr: &Instruction<_>| match instr {
        Instruction::From(base) => match &base.platform {
            Some(platform) => platform != "$TARGETPLATFORM" && platform != "${TARGETPLATFORM}",
            None => true,
        },
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn ok_from_no_platform() {
        rule_catches_not("DL3065", "FROM alpine:3.24");
    }

    #[test]
    fn ok_from_platform_not_targetplatform() {
        rule_catches_not("DL3065", "FROM --platform=$FOOBAR alpine:3.24");
        rule_catches_not("DL3065", "FROM --platform=foobar alpine:3.24");
        rule_catches_not("DL3065", "FROM --platform=foobar/arm64 alpine:3.24");
    }

    #[test]
    fn not_ok_from_platform_targetplatform() {
        rule_catches("DL3065", "FROM --platform=$TARGETPLATFORM alpine:3.24");
        rule_catches("DL3065", "FROM --platform=${TARGETPLATFORM} alpine:3.24");
    }
}
