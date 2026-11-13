// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Pairs.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use dlint_syntax::*;
use crate::combinator::{PResult, Parser};


/// Haskell `unquotedString` (Pairs.hs).
pub fn unquoted_string(p: &mut Parser, accept_condition: &dyn Fn(char) -> bool) -> PResult<String> {
    todo!("port Pairs.hs unquotedString")
}

/// Haskell `singleValue` (Pairs.hs).
pub fn single_value(p: &mut Parser, accept_condition: &dyn Fn(char) -> bool) -> PResult<String> {
    todo!("port Pairs.hs singleValue")
}

/// Haskell `pair` (Pairs.hs).
pub fn pair(p: &mut Parser) -> PResult<(String, String)> {
    todo!("port Pairs.hs pair")
}

/// Haskell `pairs` (Pairs.hs).
pub fn pairs(p: &mut Parser) -> PResult<Pairs> {
    todo!("port Pairs.hs pairs")
}

/// Haskell `parseLabel` (Pairs.hs).
pub fn parse_label(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Pairs.hs parseLabel")
}

/// Haskell `parseEnv` (Pairs.hs).
pub fn parse_env(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Pairs.hs parseEnv")
}

