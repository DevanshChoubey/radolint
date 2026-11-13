// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3019.hs.

use crate::rule::*;
use crate::shell;
use crate::shell::ParsedShell;
use crate::utils;
use radolint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3019(), onbuild(dl3019()))
}

/// Haskell `dl3019`.
fn dl3019() -> Box<dyn Rule> {
    let code = "DL3019";
    let severity = DLSeverity::DLInfoC;
    let message = "Use the `--no-cache` switch to avoid the need to use `--update` and remove `/var/cache/apk/*` when done installing packages";

    let check = |instr: &Instruction<ParsedShell>| -> bool {
        match instr {
            Instruction::Run(RunArgs(args, flags)) => {
                if utils::has_cache_or_tmpfs_mount_with("/var/cache/apk", flags) {
                    true
                } else {
                    fold_arguments(|a| shell::no_commands(forgot_cache_option, a), args)
                }
            }
            _ => true,
        }
    };
    simple_rule(code, severity, message, check)
}

/// Haskell `forgotCacheOption`.
fn forgot_cache_option(cmd: &shell::Command) -> bool {
    shell::cmd_has_args("apk", &["add"], cmd) && !shell::has_flag("no-cache", cmd)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    fn both_catch(line: &str) {
        rule_catches("DL3019", line);
        on_build_rule_catches("DL3019", line);
    }
    fn both_catch_not(line: &str) {
        rule_catches_not("DL3019", line);
        on_build_rule_catches_not("DL3019", line);
    }

    #[test]
    fn warn_apk_add_without_no_cache() {
        both_catch("RUN apk add flex=2.6.4-r1");
    }
    #[test]
    fn dont_warn_apk_add_with_no_cache() {
        both_catch_not("RUN apk add --no-cache flex=2.6.4-r1");
    }
    #[test]
    fn dont_warn_cache_mount() {
        both_catch_not("RUN --mount=type=cache,target=/var/cache/apk apk add -U curl=7.77.0");
    }
    #[test]
    fn dont_warn_tmpfs_mount() {
        both_catch_not("RUN --mount=type=tmpfs,target=/var/cache/apk apk add -U curl=7.77.0");
    }
    #[test]
    fn dont_warn_wrong_dir_mount_and_no_cache() {
        both_catch_not(
            "RUN --mount=type=cache,target=/var/cache/foo apk add --no-cache -U curl=7.77.0",
        );
    }
    #[test]
    fn warn_cache_mount_wrong_dir() {
        both_catch("RUN --mount=type=cache,target=/var/cache/foo apk add -U curl=7.77.0");
    }
    #[test]
    fn warn_tmpfs_mount_wrong_dir() {
        both_catch("RUN --mount=type=tmpfs,target=/var/cache/foo apk add -U curl=7.77.0");
    }
}
