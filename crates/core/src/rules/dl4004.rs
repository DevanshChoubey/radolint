// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL4004.hs.

use crate::rule::{
    add_fail, custom_rule, empty_state, replace_with, CheckFailure, DLSeverity, Rule, RuleCode,
};
use dlint_syntax::Instruction;

/// Haskell `data HasEntry = HasEntry | NoEntry`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HasEntry {
    HasEntry,
    NoEntry,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL4004";
    let severity = DLSeverity::DLErrorC;
    let message = "Multiple `ENTRYPOINT` instructions found. If you list more than one `ENTRYPOINT` then only the last `ENTRYPOINT` will take effect";
    custom_rule(
        move |line, st, instr| match instr {
            // Reset the state each time we find a FROM
            Instruction::From(_) => replace_with(HasEntry::NoEntry, st),
            // Remember we found an ENTRYPOINT
            Instruction::Entrypoint(_) if st.state == HasEntry::NoEntry => {
                replace_with(HasEntry::HasEntry, st)
            }
            // Add a failure if we found another entrypoint in the same stage
            Instruction::Entrypoint(_) => add_fail(
                CheckFailure {
                    code: RuleCode::from(code),
                    severity,
                    message: message.to_string(),
                    line,
                },
                st,
            ),
            _ => st,
        },
        empty_state(HasEntry::NoEntry),
    )
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn no_cmd() {
        rule_catches_not("DL4004", "FROM busybox");
    }
    #[test]
    fn many_entrypoints() {
        rule_catches(
            "DL4004",
            "FROM debian\nENTRYPOINT bash\nRUN foo\nENTRYPOINT another\n",
        );
    }
    #[test]
    fn single_entrypoint_different_stages() {
        rule_catches_not(
            "DL4004",
            "FROM debian as distro1\nENTRYPOINT bash\nRUN foo\nFROM debian as distro2\nENTRYPOINT another\n",
        );
    }
    #[test]
    fn many_entrypoints_different_stages() {
        rule_catches(
            "DL4004",
            "FROM debian as distro1\nENTRYPOINT bash\nRUN foo\nENTRYPOINT another\nFROM debian as distro2\nENTRYPOINT another\n",
        );
    }
    #[test]
    fn single_entry() {
        rule_catches_not("DL4004", "ENTRYPOINT /bin/true");
    }
    #[test]
    fn no_entry() {
        rule_catches_not("DL4004", "FROM busybox");
    }
}
