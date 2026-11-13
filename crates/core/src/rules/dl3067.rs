// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3067.hs.

use crate::rule::{drop_quotes, simple_rule, DLSeverity, Rule};
use crate::shell::ParsedShell;
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3067";
    let severity = DLSeverity::DLWarningC;
    let message = "Do not copy an entire filesystem from another stage";

    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Copy(CopyArgs { source_paths, target_path }, flags)
            if matches!(flags.source_flag, CopySource::CopySource(_)) =>
        {
            !(source_paths.iter().any(is_root_source) && is_root_target(target_path))
        }
        _ => true,
    };

    simple_rule(code, severity, message, check)
}

/// Haskell `isRootSource`.
fn is_root_source(s: &SourcePath) -> bool {
    drop_quotes(&s.un_source_path) == "/"
}

/// Haskell `isRootTarget`.
fn is_root_target(t: &TargetPath) -> bool {
    drop_quotes(&t.un_target_path) == "/"
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn warns_when_copying_the_root_filesystem_from_another_stage_to_root() {
        rule_catches("DL3067", "COPY --from=build / /");
    }

    #[test]
    fn warns_when_copying_quoted_root_paths_from_another_stage_to_root() {
        rule_catches("DL3067", "COPY --from=build \"/\" \"/\"");
    }

    #[test]
    fn does_not_warn_when_copying_root_without_from() {
        rule_catches_not("DL3067", "COPY / /");
    }

    #[test]
    fn does_not_warn_when_copying_a_directory_from_another_stage() {
        rule_catches_not("DL3067", "COPY --from=build /app /");
    }

    #[test]
    fn does_not_warn_when_copying_root_from_another_stage_into_a_subdirectory() {
        rule_catches_not("DL3067", "COPY --from=build / /app");
    }
}
