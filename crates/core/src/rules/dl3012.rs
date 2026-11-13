// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3012.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, add_fail, custom_rule, empty_state, replace_with,
};
use dlint_syntax::Instruction;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3012";
    let severity = DLSeverity::DLErrorC;
    let message = "Multiple `HEALTHCHECK` instructions";
    custom_rule(
        move |line, st, instr| match instr {
            Instruction::From(_) => replace_with(false, st),
            Instruction::Healthcheck(_) => {
                if !st.state {
                    replace_with(true, st)
                } else {
                    add_fail(
                        CheckFailure {
                            code: RuleCode::from(code),
                            severity,
                            message: message.to_string(),
                            line,
                        },
                        st,
                    )
                }
            }
            _ => st,
        },
        empty_state(false),
    )
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn ok_with_no_healthcheck_instruction() {
        rule_catches_not("DL3012", "FROM scratch");
    }
    #[test]
    fn ok_with_one_healthcheck_instruction() {
        rule_catches_not("DL3012", "FROM scratch\nHEALTHCHECK CMD /bin/bla");
    }
    #[test]
    fn ok_with_two_healthcheck_instructions_in_two_stages() {
        rule_catches_not(
            "DL3012",
            "FROM scratch\nHEALTHCHECK CMD /bin/bla1\nFROM scratch\nHEALTHCHECK CMD /bin/bla2",
        );
    }
    #[test]
    fn warn_with_two_healthcheck_instructions() {
        rule_catches(
            "DL3012",
            "FROM scratch\nHEALTHCHECK CMD /bin/bla1\nHEALTHCHECK CMD /bin/bla2",
        );
    }
}
