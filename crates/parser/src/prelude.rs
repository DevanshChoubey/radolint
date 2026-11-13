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
    p.custom_failure(e)
}
/// Haskell `castToSpace`.
pub fn cast_to_space(w: FoundWhitespace) -> String {
    match w {
        FoundWhitespace::FoundWhitespace => " ".to_string(),
        FoundWhitespace::MissingWhitespace => String::new(),
    }
}
/// Haskell `eol`.
pub fn eol(p: &mut Parser) -> PResult<()> {
    p.label("end of line", |p| {
        p.some(|s| {
            s.choice(&[
                &|s| only_spaces1(s).map(|_| ()),
                &|s| s.take_while1_p(None, |c| c == '\n').map(|_| ()),
                &|s| escaped_line_breaks(s).map(|_| ()),
            ])
        })
        .map(|_| ())
    })
}
/// Haskell `reserved`.
pub fn reserved(p: &mut Parser, name: &str) -> PResult<()> {
    p.label(name, |s| lexeme(s, |s| case_insensitive_string(s, name))).map(|_| ())
}
/// Haskell `natural`.
pub fn natural(p: &mut Parser) -> PResult<u128> {
    p.label("positive number", |s| s.decimal())
}
/// Haskell `fractional`.
pub fn fractional(p: &mut Parser) -> PResult<f32> {
    p.label("fractional number", |s| s.float())
}
/// Haskell `commaSep`.
pub fn comma_sep<T>(p: &mut Parser, item: impl Fn(&mut Parser) -> PResult<T>) -> PResult<Vec<T>> {
    p.sep_by(
        |s| {
            let x = item(s)?;
            whitespace(s)?;
            Ok(x)
        },
        |s| symbol(s, ","),
    )
}
/// Haskell `spaceSep1`.
pub fn space_sep1<T>(p: &mut Parser, item: impl Fn(&mut Parser) -> PResult<T>) -> PResult<Vec<T>> {
    p.sep_end_by1(item, only_spaces)
}
/// Haskell `singleQuotedString`.
pub fn single_quoted_string(p: &mut Parser) -> PResult<String> {
    quoted_string(p, '\'')
}
/// Haskell `doubleQuotedString`.
pub fn double_quoted_string(p: &mut Parser) -> PResult<String> {
    quoted_string(p, '"')
}
/// Haskell `singleQuotedStringEscaped`.
pub fn single_quoted_string_escaped(p: &mut Parser) -> PResult<String> {
    quoted_string_escaped(p, '\'')
}
/// Haskell `doubleQuotedStringEscaped`.
pub fn double_quoted_string_escaped(p: &mut Parser) -> PResult<String> {
    quoted_string_escaped(p, '"')
}
/// Haskell `quotedString`.
pub fn quoted_string(p: &mut Parser, c: char) -> PResult<String> {
    p.char(c)?;
    let lit = p.many_till(|s| s.char_literal(), |s| s.char(c))?;
    Ok(lit.into_iter().collect())
}
/// Haskell `quotedStringEscaped`.
pub fn quoted_string_escaped(p: &mut Parser, q: char) -> PResult<String> {
    p.between(|s| s.char(q), |s| s.char(q), |s| string_with_escaped(s, &[q], None))
}
/// Haskell `brackets`.
pub fn brackets<T>(p: &mut Parser, inner: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
    p.between(
        |s| {
            symbol(s, "[")?;
            whitespace(s)
        },
        |s| {
            whitespace(s)?;
            symbol(s, "]")
        },
        inner,
    )
}
/// Haskell `untilWS`.
pub fn until_ws(p: &mut Parser) -> PResult<String> {
    let s = p.many_till(|s| s.any_single(), |s| s.space_char())?;
    Ok(s.into_iter().collect())
}
/// Haskell `heredocMarker`.
pub fn heredoc_marker(p: &mut Parser) -> PResult<String> {
    p.string("<<")?;
    p.take_while_p(Some("dash"), |c| c == '-')?;
    let m = p.alt(
        |s| s.try_(double_quoted_string),
        |s| s.alt(|s| s.try_(single_quoted_string), until_ws),
    )?;
    p.optional(heredoc_redirect)?;
    Ok(m)
}
/// Haskell `heredocRedirect`.
pub fn heredoc_redirect(p: &mut Parser) -> PResult<String> {
    p.alt(
        |s| s.string("|"),
        |s| s.alt(|s| s.string(">"), |s| s.string(">>")),
    )?;
    only_spaces(p)?;
    until_eol(p, "heredoc path")
}
/// Haskell `heredocContent`.
pub fn heredoc_content(p: &mut Parser, marker: &str) -> PResult<String> {
    let ter_eol = |s: &mut Parser| s.string(&format!("\n{marker}\n"));
    let ter_eof = |s: &mut Parser| {
        let t = s.string(&format!("\n{marker}"))?;
        s.hidden(|s| s.eof())?;
        Ok(t)
    };
    let termination = |s: &mut Parser| s.alt(|s| s.try_(ter_eol), ter_eof);
    let del_eol = |s: &mut Parser| s.string(&format!("{marker}\n"));
    let del_eof = |s: &mut Parser| {
        let t = s.string(marker)?;
        s.hidden(|s| s.eof())?;
        Ok(t)
    };
    let delimiter = |s: &mut Parser| s.alt(|s| s.try_(del_eol), del_eof);
    let empty_heredoc = p.observing(delimiter)?;
    let doc: String = match empty_heredoc {
        Err(_) => p.many_till(|s| s.any_single(), termination)?.into_iter().collect(),
        Ok(_) => String::new(),
    };
    Ok(doc.trim().to_string())
}
/// Haskell `heredoc`.
pub fn heredoc(p: &mut Parser) -> PResult<String> {
    let m = heredoc_marker(p)?;
    heredoc_content(p, &m)
}
/// Haskell `untilHeredoc`.
pub fn until_heredoc(p: &mut Parser) -> PResult<String> {
    let chars = |s: &mut Parser| -> PResult<String> {
        s.choice(&[
            &|s| escaped_line_breaks(s).map(cast_to_space),
            &|s| s.any_single_but('\n').map(|c| c.to_string()),
        ])
    };
    let txt = p.many_till(chars, heredoc)?;
    Ok(txt.concat().trim().to_string())
}
/// Haskell `onlySpaces`.
pub fn only_spaces(p: &mut Parser) -> PResult<String> {
    p.take_while_p(Some("spaces"), |c| c == ' ' || c == '\t')
}
/// Haskell `onlySpaces1`.
pub fn only_spaces1(p: &mut Parser) -> PResult<String> {
    p.take_while1_p(Some("at least one space"), |c| c == ' ' || c == '\t')
}
/// Haskell `onlyWhitespaces`.
pub fn only_whitespaces(p: &mut Parser) -> PResult<String> {
    p.take_while_p(Some("whitespaces"), |c| c == ' ' || c == '\t' || c == '\n' || c == '\r')
}
/// Haskell `escapedLineBreaks`.
pub fn escaped_line_breaks(p: &mut Parser) -> PResult<FoundWhitespace> {
    let breaks = p.some(|s| {
        s.try_(|s| {
            s.char(s.esc)?;
            only_spaces(s)?;
            s.take_while1_p(None, is_nl)
        })?;
        s.skip_many(|s| {
            s.try_(|s| {
                only_spaces(s)?;
                comment(s)?;
                s.take_while1_p(None, is_nl)
            })
        })?;
        // Spaces before the next '\' have a special significance
        // so we remembeer the fact that we found some
        s.alt(
            |s| only_spaces1(s).map(|_| FoundWhitespace::FoundWhitespace),
            |_| Ok(FoundWhitespace::MissingWhitespace),
        )
    })?;
    Ok(breaks.into_iter().fold(FoundWhitespace::mempty(), |a, b| a.combine(b)))
}
/// Haskell `escapedLineBreaks'`.
pub fn escaped_line_breaks_(p: &mut Parser) -> PResult<String> {
    let breaks = p.some(|s| {
        s.try_(|s| {
            s.char(s.esc)?;
            only_spaces(s)?;
            s.take_while1_p(None, is_nl)
        })?;
        s.skip_many(|s| {
            s.try_(|s| {
                only_spaces(s)?;
                comment(s)?;
                s.take_while1_p(None, is_nl)
            })
        })?;
        only_spaces1(s)
    })?;
    Ok(breaks.concat())
}
/// Haskell `foundWhitespace`.
pub fn found_whitespace(p: &mut Parser) -> PResult<FoundWhitespace> {
    let found = p.many(|s| {
        s.choice(&[
            &|s| only_spaces1(s).map(|_| FoundWhitespace::FoundWhitespace),
            &|s| escaped_line_breaks(s),
        ])
    })?;
    Ok(found.into_iter().fold(FoundWhitespace::mempty(), |a, b| a.combine(b)))
}
/// Haskell `whitespace`.
pub fn whitespace(p: &mut Parser) -> PResult<()> {
    found_whitespace(p).map(|_| ())
}
/// Haskell `requiredWhitespace`.
pub fn required_whitespace(p: &mut Parser) -> PResult<()> {
    match found_whitespace(p)? {
        FoundWhitespace::FoundWhitespace => Ok(()),
        FoundWhitespace::MissingWhitespace => p.fail("missing whitespace"),
    }
}
/// Haskell `untilEol`.
pub fn until_eol(p: &mut Parser, name: &str) -> PResult<String> {
    let predicate = p.many(|s| {
        s.choice(&[
            &|s| escaped_line_breaks(s).map(cast_to_space),
            &|s| {
                let esc = s.esc;
                s.take_while1_p(Some(name), |c| c != '\n' && c != esc)
            },
            &|s| {
                let esc = s.esc;
                let r = s.take_while1_p(None, |c| c == esc)?;
                s.not_followed_by(|s| s.char('\n'))?;
                Ok(r)
            },
        ])
    })?;
    let res = predicate.concat();
    if res.is_empty() {
        return p.fail(&format!("expecting {name}"));
    }
    Ok(res)
}
/// Haskell `symbol`.
pub fn symbol(p: &mut Parser, name: &str) -> PResult<String> {
    let x = p.string(name)?;
    whitespace(p)?;
    Ok(x)
}
/// Haskell `caseInsensitiveString`.
pub fn case_insensitive_string(p: &mut Parser, s: &str) -> PResult<String> {
    p.string_ci(s)
}

fn char_choice(p: &mut Parser, qs: &[char]) -> PResult<char> {
    match qs {
        [] => p.empty(),
        [c, rest @ ..] => p.alt(|s| s.char(*c), |s| char_choice(s, rest)),
    }
}

/// Haskell `stringWithEscaped`.
pub fn string_with_escaped(
    p: &mut Parser,
    quote_chars: &[char],
    maybe_accept_condition: Option<&dyn Fn(char) -> bool>,
) -> PResult<String> {
    let accept_condition = |c: char| match maybe_accept_condition {
        Some(f) => f(c),
        None => true,
    };
    let quote_parser = |s: &mut Parser| -> PResult<String> { char_choice(s, quote_chars).map(|c| c.to_string()) };
    let inner = |s: &mut Parser| -> PResult<String> {
        let esc = s.esc;
        let v = s.some(|s| {
            s.choice(&[
                &|s| escaped_line_breaks(s).map(cast_to_space),
                &|s| {
                    s.take_while1_p(None, |c| {
                        c != esc && c != '\n' && !quote_chars.contains(&c) && accept_condition(c)
                    })
                },
            ])
        })?;
        Ok(v.concat())
    };
    let sequences = p.many(|s| {
        s.choice(&[
            &|s| inner(s),
            &|s| {
                s.try_(|s| {
                    let esc = s.esc;
                    let r = s.take_while1_p(None, |c| c == esc)?;
                    s.not_followed_by(|s| quote_parser(s))?;
                    Ok(r)
                })
            },
            &|s| {
                let esc = s.esc;
                s.string(&esc.to_string())?;
                quote_parser(s)
            },
        ])
    })?;
    Ok(sequences.concat())
}
/// Haskell `lexeme`.
pub fn lexeme<T>(p: &mut Parser, inner: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
    let x = inner(p)?;
    required_whitespace(p)?;
    Ok(x)
}
/// Haskell `lexeme'`.
pub fn lexeme_<T>(p: &mut Parser, inner: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
    let x = inner(p)?;
    only_spaces(p)?;
    Ok(x)
}
/// Haskell `isNl`.
pub fn is_nl(c: char) -> bool {
    c == '\n'
}
/// Haskell `isSpaceNl` (`?esc` is the first argument).
pub fn is_space_nl(esc: char, c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n' || c == esc
}
/// Haskell `anyUnless`.
pub fn any_unless(p: &mut Parser, predicate: &dyn Fn(char) -> bool) -> PResult<String> {
    p.alt(|s| some_unless(s, "", predicate), |_| Ok(String::new()))
}
/// Haskell `someUnless`.
pub fn some_unless(p: &mut Parser, name: &str, predicate: &dyn Fn(char) -> bool) -> PResult<String> {
    let res = p.many(|s| {
        let esc = s.esc;
        s.choice(&[
            &|s| escaped_line_breaks(s).map(cast_to_space),
            &|s| s.take_while1_p(Some(name), |c| !(is_space_nl(esc, c) || predicate(c))),
            &|s| {
                let r = s.take_while1_p(None, |c| c == esc && !predicate(c))?;
                s.not_followed_by(|s| s.char('\n'))?;
                Ok(r)
            },
        ])
    })?;
    if res.is_empty() {
        return p.fail(&format!("expecting {name}"));
    }
    Ok(res.concat())
}
/// Haskell `comment`.
pub fn comment(p: &mut Parser) -> PResult<String> {
    p.char('#')?;
    p.take_while_p(None, |c| !is_nl(c))
}

#[allow(dead_code)]
fn _uses(_: Instruction<String>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combinator::run_parser;

    #[test]
    fn quoted_strings() {
        assert_eq!(run_parser("\"a b\"", "", '\\', double_quoted_string).unwrap(), "a b");
        assert_eq!(run_parser("'a\\'b'", "", '\\', single_quoted_string_escaped).unwrap(), "a'b");
    }

    #[test]
    fn heredoc_basic() {
        let r = run_parser("<<EOF\nhello\nEOF\n", "", '\\', heredoc).unwrap();
        assert_eq!(r, "hello");
        let r = run_parser("<<EOF\nEOF\n", "", '\\', heredoc).unwrap();
        assert_eq!(r, "");
    }

    #[test]
    fn escaped_breaks_and_eol() {
        let r = run_parser("\\\n  x", "", '\\', escaped_line_breaks).unwrap();
        assert_eq!(r, FoundWhitespace::FoundWhitespace);
        assert!(run_parser(" \n\n", "", '\\', eol).is_ok());
    }

    #[test]
    fn until_eol_joins_lines() {
        let r = run_parser("ab \\\n cd\nrest", "", '\\', |p| until_eol(p, "x")).unwrap();
        assert_eq!(r, "ab  cd");
    }

    #[test]
    fn comma_sep_and_brackets() {
        let r = run_parser("[ \"a\" , \"b\" ]", "", '\\', |p| {
            brackets(p, |p| comma_sep(p, double_quoted_string))
        })
        .unwrap();
        assert_eq!(r, vec!["a", "b"]);
    }
}
