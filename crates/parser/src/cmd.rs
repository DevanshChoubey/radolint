// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Cmd.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use dlint_syntax::*;
use crate::combinator::{PResult, Parser};


/// Haskell `parseCmd` (Cmd.hs).
pub fn parse_cmd(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Cmd.hs parseCmd")
}

