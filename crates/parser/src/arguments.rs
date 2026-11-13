// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Arguments.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use dlint_syntax::*;
use crate::combinator::{PResult, Parser};


use crate::prelude::*;

/// Haskell `argumentsExec` (Arguments.hs).
pub fn arguments_exec(p: &mut Parser) -> PResult<Arguments<String>> {
    let args = brackets(p, |s| comma_sep(s, double_quoted_string_escaped))?;
    Ok(Arguments::ArgumentsList(args.join(" ")))
}

/// Haskell `argumentsShell` (Arguments.hs).
pub fn arguments_shell(p: &mut Parser) -> PResult<Arguments<String>> {
    fn to_end(s: &mut Parser) -> PResult<String> {
        until_eol(s, "the shell arguments")
    }
    p.alt(
        |s| s.try_(|s| Ok(Arguments::ArgumentsText(until_heredoc(s)?))),
        |s| Ok(Arguments::ArgumentsText(to_end(s)?)),
    )
}

/// Haskell `argumentsHeredoc` (Arguments.hs).
pub fn arguments_heredoc(p: &mut Parser) -> PResult<Arguments<String>> {
    Ok(Arguments::ArgumentsText(heredoc(p)?))
}

/// Haskell `arguments` (Arguments.hs).
pub fn arguments(p: &mut Parser) -> PResult<Arguments<String>> {
    p.alt(
        |s| s.try_(arguments_heredoc),
        |s| s.alt(|s| s.try_(arguments_exec), |s| s.try_(arguments_shell)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combinator::run_parser;

    #[test]
    fn exec_form() {
        let r = run_parser("[\"a\", \"b\"]", "<string>", '\\', arguments).unwrap();
        assert_eq!(r, Arguments::ArgumentsList("a b".to_string()));
    }

    #[test]
    fn shell_form() {
        let r = run_parser("echo hi", "<string>", '\\', arguments).unwrap();
        assert_eq!(r, Arguments::ArgumentsText("echo hi".to_string()));
    }
}
