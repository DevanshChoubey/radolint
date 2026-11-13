// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Copy.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use dlint_syntax::*;
use crate::combinator::{PResult, Parser};

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
    todo!("port Copy.hs parseCopy")
}

/// Haskell `parseAdd` (Copy.hs).
pub fn parse_add(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Copy.hs parseAdd")
}

/// Haskell `heredocList` (Copy.hs).
pub fn heredoc_list(p: &mut Parser, constr: &dyn Fn(NonEmpty<SourcePath>, TargetPath) -> Instruction<String>) -> PResult<Instruction<String>> {
    todo!("port Copy.hs heredocList")
}

/// Haskell `fileList` (Copy.hs).
pub fn file_list(p: &mut Parser, name: &str, constr: &dyn Fn(NonEmpty<SourcePath>, TargetPath) -> Instruction<String>) -> PResult<Instruction<String>> {
    todo!("port Copy.hs fileList")
}

/// Haskell `unexpectedFlag` (Copy.hs).
pub fn unexpected_flag<T>(p: &mut Parser, name: &str, value: &str) -> PResult<T> {
    todo!("port Copy.hs unexpectedFlag")
}

/// Haskell `copyFlag` (Copy.hs).
pub fn copy_flag(p: &mut Parser) -> PResult<Flag> {
    todo!("port Copy.hs copyFlag")
}

/// Haskell `addFlag` (Copy.hs).
pub fn add_flag(p: &mut Parser) -> PResult<Flag> {
    todo!("port Copy.hs addFlag")
}

/// Haskell `checksum` (Copy.hs).
pub fn checksum(p: &mut Parser) -> PResult<Checksum> {
    todo!("port Copy.hs checksum")
}

/// Haskell `chown` (Copy.hs).
pub fn chown(p: &mut Parser) -> PResult<Chown> {
    todo!("port Copy.hs chown")
}

/// Haskell `chmod` (Copy.hs).
pub fn chmod(p: &mut Parser) -> PResult<Chmod> {
    todo!("port Copy.hs chmod")
}

/// Haskell `link` (Copy.hs).
pub fn link(p: &mut Parser) -> PResult<Link> {
    todo!("port Copy.hs link")
}

/// Haskell `parents` (Copy.hs).
pub fn parents(p: &mut Parser) -> PResult<Parents> {
    todo!("port Copy.hs parents")
}

/// Haskell `keepGitDir` (Copy.hs).
pub fn keep_git_dir(p: &mut Parser) -> PResult<KeepGitDir> {
    todo!("port Copy.hs keepGitDir")
}

/// Haskell `unpack` (Copy.hs).
pub fn unpack(p: &mut Parser) -> PResult<Unpack> {
    todo!("port Copy.hs unpack")
}

/// Haskell `copySource` (Copy.hs).
pub fn copy_source(p: &mut Parser) -> PResult<CopySource> {
    todo!("port Copy.hs copySource")
}

/// Haskell `exclude` (Copy.hs).
pub fn exclude(p: &mut Parser) -> PResult<Exclude> {
    todo!("port Copy.hs exclude")
}

/// Haskell `anyFlag` (Copy.hs).
pub fn any_flag(p: &mut Parser) -> PResult<(String, String)> {
    todo!("port Copy.hs anyFlag")
}

