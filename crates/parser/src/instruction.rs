// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Instruction.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use crate::arguments::arguments;
use crate::cmd::parse_cmd;
use crate::combinator::{PResult, Parser};
use crate::copy::{parse_add, parse_copy};
use crate::expose::parse_expose;
use crate::from::parse_from;
use crate::healthcheck::parse_healthcheck;
use crate::pairs::{parse_env, parse_label};
use crate::prelude::*;
use crate::run::parse_run;
use dlint_syntax::*;

/// Haskell `parseShell` (Instruction.hs).
pub fn parse_shell(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "SHELL")?;
    Ok(Instruction::Shell(arguments(p)?))
}

/// Haskell `parseStopSignal` (Instruction.hs).
pub fn parse_stop_signal(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "STOPSIGNAL")?;
    let args = until_eol(p, "the stop signal")?;
    Ok(Instruction::Stopsignal(args))
}

/// Haskell `parseArg` (Instruction.hs).
pub fn parse_arg(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "ARG")?;
    fn name_without_default(p: &mut Parser) -> PResult<Instruction<String>> {
        let name = some_unless(p, "the argument name", &|c| c == '=')?;
        until_eol(p, "the rest")?;
        Ok(Instruction::Arg(name, None))
    }
    fn name_with_default(p: &mut Parser) -> PResult<Instruction<String>> {
        let name = some_unless(p, "the argument name", &|c| c == '=')?;
        p.char('=')?;
        let df = until_eol(p, "the argument value")?;
        Ok(Instruction::Arg(name, Some(df)))
    }
    p.alt(
        |p| p.label("the arg name", |p| p.try_(name_with_default)),
        |p| {
            p.alt(
                |p| p.label("the arg name", |p| p.try_(name_without_default)),
                |p| {
                    let n = until_eol(p, "the argument name")?;
                    Ok(Instruction::Arg(n, None))
                },
            )
        },
    )
}

/// Haskell `parseUser` (Instruction.hs).
pub fn parse_user(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "USER")?;
    let username = until_eol(p, "the user")?;
    Ok(Instruction::User(username))
}

/// Haskell `parseWorkdir` (Instruction.hs).
pub fn parse_workdir(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "WORKDIR")?;
    let directory = until_eol(p, "the workdir path")?;
    Ok(Instruction::Workdir(directory))
}

/// Haskell `parseVolume` (Instruction.hs).
pub fn parse_volume(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "VOLUME")?;
    let directory = until_eol(p, "the volume path")?;
    Ok(Instruction::Volume(directory))
}

/// Haskell `parseMaintainer` (Instruction.hs).
pub fn parse_maintainer(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "MAINTAINER")?;
    let name = until_eol(p, "the maintainer name")?;
    Ok(Instruction::Maintainer(name))
}

/// Haskell `parseEntryPoint` (Instruction.hs).
pub fn parse_entry_point(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "ENTRYPOINT")?;
    Ok(Instruction::Entrypoint(arguments(p)?))
}

/// Haskell `parseOnbuild` (Instruction.hs).
pub fn parse_onbuild(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "ONBUILD")?;
    Ok(Instruction::OnBuild(Box::new(parse_instruction(p)?)))
}

/// Haskell `parsePragma` (Instruction.hs).
pub fn parse_pragma(p: &mut Parser) -> PResult<Instruction<String>> {
    lexeme_(p, |p| p.char('#'))?;
    p.choice(&[&|p| p.label("an escape", parse_escape_pragma), &|p| {
        p.label("a syntax", parse_syntax_pragma)
    }])
}

/// Haskell `parseEscapePragma` (Instruction.hs).
pub fn parse_escape_pragma(p: &mut Parser) -> PResult<Instruction<String>> {
    lexeme_(p, |p| p.string("escape"))?;
    lexeme_(p, |p| p.string("="))?;
    let c = p.char_literal()?;
    Ok(Instruction::Pragma(PragmaDirective::Escape(EscapeChar {
        escape: c,
    })))
}

/// Haskell `parseSyntaxPragma` (Instruction.hs).
pub fn parse_syntax_pragma(p: &mut Parser) -> PResult<Instruction<String>> {
    lexeme_(p, |p| p.string("syntax"))?;
    lexeme_(p, |p| p.string("="))?;
    let img = until_eol(p, "the syntax")?;
    Ok(Instruction::Pragma(PragmaDirective::Syntax(SyntaxImage {
        syntax: Image {
            registry_name: None,
            image_name: img,
        },
    })))
}

/// Haskell `parseComment` (Instruction.hs).
pub fn parse_comment(p: &mut Parser) -> PResult<Instruction<String>> {
    p.alt(
        |p| p.label("a pragma", |p| p.try_(parse_pragma)),
        |p| Ok(Instruction::Comment(comment(p)?)),
    )
}

/// Haskell `parseInstruction` (Instruction.hs).
pub fn parse_instruction(p: &mut Parser) -> PResult<Instruction<String>> {
    p.choice(&[
        &parse_onbuild,
        &parse_from,
        &parse_copy,
        &parse_run,
        &parse_workdir,
        &parse_entry_point,
        &parse_volume,
        &parse_expose,
        &parse_env,
        &parse_arg,
        &parse_user,
        &parse_label,
        &parse_stop_signal,
        &parse_cmd,
        &parse_shell,
        &parse_maintainer,
        &parse_add,
        &parse_comment,
        &parse_healthcheck,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combinator::run_parser;

    fn ast(src: &str) -> Instruction<String> {
        run_parser(src, "<string>", '\\', parse_instruction).expect("parse")
    }

    #[test]
    fn parses_arg_without_default() {
        assert_eq!(ast("ARG FOO"), Instruction::Arg("FOO".into(), None));
    }

    #[test]
    fn parses_arg_no_default_with_equals() {
        assert_eq!(ast("ARG FOO="), Instruction::Arg("FOO".into(), None));
    }

    #[test]
    fn parses_arg_with_default() {
        assert_eq!(
            ast("ARG FOO=bar"),
            Instruction::Arg("FOO".into(), Some("bar".into()))
        );
    }

    #[test]
    fn parses_maintainer() {
        assert_eq!(
            ast("MAINTAINER hudu@mail.com"),
            Instruction::Maintainer("hudu@mail.com".into())
        );
    }

    #[test]
    fn parses_comment() {
        assert_eq!(ast("# line 1"), Instruction::Comment(" line 1".into()));
    }

    #[test]
    fn parses_shell() {
        assert_eq!(
            ast("SHELL [\"/bin/bash\",  \"-c\"]"),
            Instruction::Shell(Arguments::ArgumentsList("/bin/bash -c".into()))
        );
    }
}
