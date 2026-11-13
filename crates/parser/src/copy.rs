// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Copy.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use crate::combinator::{PResult, Parser};
use crate::prelude::*;
use radolint_syntax::*;

/// Haskell `data Flag` (Copy.hs, module-private upstream).
#[derive(Debug, Clone)]
pub enum Flag {
    FlagChecksum(Checksum),
    FlagChown(Chown),
    FlagChmod(Chmod),
    FlagLink(Link),
    FlagKeepGitDir(KeepGitDir),
    FlagParents(Parents),
    FlagUnpack(Unpack),
    FlagSource(CopySource),
    FlagExclude(Exclude),
    FlagInvalid((String, String)),
}

/// Haskell `parseCopy` (Copy.hs).
pub fn parse_copy(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "COPY")?;
    let flags = p.sep_end_by(copy_flag, required_whitespace)?;
    let chown_flags: Vec<Chown> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagChown(c) = f {
                Some(c.clone())
            } else {
                None
            }
        })
        .collect();
    let chmod_flags: Vec<Chmod> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagChmod(c) = f {
                Some(c.clone())
            } else {
                None
            }
        })
        .collect();
    let link_flags: Vec<Link> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagLink(l) = f {
                Some(*l)
            } else {
                None
            }
        })
        .collect();
    let parents_flags: Vec<Parents> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagParents(x) = f {
                Some(*x)
            } else {
                None
            }
        })
        .collect();
    let source_flags: Vec<CopySource> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagSource(x) = f {
                Some(x.clone())
            } else {
                None
            }
        })
        .collect();
    let exclude_flags: Vec<Exclude> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagExclude(x) = f {
                Some(x.clone())
            } else {
                None
            }
        })
        .collect();
    let invalid: Vec<(String, String)> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagInvalid(x) = f {
                Some(x.clone())
            } else {
                None
            }
        })
        .collect();
    // Let's do some validation on the flags
    if let Some((k, v)) = invalid.first() {
        unexpected_flag(p, k, v)
    } else if chown_flags.len() >= 2 {
        custom_error(
            p,
            DockerfileError::DuplicateFlagError("--chown".to_string()),
        )
    } else if chmod_flags.len() >= 2 {
        custom_error(
            p,
            DockerfileError::DuplicateFlagError("--chmod".to_string()),
        )
    } else if link_flags.len() >= 2 {
        custom_error(p, DockerfileError::DuplicateFlagError("--link".to_string()))
    } else if parents_flags.len() >= 2 {
        custom_error(
            p,
            DockerfileError::DuplicateFlagError("--parents".to_string()),
        )
    } else if source_flags.len() >= 2 {
        custom_error(p, DockerfileError::DuplicateFlagError("--from".to_string()))
    } else {
        let cho = chown_flags.first().cloned().unwrap_or(Chown::NoChown);
        let chm = chmod_flags.first().cloned().unwrap_or(Chmod::NoChmod);
        let lnk = link_flags.first().copied().unwrap_or(Link::NoLink);
        let par = parents_flags.first().copied().unwrap_or(Parents::NoParents);
        let fr = source_flags
            .first()
            .cloned()
            .unwrap_or(CopySource::NoSource);
        let constr = |src: NonEmpty<SourcePath>, dest: TargetPath| {
            Instruction::Copy(
                CopyArgs {
                    source_paths: src,
                    target_path: dest,
                },
                CopyFlags {
                    chown_flag: cho.clone(),
                    chmod_flag: chm.clone(),
                    link_flag: lnk,
                    parents_flag: par,
                    source_flag: fr.clone(),
                    exclude_flags: exclude_flags.clone(),
                },
            )
        };
        p.alt(
            |p| p.try_(|p| heredoc_list(p, &constr)),
            |p| file_list(p, "COPY", &constr),
        )
    }
}

/// Haskell `parseAdd` (Copy.hs).
pub fn parse_add(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "ADD")?;
    let flags = p.sep_end_by(add_flag, required_whitespace)?;
    let checksum_flags: Vec<Checksum> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagChecksum(c) = f {
                Some(c.clone())
            } else {
                None
            }
        })
        .collect();
    let chown_flags: Vec<Chown> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagChown(c) = f {
                Some(c.clone())
            } else {
                None
            }
        })
        .collect();
    let chmod_flags: Vec<Chmod> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagChmod(c) = f {
                Some(c.clone())
            } else {
                None
            }
        })
        .collect();
    let link_flags: Vec<Link> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagLink(l) = f {
                Some(*l)
            } else {
                None
            }
        })
        .collect();
    let keep_git_dir_flags: Vec<KeepGitDir> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagKeepGitDir(k) = f {
                Some(*k)
            } else {
                None
            }
        })
        .collect();
    let unpack_flags: Vec<Unpack> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagUnpack(u) = f {
                Some(*u)
            } else {
                None
            }
        })
        .collect();
    let exclude_flags: Vec<Exclude> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagExclude(x) = f {
                Some(x.clone())
            } else {
                None
            }
        })
        .collect();
    let invalid_flags: Vec<(String, String)> = flags
        .iter()
        .filter_map(|f| {
            if let Flag::FlagInvalid(x) = f {
                Some(x.clone())
            } else {
                None
            }
        })
        .collect();
    p.label(
        "only the --checksum, --chown, --chmod, --link, --exclude, --keep-git-dir, --unpack flags or the src and dest paths",
        |p| p.not_followed_by(|p| p.string("--")),
    )?;
    // Odd upstream pairing kept as-is (not a divergence): the duplicate check for
    // link flags reports "--chmod" and the one for chmod flags reports "--link".
    if let Some((k, v)) = invalid_flags.first() {
        unexpected_flag(p, k, v)
    } else if checksum_flags.len() >= 2 {
        custom_error(
            p,
            DockerfileError::DuplicateFlagError("--checksum".to_string()),
        )
    } else if chown_flags.len() >= 2 {
        custom_error(
            p,
            DockerfileError::DuplicateFlagError("--chown".to_string()),
        )
    } else if link_flags.len() >= 2 {
        custom_error(
            p,
            DockerfileError::DuplicateFlagError("--chmod".to_string()),
        )
    } else if chmod_flags.len() >= 2 {
        custom_error(p, DockerfileError::DuplicateFlagError("--link".to_string()))
    } else if keep_git_dir_flags.len() >= 2 {
        custom_error(
            p,
            DockerfileError::DuplicateFlagError("--keep-git-dir".to_string()),
        )
    } else if unpack_flags.len() >= 2 {
        custom_error(
            p,
            DockerfileError::DuplicateFlagError("--unpack".to_string()),
        )
    } else {
        let chk = checksum_flags
            .first()
            .cloned()
            .unwrap_or(Checksum::NoChecksum);
        let cho = chown_flags.first().cloned().unwrap_or(Chown::NoChown);
        let chm = chmod_flags.first().cloned().unwrap_or(Chmod::NoChmod);
        let lnk = link_flags.first().copied().unwrap_or(Link::NoLink);
        let kgd = keep_git_dir_flags
            .first()
            .copied()
            .unwrap_or(KeepGitDir::NoKeepGitDir);
        let unp = unpack_flags.first().copied().unwrap_or(Unpack::NoUnpack);
        file_list(p, "ADD", &|src, dest| {
            Instruction::Add(
                AddArgs {
                    source_paths: src,
                    target_path: dest,
                },
                AddFlags {
                    checksum_flag: chk.clone(),
                    chown_flag: cho.clone(),
                    chmod_flag: chm.clone(),
                    link_flag: lnk,
                    keep_git_dir_flag: kgd,
                    unpack_flag: unp,
                    exclude_flags: exclude_flags.clone(),
                },
            )
        })
    }
}

/// Haskell `heredocList` (Copy.hs).
pub fn heredoc_list(
    p: &mut Parser,
    constr: &dyn Fn(NonEmpty<SourcePath>, TargetPath) -> Instruction<String>,
) -> PResult<Instruction<String>> {
    let markers = p.label("a list of heredoc markers", |p| {
        p.try_(|p| space_sep1(p, heredoc_marker))
    })?;
    let target = until_eol(p, "target path")?;
    match markers.last() {
        None => {
            return custom_error(
                p,
                DockerfileError::FileListError("empty list of heredoc markers".to_string()),
            );
        }
        Some(m) => {
            heredoc_content(p, m)?;
        }
    }
    let srcs: Vec<SourcePath> = markers.into_iter().map(SourcePath::from).collect();
    // `fromList markers` is total here: the empty case failed above.
    Ok(constr(
        NonEmpty::from_vec(srcs).unwrap(),
        TargetPath::from(target),
    ))
}

/// Haskell `fileList` (Copy.hs).
pub fn file_list(
    p: &mut Parser,
    name: &str,
    constr: &dyn Fn(NonEmpty<SourcePath>, TargetPath) -> Instruction<String>,
) -> PResult<Instruction<String>> {
    fn space_separated(p: &mut Parser) -> PResult<Vec<String>> {
        p.sep_end_by1(
            |p| some_unless(p, "a file", &|c| c == ' '),
            |p| {
                p.label("at least another file path", |p| {
                    p.try_(required_whitespace)
                })
            },
        )
    }
    fn string_list(p: &mut Parser) -> PResult<Vec<String>> {
        brackets(p, |p| comma_sep(p, double_quoted_string))
    }
    let paths = p.alt(
        |p| {
            p.label("an array of strings [\"src_file\", \"dest_file\"]", |p| {
                p.try_(string_list)
            })
        },
        |p| {
            p.label("a space separated list of file paths", |p| {
                p.try_(space_separated)
            })
        },
    )?;
    if paths.len() == 1 {
        custom_error(p, DockerfileError::FileListError(name.to_string()))
    } else if paths.is_empty() {
        // DIVERGENCE: Haskell `init []`/`last []` throws (e.g. `COPY []`); we report a FileListError instead of panicking.
        custom_error(p, DockerfileError::FileListError(name.to_string()))
    } else {
        let (last, init) = paths.split_last().unwrap();
        let srcs: Vec<SourcePath> = init.iter().map(|s| SourcePath::from(s.as_str())).collect();
        Ok(constr(
            NonEmpty::from_vec(srcs).unwrap(),
            TargetPath::from(last.as_str()),
        ))
    }
}

/// Haskell `unexpectedFlag` (Copy.hs).
pub fn unexpected_flag<T>(p: &mut Parser, name: &str, value: &str) -> PResult<T> {
    if value.is_empty() {
        p.custom_failure(DockerfileError::NoValueFlagError(name.to_string()))
    } else {
        p.custom_failure(DockerfileError::InvalidFlagError(name.to_string()))
    }
}

/// Haskell `copyFlag` (Copy.hs).
pub fn copy_flag(p: &mut Parser) -> PResult<Flag> {
    p.alt(
        |p| {
            p.label("only one --from", |p| {
                Ok(Flag::FlagSource(p.try_(copy_source)?))
            })
        },
        |p| {
            p.alt(
                |p| p.label("--chown", |p| Ok(Flag::FlagChown(p.try_(chown)?))),
                |p| {
                    p.alt(
                        |p| p.label("--chmod", |p| Ok(Flag::FlagChmod(p.try_(chmod)?))),
                        |p| {
                            p.alt(
                                |p| p.label("--link", |p| Ok(Flag::FlagLink(p.try_(link)?))),
                                |p| {
                                    p.alt(
                                        |p| {
                                            p.label("--parents", |p| {
                                                Ok(Flag::FlagParents(p.try_(parents)?))
                                            })
                                        },
                                        |p| {
                                            p.alt(
                                                |p| {
                                                    p.label("--exclude", |p| {
                                                        Ok(Flag::FlagExclude(p.try_(exclude)?))
                                                    })
                                                },
                                                |p| {
                                                    p.label("other flag", |p| {
                                                        Ok(Flag::FlagInvalid(p.try_(any_flag)?))
                                                    })
                                                },
                                            )
                                        },
                                    )
                                },
                            )
                        },
                    )
                },
            )
        },
    )
}

/// Haskell `addFlag` (Copy.hs).
pub fn add_flag(p: &mut Parser) -> PResult<Flag> {
    p.alt(
        |p| p.label("--checksum", |p| Ok(Flag::FlagChecksum(p.try_(checksum)?))),
        |p| {
            p.alt(
                |p| p.label("--chown", |p| Ok(Flag::FlagChown(p.try_(chown)?))),
                |p| {
                    p.alt(
                        |p| p.label("--chmod", |p| Ok(Flag::FlagChmod(p.try_(chmod)?))),
                        |p| {
                            p.alt(
                                |p| p.label("--link", |p| Ok(Flag::FlagLink(p.try_(link)?))),
                                |p| {
                                    p.alt(
                                        |p| {
                                            p.label("--keep-git-dir", |p| {
                                                Ok(Flag::FlagKeepGitDir(p.try_(keep_git_dir)?))
                                            })
                                        },
                                        |p| {
                                            p.alt(
                                                |p| {
                                                    p.label("--unpack", |p| {
                                                        Ok(Flag::FlagUnpack(p.try_(unpack)?))
                                                    })
                                                },
                                                |p| {
                                                    p.alt(
                                                        |p| {
                                                            p.label("--exclude", |p| {
                                                                Ok(Flag::FlagExclude(
                                                                    p.try_(exclude)?,
                                                                ))
                                                            })
                                                        },
                                                        |p| {
                                                            p.label("other flag", |p| {
                                                                Ok(Flag::FlagInvalid(
                                                                    p.try_(any_flag)?,
                                                                ))
                                                            })
                                                        },
                                                    )
                                                },
                                            )
                                        },
                                    )
                                },
                            )
                        },
                    )
                },
            )
        },
    )
}

/// Haskell `checksum` (Copy.hs).
pub fn checksum(p: &mut Parser) -> PResult<Checksum> {
    p.string("--checksum=")?;
    let chk = some_unless(p, "the remote file checksum", &|c| c == ' ')?;
    Ok(Checksum::Checksum(chk))
}

/// Haskell `chown` (Copy.hs).
pub fn chown(p: &mut Parser) -> PResult<Chown> {
    p.string("--chown=")?;
    let cho = some_unless(p, "the user and group for chown", &|c| c == ' ')?;
    Ok(Chown::Chown(cho))
}

/// Haskell `chmod` (Copy.hs).
pub fn chmod(p: &mut Parser) -> PResult<Chmod> {
    p.string("--chmod=")?;
    let chm = some_unless(p, "the mode for chmod", &|c| c == ' ')?;
    Ok(Chmod::Chmod(chm))
}

/// `string "true" <|> string "false"`
fn true_or_false(p: &mut Parser) -> PResult<String> {
    p.alt(|p| p.string("true"), |p| p.string("false"))
}

/// Haskell `link` (Copy.hs).
pub fn link(p: &mut Parser) -> PResult<Link> {
    fn link_explicit(p: &mut Parser) -> PResult<Link> {
        p.string("--link=")?;
        let val = true_or_false(p)?;
        Ok(Link::Link(val == "true"))
    }
    fn link_implicit(p: &mut Parser) -> PResult<Link> {
        p.string("--link")?;
        Ok(Link::Link(true))
    }
    p.alt(
        |p| p.label("explicit --link", |p| p.try_(link_explicit)),
        |p| p.label("implicit --link", |p| p.try_(link_implicit)),
    )
}

/// Haskell `parents` (Copy.hs).
pub fn parents(p: &mut Parser) -> PResult<Parents> {
    fn parents_explicit(p: &mut Parser) -> PResult<Parents> {
        p.string("--parents=")?;
        let val = true_or_false(p)?;
        Ok(Parents::Parents(val == "true"))
    }
    fn parents_implicit(p: &mut Parser) -> PResult<Parents> {
        p.string("--parents")?;
        Ok(Parents::Parents(true))
    }
    p.alt(
        |p| p.label("explicit --parents", |p| p.try_(parents_explicit)),
        |p| p.label("implicit --parents", |p| p.try_(parents_implicit)),
    )
}

/// Haskell `keepGitDir` (Copy.hs).
pub fn keep_git_dir(p: &mut Parser) -> PResult<KeepGitDir> {
    fn keep_git_dir_explicit(p: &mut Parser) -> PResult<KeepGitDir> {
        p.string("--keep-git-dir=")?;
        let val = true_or_false(p)?;
        Ok(KeepGitDir::KeepGitDir(val == "true"))
    }
    fn keep_git_dir_implicit(p: &mut Parser) -> PResult<KeepGitDir> {
        p.string("--keep-git-dir")?;
        Ok(KeepGitDir::KeepGitDir(true))
    }
    p.alt(
        |p| p.label("explicit --keep-git-dir", |p| p.try_(keep_git_dir_explicit)),
        |p| p.label("implicit --keep-git-dir", |p| p.try_(keep_git_dir_implicit)),
    )
}

/// Haskell `unpack` (Copy.hs).
pub fn unpack(p: &mut Parser) -> PResult<Unpack> {
    fn unpack_explicit(p: &mut Parser) -> PResult<Unpack> {
        p.string("--unpack=")?;
        let val = true_or_false(p)?;
        Ok(Unpack::Unpack(val == "true"))
    }
    fn unpack_implicit(p: &mut Parser) -> PResult<Unpack> {
        p.string("--unpack")?;
        Ok(Unpack::Unpack(true))
    }
    p.alt(
        |p| p.label("explicit --unpack", |p| p.try_(unpack_explicit)),
        |p| p.label("implicit --unpack", |p| p.try_(unpack_implicit)),
    )
}

/// Haskell `copySource` (Copy.hs).
pub fn copy_source(p: &mut Parser) -> PResult<CopySource> {
    p.string("--from=")?;
    let src = some_unless(p, "the copy source path", &is_nl)?;
    Ok(CopySource::CopySource(src))
}

/// Haskell `exclude` (Copy.hs).
pub fn exclude(p: &mut Parser) -> PResult<Exclude> {
    p.string("--exclude=")?;
    let exc = some_unless(p, "the exclude pattern", &|c| c == ' ')?;
    Ok(Exclude::from(exc))
}

/// Haskell `anyFlag` (Copy.hs).
pub fn any_flag(p: &mut Parser) -> PResult<(String, String)> {
    p.string("--")?;
    let name = some_unless(p, "the flag value", &|c| c == '=')?;
    p.char('=')?;
    let val = any_unless(p, &|c| c == ' ')?;
    Ok((format!("--{name}"), val))
}

#[cfg(test)]
mod tests {
    //! Port of language-docker ParseCopySpec.hs and ParseAddSpec.hs (`assertAst`).
    use super::*;
    use crate::parser::parse_text;

    fn assert_ast(lines: &[&str], expected: Vec<Instruction<String>>) {
        let mut file = lines.join("\n");
        file.push('\n'); // Text.unlines
        match parse_text(&file) {
            Err(e) => panic!("{}", e.bundle_pretty()),
            Ok(d) => {
                let got: Vec<Instruction<String>> = d.into_iter().map(|i| i.instruction).collect();
                assert_eq!(expected, got, "ASTs are not equal");
            }
        }
    }

    fn sps(xs: &[&str]) -> NonEmpty<SourcePath> {
        NonEmpty::from_vec(xs.iter().map(|x| SourcePath::from(*x)).collect()).unwrap()
    }

    fn cp(src: &[&str], dst: &str, flags: CopyFlags) -> Instruction<String> {
        Instruction::Copy(
            CopyArgs {
                source_paths: sps(src),
                target_path: TargetPath::from(dst),
            },
            flags,
        )
    }

    fn ad(src: &[&str], dst: &str, flags: AddFlags) -> Instruction<String> {
        Instruction::Add(
            AddArgs {
                source_paths: sps(src),
                target_path: TargetPath::from(dst),
            },
            flags,
        )
    }

    #[test]
    fn simple_copy() {
        assert_ast(
            &["COPY . /app", "COPY baz /some/long/path"],
            vec![
                cp(
                    &["."],
                    "/app",
                    CopyFlags {
                        ..Default::default()
                    },
                ),
                cp(
                    &["baz"],
                    "/some/long/path",
                    CopyFlags {
                        ..Default::default()
                    },
                ),
            ],
        );
    }

    #[test]
    fn multifiles_copy() {
        assert_ast(
            &["COPY foo bar baz /app"],
            vec![cp(
                &["foo", "bar", "baz"],
                "/app",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn list_of_quoted_files() {
        assert_ast(
            &["COPY [\"foo\", \"bar\", \"baz\", \"/app\"]"],
            vec![cp(
                &["foo", "bar", "baz"],
                "/app",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn supports_windows_paths() {
        assert_ast(
            &["COPY C:\\\\go C:\\\\go"],
            vec![cp(
                &["C:\\\\go"],
                "C:\\\\go",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn does_not_get_confused_with_trailing_whitespace() {
        assert_ast(
            &["COPY a b  "],
            vec![cp(
                &["a"],
                "b",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_chown_flag() {
        assert_ast(
            &["COPY --chown=user:group foo bar"],
            vec![cp(
                &["foo"],
                "bar",
                CopyFlags {
                    chown_flag: Chown::Chown("user:group".into()),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_chmod_flag() {
        assert_ast(
            &["COPY --chmod=777 foo bar"],
            vec![cp(
                &["foo"],
                "bar",
                CopyFlags {
                    chmod_flag: Chmod::Chmod("777".into()),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_link_flag() {
        assert_ast(
            &["COPY --link source /target"],
            vec![cp(
                &["source"],
                "/target",
                CopyFlags {
                    link_flag: Link::Link(true),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_link_flag_explicit_true() {
        assert_ast(
            &["COPY --link=true source /target"],
            vec![cp(
                &["source"],
                "/target",
                CopyFlags {
                    link_flag: Link::Link(true),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_link_flag_explicit_false() {
        assert_ast(
            &["COPY --link=false source /target"],
            vec![cp(
                &["source"],
                "/target",
                CopyFlags {
                    link_flag: Link::Link(false),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_parents_flag() {
        assert_ast(
            &["COPY --parents source /target"],
            vec![cp(
                &["source"],
                "/target",
                CopyFlags {
                    parents_flag: Parents::Parents(true),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_parents_flag_explicit_true() {
        assert_ast(
            &["COPY --parents=true source /target"],
            vec![cp(
                &["source"],
                "/target",
                CopyFlags {
                    parents_flag: Parents::Parents(true),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_parents_flag_explicit_false() {
        assert_ast(
            &["COPY --parents=false source /target"],
            vec![cp(
                &["source"],
                "/target",
                CopyFlags {
                    parents_flag: Parents::Parents(false),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_from_flag() {
        assert_ast(
            &["COPY --from=node foo bar"],
            vec![cp(
                &["foo"],
                "bar",
                CopyFlags {
                    source_flag: CopySource::CopySource("node".into()),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_all_flags() {
        assert_ast(
            &["COPY --from=node --chmod=751 --link --chown=user:group --parents foo bar"],
            vec![cp(
                &["foo"],
                "bar",
                CopyFlags {
                    chown_flag: Chown::Chown("user:group".into()),
                    chmod_flag: Chmod::Chmod("751".into()),
                    link_flag: Link::Link(true),
                    parents_flag: Parents::Parents(true),
                    source_flag: CopySource::CopySource("node".into()),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_all_flags_in_different_order() {
        assert_ast(
            &["COPY --link --parents --chown=user:group --from=node --chmod=644 foo bar"],
            vec![cp(
                &["foo"],
                "bar",
                CopyFlags {
                    chown_flag: Chown::Chown("user:group".into()),
                    chmod_flag: Chmod::Chmod("644".into()),
                    link_flag: Link::Link(true),
                    parents_flag: Parents::Parents(true),
                    source_flag: CopySource::CopySource("node".into()),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_exclude_flag() {
        assert_ast(
            &["COPY --exclude=*.tmp foo bar"],
            vec![cp(
                &["foo"],
                "bar",
                CopyFlags {
                    exclude_flags: vec![Exclude::from("*.tmp")],
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_multiple_exclude_flags() {
        assert_ast(
            &["COPY --exclude=*.tmp --exclude=*.log foo bar"],
            vec![cp(
                &["foo"],
                "bar",
                CopyFlags {
                    exclude_flags: vec![Exclude::from("*.tmp"), Exclude::from("*.log")],
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn with_exclude_and_other_flags() {
        assert_ast(
            &["COPY --chown=root:root --exclude=*.tmp foo bar"],
            vec![cp(
                &["foo"],
                "bar",
                CopyFlags {
                    chown_flag: Chown::Chown("root:root".into()),
                    exclude_flags: vec![Exclude::from("*.tmp")],
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn empty_heredoc() {
        assert_ast(
            &["COPY <<EOF /target", "EOF"],
            vec![cp(
                &["EOF"],
                "/target",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn foo_heredoc() {
        assert_ast(
            &["COPY <<FOO /target", "foo", "FOO"],
            vec![cp(
                &["FOO"],
                "/target",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn foo_heredoc_lowercase_extensions() {
        assert_ast(
            &["COPY <<foo.txt /target", "foo content", "line 2", "foo.txt"],
            vec![cp(
                &["foo.txt"],
                "/target",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn foo_heredoc_single_quoted_marker() {
        assert_ast(
            &["COPY <<'FOO' /target", "foo", "FOO"],
            vec![cp(
                &["FOO"],
                "/target",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn foo_heredoc_double_quoted_marker() {
        assert_ast(
            &["COPY <<\"FOO\" /target", "foo", "FOO"],
            vec![cp(
                &["FOO"],
                "/target",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn foo_heredoc_dash() {
        assert_ast(
            &["COPY <<-FOO /target", "foo", "FOO"],
            vec![cp(
                &["FOO"],
                "/target",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn foo_heredoc_single_quoted_marker_dash() {
        assert_ast(
            &["COPY <<-'FOO' /target", "foo", "FOO"],
            vec![cp(
                &["FOO"],
                "/target",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn foo_heredoc_double_quoted_marker_dash() {
        assert_ast(
            &["COPY <<-\"FOO\" /target", "foo", "FOO"],
            vec![cp(
                &["FOO"],
                "/target",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn multiple_heredocs() {
        assert_ast(
            &["COPY <<FOO <<BAR /target", "foo", "FOO", "bar", "BAR"],
            vec![cp(
                &["FOO", "BAR"],
                "/target",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn multiple_heredocs_mixed() {
        assert_ast(
            &[
                "COPY <<FOO <<-\"BAR\" <<'FIZZ' <<-BUZZ /target",
                "foo",
                "FOO",
                "bar",
                "BAR",
                "fizz",
                "FIZZ",
                "buss",
                "BUZZ",
            ],
            vec![cp(
                &["FOO", "BAR", "FIZZ", "BUZZ"],
                "/target",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn foo_heredoc_with_line_continuations() {
        assert_ast(
            &["COPY <<FOO /target", "foo \\", "bar", "FOO"],
            vec![cp(
                &["FOO"],
                "/target",
                CopyFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_simple_add() {
        assert_ast(
            &["ADD . /app", "ADD http://foo.bar/baz ."],
            vec![
                ad(
                    &["."],
                    "/app",
                    AddFlags {
                        ..Default::default()
                    },
                ),
                ad(
                    &["http://foo.bar/baz"],
                    ".",
                    AddFlags {
                        ..Default::default()
                    },
                ),
            ],
        );
    }

    #[test]
    fn add_multifiles_add() {
        assert_ast(
            &["ADD foo bar baz /app"],
            vec![ad(
                &["foo", "bar", "baz"],
                "/app",
                AddFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_list_of_quoted_files() {
        assert_ast(
            &["ADD [\"foo\", \"bar\", \"baz\", \"/app\"]"],
            vec![ad(
                &["foo", "bar", "baz"],
                "/app",
                AddFlags {
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_checksum_flag() {
        assert_ast(
            &["ADD --checksum=sha256:24454f830cdd http://www.example.com/foo bar"],
            vec![ad(
                &["http://www.example.com/foo"],
                "bar",
                AddFlags {
                    checksum_flag: Checksum::Checksum("sha256:24454f830cdd".into()),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_chown_flag() {
        assert_ast(
            &["ADD --chown=root:root foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    chown_flag: Chown::Chown("root:root".into()),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_chmod_flag() {
        assert_ast(
            &["ADD --chmod=640 foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    chmod_flag: Chmod::Chmod("640".into()),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_link_flag() {
        assert_ast(
            &["ADD --link foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    link_flag: Link::Link(true),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_link_flag_explicit_true() {
        assert_ast(
            &["ADD --link=true foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    link_flag: Link::Link(true),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_link_flag_explicit_false() {
        assert_ast(
            &["ADD --link=false foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    link_flag: Link::Link(false),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_keep_git_dir_flag() {
        assert_ast(
            &["ADD --keep-git-dir foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    keep_git_dir_flag: KeepGitDir::KeepGitDir(true),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_keep_git_dir_flag_explicit_true() {
        assert_ast(
            &["ADD --keep-git-dir=true foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    keep_git_dir_flag: KeepGitDir::KeepGitDir(true),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_keep_git_dir_flag_explicit_false() {
        assert_ast(
            &["ADD --keep-git-dir=false foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    keep_git_dir_flag: KeepGitDir::KeepGitDir(false),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_chown_and_chmod_flag() {
        assert_ast(
            &["ADD --chown=root:root --chmod=640 foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    chown_flag: Chown::Chown("root:root".into()),
                    chmod_flag: Chmod::Chmod("640".into()),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_chown_and_chmod_flag_other_order() {
        assert_ast(
            &["ADD --chmod=640 --chown=root:root foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    chown_flag: Chown::Chown("root:root".into()),
                    chmod_flag: Chmod::Chmod("640".into()),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_all_flags() {
        assert_ast(
            &[
                "ADD --chmod=640 --chown=root:root --checksum=sha256:24454f830cdd --link --keep-git-dir --unpack foo bar",
            ],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    checksum_flag: Checksum::Checksum("sha256:24454f830cdd".into()),
                    chown_flag: Chown::Chown("root:root".into()),
                    chmod_flag: Chmod::Chmod("640".into()),
                    link_flag: Link::Link(true),
                    keep_git_dir_flag: KeepGitDir::KeepGitDir(true),
                    unpack_flag: Unpack::Unpack(true),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_list_of_quoted_files_and_chown() {
        assert_ast(
            &["ADD --chown=user:group [\"foo\", \"bar\", \"baz\", \"/app\"]"],
            vec![ad(
                &["foo", "bar", "baz"],
                "/app",
                AddFlags {
                    chown_flag: Chown::Chown("user:group".into()),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_unpack_flag() {
        assert_ast(
            &["ADD --unpack http://www.example.com/archive.tar.gz /download"],
            vec![ad(
                &["http://www.example.com/archive.tar.gz"],
                "/download",
                AddFlags {
                    unpack_flag: Unpack::Unpack(true),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_unpack_flag_explicit_true() {
        assert_ast(
            &["ADD --unpack=true http://www.example.com/archive.tar.gz /download"],
            vec![ad(
                &["http://www.example.com/archive.tar.gz"],
                "/download",
                AddFlags {
                    unpack_flag: Unpack::Unpack(true),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_unpack_flag_explicit_false() {
        assert_ast(
            &["ADD --unpack=false my-archive.tar.gz ."],
            vec![ad(
                &["my-archive.tar.gz"],
                ".",
                AddFlags {
                    unpack_flag: Unpack::Unpack(false),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_exclude_flag() {
        assert_ast(
            &["ADD --exclude=*.tmp foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    exclude_flags: vec![Exclude::from("*.tmp")],
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_multiple_exclude_flags() {
        assert_ast(
            &["ADD --exclude=*.tmp --exclude=*.log foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    exclude_flags: vec![Exclude::from("*.tmp"), Exclude::from("*.log")],
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn add_with_exclude_and_other_flags() {
        assert_ast(
            &["ADD --chown=root:root --exclude=*.tmp foo bar"],
            vec![ad(
                &["foo"],
                "bar",
                AddFlags {
                    chown_flag: Chown::Chown("root:root".into()),
                    exclude_flags: vec![Exclude::from("*.tmp")],
                    ..Default::default()
                },
            )],
        );
    }
}
