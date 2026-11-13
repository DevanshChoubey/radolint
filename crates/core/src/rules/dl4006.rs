// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL4006.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Rule, RuleCode, State, add_fail, custom_rule, empty_state,
    fold_arguments, replace_with,
};
use crate::shell::{self, ParsedShell};
use dlint_syntax::{Instruction, Linenumber, RunArgs};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL4006";
    let severity = DLSeverity::DLWarningC;
    let message = "Set the SHELL option -o pipefail before RUN with a pipe in it. If you are using \
/bin/sh in an alpine image or if your shell is symlinked to busybox then consider \
explicitly setting your SHELL to /bin/ash, or disable this check";

    let check =
        move |line: Linenumber, st: State<bool>, instr: &Instruction<ParsedShell>| match instr {
            // Reset the state each time we find a new FROM
            Instruction::From(_) => replace_with(false, st),
            Instruction::Shell(args) => {
                if fold_arguments(is_non_posix_shell, args) {
                    replace_with(true, st)
                } else {
                    replace_with(fold_arguments(has_pipefail_option, args), st)
                }
            }
            Instruction::Run(RunArgs(args, _)) if !st.state => {
                if fold_arguments(shell::has_pipes, args) {
                    add_fail(
                        CheckFailure {
                            code: RuleCode::from(code),
                            severity,
                            message: message.to_string(),
                            line,
                        },
                        st,
                    )
                } else {
                    st
                }
            }
            _ => st,
        };
    custom_rule(check, empty_state(false))
}

fn is_non_posix_shell(p: &ParsedShell) -> bool {
    shell::non_posix_shells()
        .iter()
        .any(|s| p.original.starts_with(s))
}

fn has_pipefail_option(script: &ParsedShell) -> bool {
    for cmd in &script.present_commands {
        for valid_shell in ["/bin/bash", "/bin/zsh", "/bin/ash", "bash", "zsh", "ash"] {
            if cmd.name == valid_shell && shell::has_flag("o", cmd) {
                for a in &cmd.arguments {
                    if a.arg == "pipefail" {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn warn_on_missing_pipefail() {
        rule_catches(
            "DL4006",
            "FROM scratch\nRUN wget -O - https://some.site | wc -l > /number\n",
        );
    }

    #[test]
    fn dont_warn_on_commands_with_no_pipes() {
        rule_catches_not(
            "DL4006",
            "FROM scratch as build\nRUN wget -O - https://some.site && wc -l file > /number\n",
        );
    }

    #[test]
    fn dont_warn_pipefail() {
        rule_catches_not(
            "DL4006",
            "FROM scratch as build\nSHELL [\"/bin/bash\", \"-eo\", \"pipefail\", \"-c\"]\nRUN wget -O - https://some.site | wc -l file > /number\n",
        );
    }

    #[test]
    fn dont_warn_pipefail_2() {
        rule_catches_not(
            "DL4006",
            "FROM scratch as build\nSHELL [\"/bin/bash\", \"-e\", \"-o\", \"pipefail\", \"-c\"]\nRUN wget -O - https://some.site | wc -l file > /number\n",
        );
    }

    #[test]
    fn dont_warn_pipefail_3() {
        rule_catches_not(
            "DL4006",
            "FROM scratch as build\nSHELL [\"/bin/bash\", \"-o\", \"errexit\", \"-o\", \"pipefail\", \"-c\"]\nRUN wget -O - https://some.site | wc -l file > /number\n",
        );
    }

    #[test]
    fn dont_warn_pipefail_zsh() {
        rule_catches_not(
            "DL4006",
            "FROM scratch as build\nSHELL [\"/bin/zsh\", \"-o\", \"pipefail\", \"-c\"]\nRUN wget -O - https://some.site | wc -l file > /number\n",
        );
    }

    #[test]
    fn dont_warn_on_powershell() {
        rule_catches_not(
            "DL4006",
            "FROM scratch as build\nSHELL [\"pwsh\", \"-c\"]\nRUN Get-Variable PSVersionTable | Select-Object -ExpandProperty Value\n",
        );
    }

    #[test]
    fn warns_when_using_plain_sh() {
        rule_catches(
            "DL4006",
            "FROM scratch as build\nSHELL [\"/bin/sh\", \"-o\", \"pipefail\", \"-c\"]\nRUN wget -O - https://some.site | wc -l file > /number\n",
        );
    }

    #[test]
    fn warn_on_missing_pipefail_in_next_image() {
        rule_catches(
            "DL4006",
            "FROM scratch as build\nSHELL [\"/bin/bash\", \"-o\", \"pipefail\", \"-c\"]\nRUN wget -O - https://some.site | wc -l file > /number\nFROM scratch as build2\nRUN wget -O - https://some.site | wc -l file > /number\n",
        );
    }

    #[test]
    fn warn_on_missing_pipefail_if_next_shell_not_using_it() {
        rule_catches(
            "DL4006",
            "FROM scratch as build\nSHELL [\"/bin/bash\", \"-o\", \"pipefail\", \"-c\"]\nRUN wget -O - https://some.site | wc -l file > /number\nSHELL [\"/bin/sh\", \"-c\"]\nRUN wget -O - https://some.site | wc -l file > /number\n",
        );
    }

    #[test]
    fn ignore_non_posix_shells_pwsh() {
        rule_catches_not(
            "DL4006",
            "FROM mcr.microsoft.com/powershell:ubuntu-16.04\nSHELL [ \"pwsh\", \"-c\" ]\nRUN Get-Variable PSVersionTable | Select-Object -ExpandProperty Value\n",
        );
    }

    #[test]
    fn ignore_non_posix_shells_powershell() {
        rule_catches_not(
            "DL4006",
            "FROM mcr.microsoft.com/powershell:ubuntu-16.04\nSHELL [ \"powershell.exe\" ]\nRUN Get-Variable PSVersionTable | Select-Object -ExpandProperty Value\n",
        );
    }

    #[test]
    fn ignore_non_posix_shells_cmd_exe() {
        rule_catches_not(
            "DL4006",
            "FROM mcr.microsoft.com/powershell:ubuntu-16.04\nSHELL [ \"cmd.exe\", \"/c\" ]\nRUN Get-Variable PSVersionTable | Select-Object -ExpandProperty Value\n",
        );
    }
}
