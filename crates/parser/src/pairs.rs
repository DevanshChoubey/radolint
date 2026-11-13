// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Pairs.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use crate::combinator::{PResult, Parser};
use crate::prelude::*;
use dlint_syntax::*;

/// Haskell `unquotedString` (Pairs.hs).
pub fn unquoted_string(p: &mut Parser, accept_condition: &dyn Fn(char) -> bool) -> PResult<String> {
    let cond = |c: char| accept_condition(c) && c != '"' && c != '\'';
    let str_ = string_with_escaped(p, &[' ', '\t'], Some(&cond))?;
    // checkFaults
    match str_.chars().next() {
        None => p.fail("a non empty string"),
        Some('\'') => custom_error(
            p,
            DockerfileError::QuoteError("single".to_string(), str_.clone()),
        ),
        Some('"') => custom_error(
            p,
            DockerfileError::QuoteError("double".to_string(), str_.clone()),
        ),
        Some(_) => Ok(str_),
    }
}

/// Haskell `singleValue` (Pairs.hs).
pub fn single_value(p: &mut Parser, accept_condition: &dyn Fn(char) -> bool) -> PResult<String> {
    let variants = p.many(|s| {
        s.choice(&[
            &|s| {
                s.label("a string inside double quotes", |s| {
                    double_quoted_string_escaped(s)
                })
            },
            &|s| {
                s.label("a string inside single quotes", |s| {
                    single_quoted_string_escaped(s)
                })
            },
            &|s| {
                s.label("a string with no quotes", |s| {
                    unquoted_string(s, accept_condition)
                })
            },
        ])
    })?;
    Ok(variants.concat())
}

/// Haskell `pair` (Pairs.hs).
pub fn pair(p: &mut Parser) -> PResult<(String, String)> {
    let key = single_value(p, &|c| c != '=')?;
    let value = p.alt(
        |s| {
            // withEqualSign
            s.char('=')?;
            single_value(s, &|c| c != ' ' && c != '\t')
        },
        |s| {
            // withoutEqualSign
            required_whitespace(s)?;
            until_eol(s, "value")
        },
    )?;
    Ok((key, value))
}

/// Haskell `pairs` (Pairs.hs).
pub fn pairs(p: &mut Parser) -> PResult<Pairs> {
    p.sep_end_by1(
        |s| s.label("a key value pair (key=value)", pair),
        required_whitespace,
    )
}

/// Haskell `parseLabel` (Pairs.hs).
pub fn parse_label(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "LABEL")?;
    Ok(Instruction::Label(pairs(p)?))
}

/// Haskell `parseEnv` (Pairs.hs).
pub fn parse_env(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "ENV")?;
    Ok(Instruction::Env(pairs(p)?))
}

#[cfg(test)]
mod tests {
    // Ported from ParserSpec.hs "parse LABEL" / "parse ENV". DIVERGENCE: the Haskell spec goes
    // through the full dockerfile parser (assertAst); these call parse_label/parse_env directly.
    use super::*;
    use crate::combinator::run_parser;

    fn env(s: &str) -> Instruction<String> {
        run_parser(s, "<string>", '\\', parse_env).unwrap()
    }
    fn label(s: &str) -> Instruction<String> {
        run_parser(s, "<string>", '\\', parse_label).unwrap()
    }
    fn kv(a: &str, b: &str) -> (String, String) {
        (a.to_string(), b.to_string())
    }

    #[test]
    fn parse_label_test() {
        assert_eq!(
            label("LABEL foo=bar"),
            Instruction::Label(vec![kv("foo", "bar")])
        );
    }
    #[test]
    fn parse_space_separated_label() {
        assert_eq!(
            label("LABEL foo bar baz"),
            Instruction::Label(vec![kv("foo", "bar baz")])
        );
    }
    #[test]
    fn parse_quoted_labels() {
        assert_eq!(
            label("LABEL \"foo bar\"=baz"),
            Instruction::Label(vec![kv("foo bar", "baz")])
        );
    }
    #[test]
    fn parses_unquoted_pair() {
        assert_eq!(env("ENV foo=bar"), Instruction::Env(vec![kv("foo", "bar")]));
    }
    #[test]
    fn parses_pair_with_spaces_instead_of_equal_sign() {
        assert_eq!(env("ENV foo bar"), Instruction::Env(vec![kv("foo", "bar")]));
    }
    #[test]
    fn parse_with_many_spaces_and_tabs() {
        assert_eq!(
            env("ENV          NODE_VERSION  \t   v5.7.1"),
            Instruction::Env(vec![kv("NODE_VERSION", "v5.7.1")])
        );
    }
    #[test]
    fn parse_quoted_value_pair() {
        assert_eq!(
            env("ENV foo=\"bar\""),
            Instruction::Env(vec![kv("foo", "bar")])
        );
    }
    #[test]
    fn parse_multiple_unquoted_pairs() {
        assert_eq!(
            env("ENV foo=bar baz=foo"),
            Instruction::Env(vec![kv("foo", "bar"), kv("baz", "foo")])
        );
    }
    #[test]
    fn parse_multiple_quoted_pairs() {
        assert_eq!(
            env("ENV foo=\"bar\" baz=\"foo\""),
            Instruction::Env(vec![kv("foo", "bar"), kv("baz", "foo")])
        );
    }
    #[test]
    fn env_works_with_many_spaces_value() {
        assert_eq!(
            env("ENV foo val1 val2 val3 val4"),
            Instruction::Env(vec![kv("foo", "val1 val2 val3 val4")])
        );
    }
    #[test]
    fn parse_escaped_java_home() {
        assert_eq!(
            env("ENV JAVA_HOME=C:\\\\jdk1.8.0_112"),
            Instruction::Env(vec![kv("JAVA_HOME", "C:\\\\jdk1.8.0_112")])
        );
    }
}
