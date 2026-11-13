// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3001.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{ParsedShell, find_command_names};
use radolint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3001(), onbuild(dl3001()))
}

/// Haskell `dl3001`.
fn dl3001() -> Box<dyn Rule> {
    let code = "DL3001";
    let severity = DLSeverity::DLInfoC;
    let message = "For some bash commands it makes no sense running them in a Docker container like `ssh`, `vim`, `shutdown`, `service`, `ps`, `free`, `top`, `kill`, `mount`, `ifconfig`";

    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => fold_arguments(has_invalid, args),
        _ => true,
    };

    fn has_invalid(args: &ParsedShell) -> bool {
        find_command_names(args)
            .iter()
            .all(|arg| !INVALID_CMDS.contains(&arg.as_str()))
    }
    const INVALID_CMDS: [&str; 9] = [
        "free", "kill", "mount", "ps", "service", "shutdown", "ssh", "top", "vim",
    ];

    simple_rule(code, severity, message, check)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn invalid_cmd() {
        rule_catches("DL3001", "RUN top");
        on_build_rule_catches("DL3001", "RUN top");
    }

    #[test]
    fn install_ssh() {
        rule_catches_not("DL3001", "RUN apt-get install ssh");
        on_build_rule_catches_not("DL3001", "RUN apt-get install ssh");
    }
}
