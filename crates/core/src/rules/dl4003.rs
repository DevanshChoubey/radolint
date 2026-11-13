// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL4003.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, add_fail, custom_rule, empty_state, replace_with,
};
use radolint_syntax::Instruction;

/// Haskell `data HasCmd = HasCmd | NoCmd`.
#[derive(Debug, Clone, Copy)]
enum HasCmd {
    HasCmd,
    NoCmd,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL4003";
    let severity = DLSeverity::DLWarningC;
    let message = "Multiple `CMD` instructions found. If you list more than one `CMD` then only the last `CMD` will take effect";
    custom_rule(
        move |line, st, instr| match (instr, st.state) {
            // Reset the state each time we find a FROM
            (Instruction::From(_), _) => replace_with(HasCmd::NoCmd, st),
            // Remember we found a CMD, fail if we found a CMD before
            (Instruction::Cmd(_), HasCmd::NoCmd) => replace_with(HasCmd::HasCmd, st),
            // If we already saw a CMD, add an error for the line
            (Instruction::Cmd(_), HasCmd::HasCmd) => add_fail(
                CheckFailure {
                    code: RuleCode::from(code),
                    severity,
                    message: message.to_string(),
                    line,
                },
                st,
            ),
            // otherwise return the accumulator
            _ => st,
        },
        empty_state(HasCmd::NoCmd),
    )
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn many_cmds() {
        rule_catches("DL4003", "FROM debian\nCMD bash\nRUN foo\nCMD another\n");
    }
    #[test]
    fn single_cmds_different_stages() {
        rule_catches_not(
            "DL4003",
            "FROM debian as distro1\nCMD bash\nRUN foo\nFROM debian as distro2\nCMD another\n",
        );
    }
    #[test]
    fn many_cmds_different_stages() {
        rule_catches(
            "DL4003",
            "FROM debian as distro1\nCMD bash\nRUN foo\nCMD another\nFROM debian as distro2\nCMD another\n",
        );
    }
    #[test]
    fn single_cmd() {
        rule_catches_not("DL4003", "CMD /bin/true");
    }
}
