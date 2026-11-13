// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Arguments.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use dlint_syntax::*;
use crate::combinator::{PResult, Parser};


/// Haskell `argumentsExec` (Arguments.hs).
pub fn arguments_exec(p: &mut Parser) -> PResult<Arguments<String>> {
    todo!("port Arguments.hs argumentsExec")
}

/// Haskell `argumentsShell` (Arguments.hs).
pub fn arguments_shell(p: &mut Parser) -> PResult<Arguments<String>> {
    todo!("port Arguments.hs argumentsShell")
}

/// Haskell `argumentsHeredoc` (Arguments.hs).
pub fn arguments_heredoc(p: &mut Parser) -> PResult<Arguments<String>> {
    todo!("port Arguments.hs argumentsHeredoc")
}

/// Haskell `arguments` (Arguments.hs).
pub fn arguments(p: &mut Parser) -> PResult<Arguments<String>> {
    todo!("port Arguments.hs arguments")
}

