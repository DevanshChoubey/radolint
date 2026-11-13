// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3007.hs.

use crate::rule::{simple_rule, DLSeverity, Rule};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3007";
    let severity = DLSeverity::DLWarningC;
    let message = "Using latest is prone to errors if the image will ever update. Pin the version explicitly \
                   to a release tag";
    let check = |instr: &Instruction<_>| match instr {
        Instruction::From(BaseImage { tag: Some(_), digest: Some(_), .. }) => true,
        Instruction::From(BaseImage { tag: Some(t), .. }) => t.un_tag != "latest",
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn explicit_latest() {
        rule_catches("DL3007", "FROM debian:latest");
    }

    #[test]
    fn explicit_latest_with_name() {
        rule_catches("DL3007", "FROM debian:latest AS builder");
    }

    #[test]
    fn explicit_tagged() {
        rule_catches_not("DL3007", "FROM debian:jessie");
    }

    #[test]
    fn explicit_sha() {
        rule_catches_not(
            "DL3007",
            "FROM hub.docker.io/debian@sha256:7959ed6f7e35f8b1aaa06d1d8259d4ee25aa85a086d5c125480c333183f9deeb",
        );
    }

    #[test]
    fn explicit_tagged_and_sha() {
        rule_catches_not(
            "DL3007",
            "FROM hub.docker.io/debian:latest@sha256:7959ed6f7e35f8b1aaa06d1d8259d4ee25aa85a086d5c125480c333183f9deeb",
        );
    }

    #[test]
    fn explicit_tagged_with_name() {
        rule_catches_not("DL3007", "FROM debian:jessie AS builder");
    }
}
