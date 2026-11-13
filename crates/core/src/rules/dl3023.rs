// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3023.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, State, add_fail, alias_must_be, custom_rule,
    empty_state, pipe, replace_with,
};
use crate::shell::ParsedShell;
use dlint_syntax::{CopySource, Instruction, Linenumber};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3023";
    let severity = DLSeverity::DLErrorC;
    let message = "`COPY --from` cannot reference its own `FROM` alias";

    let check = move |line: Linenumber,
                      st: State<Option<Instruction<ParsedShell>>>,
                      instr: &Instruction<ParsedShell>| {
        match instr {
            // Remember the last FROM instruction found
            Instruction::From(_) => pipe(st, |s| replace_with(Some(instr.clone()), s)),
            Instruction::Copy(_, flags) => {
                if let (Some(from_instr), CopySource::CopySource(stage_name)) =
                    (&st.state, &flags.source_flag)
                {
                    if alias_must_be(|a| a != stage_name.as_str(), from_instr) {
                        st
                    } else {
                        // cannot copy from the same stage!
                        let failure = CheckFailure {
                            code: RuleCode::from(code),
                            severity,
                            message: message.to_string(),
                            line,
                        };
                        pipe(st, |s| add_fail(failure, s))
                    }
                } else {
                    st
                }
            }
            _ => st,
        }
    };
    custom_rule(check, empty_state(None))
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn warn_on_copying_from_the_same_from() {
        let docker_file = ["FROM node as foo", "COPY --from=foo bar ."];
        rule_catches("DL3023", &docker_file.join("\n"));
    }

    #[test]
    fn dont_warn_on_copying_from_other_sources() {
        let docker_file = [
            "FROM scratch as build",
            "RUN foo",
            "FROM node as run",
            "COPY --from=build foo .",
            "RUN baz",
        ];
        rule_catches_not("DL3023", &docker_file.join("\n"));
    }
}
