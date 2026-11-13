// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Instruction.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use dlint_syntax::*;
use crate::combinator::{PResult, Parser};


/// Haskell `parseShell` (Instruction.hs).
pub fn parse_shell(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseShell")
}

/// Haskell `parseStopSignal` (Instruction.hs).
pub fn parse_stop_signal(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseStopSignal")
}

/// Haskell `parseArg` (Instruction.hs).
pub fn parse_arg(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseArg")
}

/// Haskell `parseUser` (Instruction.hs).
pub fn parse_user(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseUser")
}

/// Haskell `parseWorkdir` (Instruction.hs).
pub fn parse_workdir(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseWorkdir")
}

/// Haskell `parseVolume` (Instruction.hs).
pub fn parse_volume(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseVolume")
}

/// Haskell `parseMaintainer` (Instruction.hs).
pub fn parse_maintainer(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseMaintainer")
}

/// Haskell `parseEntryPoint` (Instruction.hs).
pub fn parse_entry_point(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseEntryPoint")
}

/// Haskell `parseOnbuild` (Instruction.hs).
pub fn parse_onbuild(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseOnbuild")
}

/// Haskell `parsePragma` (Instruction.hs).
pub fn parse_pragma(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parsePragma")
}

/// Haskell `parseEscapePragma` (Instruction.hs).
pub fn parse_escape_pragma(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseEscapePragma")
}

/// Haskell `parseSyntaxPragma` (Instruction.hs).
pub fn parse_syntax_pragma(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseSyntaxPragma")
}

/// Haskell `parseComment` (Instruction.hs).
pub fn parse_comment(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseComment")
}

/// Haskell `parseInstruction` (Instruction.hs).
pub fn parse_instruction(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Instruction.hs parseInstruction")
}

