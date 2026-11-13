// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3010.hs.

use crate::rule::{
    CheckFailure, DLSeverity, Failures, Rule, RuleCode, State, archive_file_format_extensions,
    drop_quotes, empty_state, fold_arguments, modify, very_custom_rule,
};
use crate::shell;
use crate::shell::ParsedShell;
use radolint_syntax::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
enum Acc {
    Acc {
        archives: BTreeSet<(Linenumber, String)>,
        extracted: BTreeSet<(Linenumber, String)>,
    },
    Empty,
}

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3010";
    let severity = DLSeverity::DLInfoC;
    let message = "Use `ADD` for extracting archives into an image";

    let check =
        |line: Linenumber, st: State<Acc>, instr: &Instruction<ParsedShell>| -> State<Acc> {
            match instr {
                Instruction::From(_) => empty_state(Acc::Empty),
                Instruction::Copy(
                    CopyArgs {
                        source_paths,
                        target_path,
                    },
                    flags,
                ) if flags.source_flag == CopySource::NoSource => modify(
                    |a| remember_archives(line, source_paths, target_path, a),
                    st,
                ),
                Instruction::Run(RunArgs(args, _)) => {
                    if let Acc::Acc { archives, .. } = &st.state {
                        let ex = fold_arguments(|s| get_extracted_archives(archives, s), args);
                        modify(|a| mark_extracted(ex, a), st)
                    } else {
                        st
                    }
                }
                _ => st,
            }
        };

    let make_fail = move |line: Linenumber| CheckFailure {
        code: RuleCode::from(code),
        severity,
        message: message.to_string(),
        line,
    };

    let mark_failures = move |st: State<Acc>| -> Failures {
        match st.state {
            Acc::Acc { extracted, .. } => {
                let mut fails = st.failures;
                // Set.map makeFail: CheckFailure's Ord compares only the line, so the set
                // dedups by line and is sorted by line.
                let lines: BTreeSet<Linenumber> = extracted.iter().map(|(l, _)| *l).collect();
                fails.extend(lines.into_iter().map(&make_fail));
                fails
            }
            Acc::Empty => st.failures,
        }
    };

    very_custom_rule(check, empty_state(Acc::Empty), mark_failures)
}

fn extracts_this_archive(archive: &(Linenumber, String), cmd: &shell::Command) -> bool {
    let arguments: Vec<String> = shell::get_args_no_flags(cmd)
        .iter()
        .map(|a| basename(a))
        .collect();
    (is_tar_extract_command(cmd) || is_unzip_command(cmd)) && arguments.contains(&archive.1)
}

fn get_extracted_archives(
    archives: &BTreeSet<(Linenumber, String)>,
    shell: &ParsedShell,
) -> BTreeSet<(Linenumber, String)> {
    let cmds = &shell.present_commands;
    archives
        .iter()
        .filter(|a| cmds.iter().any(|c| extracts_this_archive(a, c)))
        .cloned()
        .collect()
}

fn is_tar_extract_command(cmd: &shell::Command) -> bool {
    let args = shell::get_args(cmd);
    let long_extract_flags = |f: &String| f == "--extract" || f == "--get";
    let short_extract_flags = |f: &String| f.starts_with('-') && f.contains('x');
    cmd.name == "tar"
        && (args.iter().any(long_extract_flags) || args.iter().any(short_extract_flags))
}

fn is_unzip_command(cmd: &shell::Command) -> bool {
    [
        "unzip",
        "gunzip",
        "bunzip2",
        "unlzma",
        "unxz",
        "zgz",
        "uncompress",
        "zcat",
        "gzcat",
    ]
    .contains(&cmd.name.as_str())
}

fn mark_extracted(exarcv: BTreeSet<(Linenumber, String)>, acc: Acc) -> Acc {
    match acc {
        Acc::Empty => Acc::Empty,
        Acc::Acc {
            archives,
            extracted,
        } => {
            let mut e = extracted;
            e.extend(exarcv);
            Acc::Acc {
                archives,
                extracted: e,
            }
        }
    }
}

fn remember_archives(
    line: Linenumber,
    paths: &NonEmpty<SourcePath>,
    target: &TargetPath,
    acc: Acc,
) -> Acc {
    let from_paths = || -> BTreeSet<(Linenumber, String)> {
        let names: BTreeSet<String> = paths_to_vec(paths)
            .iter()
            .map(|p| basename(&p.un_source_path))
            .collect();
        names
            .into_iter()
            .filter(|n| is_archive(n))
            .map(|n| (line, n))
            .collect()
    };
    match acc {
        Acc::Empty => {
            if is_archive(&target.un_target_path) {
                Acc::Acc {
                    archives: BTreeSet::from([(line, basename(&target.un_target_path))]),
                    extracted: BTreeSet::new(),
                }
            } else {
                Acc::Acc {
                    archives: from_paths(),
                    extracted: BTreeSet::new(),
                }
            }
        }
        Acc::Acc {
            archives,
            extracted,
        } => {
            if is_archive(&target.un_target_path) {
                let mut a = archives;
                a.insert((line, basename(&target.un_target_path)));
                Acc::Acc {
                    archives: a,
                    extracted,
                }
            } else {
                let mut a = archives;
                a.extend(from_paths());
                Acc::Acc {
                    archives: a,
                    extracted,
                }
            }
        }
    }
}

fn paths_to_vec(paths: &NonEmpty<SourcePath>) -> Vec<&SourcePath> {
    std::iter::once(&paths.head)
        .chain(paths.tail.iter())
        .collect()
}

fn basename(t: &str) -> String {
    let d = drop_quotes(t);
    let start = d.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    d[start..].to_string()
}

fn is_archive(src: &str) -> bool {
    let d = drop_quotes(src);
    archive_file_format_extensions()
        .iter()
        .any(|e| d.ends_with(e))
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    fn unlines(ls: &[&str]) -> String {
        ls.iter().map(|l| format!("{l}\n")).collect()
    }

    #[test]
    fn catch_copy_archive_then_extract_1() {
        let d = unlines(&[
            "COPY packaged-app.tar /usr/src/app",
            "RUN tar -xf /usr/src/app/packaged-app.tar",
        ]);
        rule_catches("DL3010", &d);
    }
    #[test]
    fn catch_copy_archive_then_extract_2() {
        let d = unlines(&[
            "COPY packaged-app.tar /usr/src/app",
            "WORKDIR /usr/src/app",
            "RUN foo bar && echo something && tar -xf packaged-app.tar",
        ]);
        rule_catches("DL3010", &d);
    }
    #[test]
    fn catch_copy_archive_then_extract_3() {
        let d = unlines(&[
            "COPY foo/bar/packaged-app.tar /foo.tar",
            "RUN tar -xf /foo.tar",
        ]);
        rule_catches("DL3010", &d);
    }
    #[test]
    fn catch_copy_archive_then_extract_windows_paths_1() {
        let d = unlines(&[
            "COPY build\\foo\\bar.tar.gz \"C:\\Program Files\\Foo\"",
            "RUN tar -xf \"C:\\Program Files\\Foo\\bar.tar.gz\"",
        ]);
        rule_catches("DL3010", &d);
    }
    #[test]
    fn catch_copy_archive_then_extract_windows_paths_2() {
        let d = unlines(&[
            "COPY build\\foo\\bar.tar.gz \"C:\\Program Files\\foo.tar.gz\"",
            "RUN tar -xf foo.tar.gz",
        ]);
        rule_catches("DL3010", &d);
    }
    #[test]
    fn ignore_copy_archive_without_extract() {
        let d = unlines(&[
            "COPY packaged-app.tar /usr/src/app",
            "FROM debian:11 as newstage",
        ]);
        rule_catches_not("DL3010", &d);
    }
    #[test]
    fn ignore_non_archive() {
        rule_catches_not("DL3010", "COPY package.json /usr/src/app");
    }
    #[test]
    fn ignore_copy_from_previous_stage() {
        rule_catches_not(
            "DL3010",
            "COPY --from=builder /usr/local/share/some.tar /opt/some.tar",
        );
    }
}
