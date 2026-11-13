// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Prelude.hs.
//!
//! The megaparsec re-exports of the Haskell Prelude live in `combinator.rs`
//! (methods on `Parser`). `?esc` is `p.esc`.

#![allow(unused_variables)]
use crate::combinator::{PResult, ParseError, Parser};
use dlint_syntax::*;

/// Haskell `data DockerfileError`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DockerfileError {
    DuplicateFlagError(String),
    NoValueFlagError(String),
    InvalidFlagError(String),
    FileListError(String),
    MissingArgument(Vec<String>),
    DuplicateArgument(String),
    UnexpectedArgument(String, String),
    QuoteError(String, String),
}

impl DockerfileError {
    /// Haskell `showErrorComponent` of `instance ShowErrorComponent DockerfileError`.
    pub fn show_error_component(&self) -> String {
        use DockerfileError::*;
        match self {
            DuplicateFlagError(f) => format!("duplicate flag: {f}"),
            FileListError(f) => {
                format!("unexpected end of line. At least two arguments are required for {f}")
            }
            NoValueFlagError(f) => format!("unexpected flag {f} with no value"),
            InvalidFlagError(f) => format!("invalid flag: {f}"),
            // Haskell `show [Text]`: ["a","b"]
            MissingArgument(f) => format!(
                "missing required argument(s) for mount flag: [{}]",
                f.iter().map(|s| format!("{s:?}")).collect::<Vec<_>>().join(",")
            ),
            DuplicateArgument(f) => format!("duplicate argument for mount flag: {f}"),
            UnexpectedArgument(a, b) => {
                format!("unexpected argument '{a}' for mount of type '{b}'")
            }
            QuoteError(t, s) => format!("unexpected end of {t} quoted string {s} (unmatched quote)"),
        }
    }
}

/// Haskell `type Error = ParseErrorBundle Text DockerfileError`.
pub type Error = ParseError;

/// Haskell `data FoundWhitespace`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoundWhitespace {
    FoundWhitespace,
    MissingWhitespace,
}

impl FoundWhitespace {
    /// Haskell `Semigroup FoundWhitespace` (`<>`): `Found <> _ = Found; _ <> a = a`.
    pub fn combine(self, other: FoundWhitespace) -> FoundWhitespace {
        match self {
            FoundWhitespace::FoundWhitespace => FoundWhitespace::FoundWhitespace,
            _ => other,
        }
    }
    /// Haskell `mempty`.
    pub fn mempty() -> FoundWhitespace {
        FoundWhitespace::MissingWhitespace
    }
}

/// Haskell `customError`.
pub fn custom_error<T>(p: &mut Parser, e: DockerfileError) -> PResult<T> {
    todo!("port Prelude.hs customError")
}
/// Haskell `castToSpace`.
pub fn cast_to_space(w: FoundWhitespace) -> String {
    todo!("port Prelude.hs castToSpace")
}
/// Haskell `eol`.
pub fn eol(p: &mut Parser) -> PResult<()> {
    todo!("port Prelude.hs eol")
}
/// Haskell `reserved`.
pub fn reserved(p: &mut Parser, name: &str) -> PResult<()> {
    todo!("port Prelude.hs reserved")
}
/// Haskell `natural`.
pub fn natural(p: &mut Parser) -> PResult<u128> {
    todo!("port Prelude.hs natural")
}
/// Haskell `fractional`.
pub fn fractional(p: &mut Parser) -> PResult<f32> {
    todo!("port Prelude.hs fractional")
}
/// Haskell `commaSep`.
pub fn comma_sep<T>(p: &mut Parser, item: impl Fn(&mut Parser) -> PResult<T>) -> PResult<Vec<T>> {
    todo!("port Prelude.hs commaSep")
}
/// Haskell `spaceSep1`.
pub fn space_sep1<T>(p: &mut Parser, item: impl Fn(&mut Parser) -> PResult<T>) -> PResult<Vec<T>> {
    todo!("port Prelude.hs spaceSep1")
}
/// Haskell `singleQuotedString`.
pub fn single_quoted_string(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs singleQuotedString")
}
/// Haskell `doubleQuotedString`.
pub fn double_quoted_string(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs doubleQuotedString")
}
/// Haskell `singleQuotedStringEscaped`.
pub fn single_quoted_string_escaped(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs singleQuotedStringEscaped")
}
/// Haskell `doubleQuotedStringEscaped`.
pub fn double_quoted_string_escaped(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs doubleQuotedStringEscaped")
}
/// Haskell `quotedString`.
pub fn quoted_string(p: &mut Parser, c: char) -> PResult<String> {
    todo!("port Prelude.hs quotedString")
}
/// Haskell `quotedStringEscaped`.
pub fn quoted_string_escaped(p: &mut Parser, q: char) -> PResult<String> {
    todo!("port Prelude.hs quotedStringEscaped")
}
/// Haskell `brackets`.
pub fn brackets<T>(p: &mut Parser, inner: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
    todo!("port Prelude.hs brackets")
}
/// Haskell `untilWS`.
pub fn until_ws(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs untilWS")
}
/// Haskell `heredocMarker`.
pub fn heredoc_marker(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs heredocMarker")
}
/// Haskell `heredocRedirect`.
pub fn heredoc_redirect(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs heredocRedirect")
}
/// Haskell `heredocContent`.
pub fn heredoc_content(p: &mut Parser, marker: &str) -> PResult<String> {
    todo!("port Prelude.hs heredocContent")
}
/// Haskell `heredoc`.
pub fn heredoc(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs heredoc")
}
/// Haskell `untilHeredoc`.
pub fn until_heredoc(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs untilHeredoc")
}
/// Haskell `onlySpaces`.
pub fn only_spaces(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs onlySpaces")
}
/// Haskell `onlySpaces1`.
pub fn only_spaces1(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs onlySpaces1")
}
/// Haskell `onlyWhitespaces`.
pub fn only_whitespaces(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs onlyWhitespaces")
}
/// Haskell `escapedLineBreaks`.
pub fn escaped_line_breaks(p: &mut Parser) -> PResult<FoundWhitespace> {
    todo!("port Prelude.hs escapedLineBreaks")
}
/// Haskell `escapedLineBreaks'`.
pub fn escaped_line_breaks_(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs escapedLineBreaks'")
}
/// Haskell `foundWhitespace`.
pub fn found_whitespace(p: &mut Parser) -> PResult<FoundWhitespace> {
    todo!("port Prelude.hs foundWhitespace")
}
/// Haskell `whitespace`.
pub fn whitespace(p: &mut Parser) -> PResult<()> {
    todo!("port Prelude.hs whitespace")
}
/// Haskell `requiredWhitespace`.
pub fn required_whitespace(p: &mut Parser) -> PResult<()> {
    todo!("port Prelude.hs requiredWhitespace")
}
/// Haskell `untilEol`.
pub fn until_eol(p: &mut Parser, name: &str) -> PResult<String> {
    todo!("port Prelude.hs untilEol")
}
/// Haskell `symbol`.
pub fn symbol(p: &mut Parser, name: &str) -> PResult<String> {
    todo!("port Prelude.hs symbol")
}
/// Haskell `caseInsensitiveString`.
pub fn case_insensitive_string(p: &mut Parser, s: &str) -> PResult<String> {
    todo!("port Prelude.hs caseInsensitiveString")
}
/// Haskell `stringWithEscaped`.
pub fn string_with_escaped(
    p: &mut Parser,
    quote_chars: &[char],
    maybe_accept_condition: Option<&dyn Fn(char) -> bool>,
) -> PResult<String> {
    todo!("port Prelude.hs stringWithEscaped")
}
/// Haskell `lexeme`.
pub fn lexeme<T>(p: &mut Parser, inner: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
    todo!("port Prelude.hs lexeme")
}
/// Haskell `lexeme'`.
pub fn lexeme_<T>(p: &mut Parser, inner: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
    todo!("port Prelude.hs lexeme'")
}
/// Haskell `isNl`.
pub fn is_nl(c: char) -> bool {
    todo!("port Prelude.hs isNl")
}
/// Haskell `isSpaceNl` (`?esc` is the first argument).
pub fn is_space_nl(esc: char, c: char) -> bool {
    todo!("port Prelude.hs isSpaceNl")
}
/// Haskell `anyUnless`.
pub fn any_unless(p: &mut Parser, predicate: &dyn Fn(char) -> bool) -> PResult<String> {
    todo!("port Prelude.hs anyUnless")
}
/// Haskell `someUnless`.
pub fn some_unless(p: &mut Parser, name: &str, predicate: &dyn Fn(char) -> bool) -> PResult<String> {
    todo!("port Prelude.hs someUnless")
}
/// Haskell `comment`.
pub fn comment(p: &mut Parser) -> PResult<String> {
    todo!("port Prelude.hs comment")
}

#[allow(dead_code)]
fn _uses(_: Instruction<String>) {}
