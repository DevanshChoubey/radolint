// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3043.hs.

use crate::rule::{DLSeverity, Rule, simple_rule};
use crate::shell::ParsedShell;
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3043";
    let severity = DLSeverity::DLErrorC;
    let message = "`ONBUILD`, `FROM` or `MAINTAINER` triggered from within `ONBUILD` instruction.";

    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::OnBuild(inner) => !matches!(
            **inner,
            Instruction::OnBuild(_) | Instruction::From(_) | Instruction::Maintainer(_)
        ),
        _ => true,
    };

    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn error_when_using_onbuild_within_onbuild() {
        rule_catches("DL3043", "ONBUILD ONBUILD RUN anything\n");
    }

    #[test]
    fn error_when_using_from_within_onbuild() {
        rule_catches("DL3043", "ONBUILD FROM debian:buster\n");
    }

    #[test]
    fn error_when_using_maintainer_within_onbuild() {
        rule_catches("DL3043", "ONBUILD MAINTAINER \"BoJack Horseman\"\n");
    }

    #[test]
    fn ok_with_add() {
        rule_catches_not("DL3043", "ONBUILD ADD anything anywhere");
    }

    #[test]
    fn ok_with_user() {
        rule_catches_not("DL3043", "ONBUILD USER anything");
    }

    #[test]
    fn ok_with_label() {
        rule_catches_not("DL3043", "ONBUILD LABEL bla=\"blubb\"");
    }

    #[test]
    fn ok_with_stopsignal() {
        rule_catches_not("DL3043", "ONBUILD STOPSIGNAL anything");
    }

    #[test]
    fn ok_with_copy() {
        rule_catches_not("DL3043", "ONBUILD COPY anything anywhere");
    }

    #[test]
    fn ok_with_run() {
        rule_catches_not("DL3043", "ONBUILD RUN anything");
    }

    #[test]
    fn ok_with_cmd() {
        rule_catches_not("DL3043", "ONBUILD CMD anything");
    }

    #[test]
    fn ok_with_shell() {
        rule_catches_not("DL3043", "ONBUILD SHELL anything");
    }

    #[test]
    fn ok_with_workdir() {
        rule_catches_not("DL3043", "ONBUILD WORKDIR anything");
    }

    #[test]
    fn ok_with_expose() {
        rule_catches_not("DL3043", "ONBUILD EXPOSE 69");
    }

    #[test]
    fn ok_with_volume() {
        rule_catches_not("DL3043", "ONBUILD VOLUME anything");
    }

    #[test]
    fn ok_with_entrypoint() {
        rule_catches_not("DL3043", "ONBUILD ENTRYPOINT anything");
    }

    #[test]
    fn ok_with_env() {
        rule_catches_not("DL3043", "ONBUILD ENV MYVAR=\"bla\"");
    }

    #[test]
    fn ok_with_arg() {
        rule_catches_not("DL3043", "ONBUILD ARG anything");
    }

    #[test]
    fn ok_with_healthcheck() {
        rule_catches_not("DL3043", "ONBUILD HEALTHCHECK NONE");
    }

    #[test]
    fn ok_with_from_outside_of_onbuild() {
        rule_catches_not("DL3043", "FROM debian:buster");
    }

    #[test]
    fn ok_with_maintainer_outside_of_onbuild() {
        rule_catches_not("DL3043", "MAINTAINER \"Some Guy\"");
    }
}
