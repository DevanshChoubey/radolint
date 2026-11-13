// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Cmd.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

use dlint_syntax::*;
use crate::arguments::arguments;
use crate::combinator::{PResult, Parser};
use crate::prelude::reserved;

/// Haskell `parseCmd` (Cmd.hs).
pub fn parse_cmd(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "CMD")?;
    Ok(Instruction::Cmd(arguments(p)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_text;

    fn text(s: &str) -> Instruction<String> {
        Instruction::Cmd(Arguments::ArgumentsText(s.to_string()))
    }

    fn list(args: &[&str]) -> Instruction<String> {
        Instruction::Cmd(Arguments::ArgumentsList(args.join(" ")))
    }

    /// Haskell `assertAst` (TestHelper.hs).
    fn assert_ast(s: &str, ast: Vec<Instruction<String>>) {
        match parse_text(s) {
            Err(e) => panic!("{}", e.bundle_pretty()),
            Ok(d) => {
                let got: Vec<Instruction<String>> = d.into_iter().map(|i| i.instruction).collect();
                assert_eq!(ast, got, "ASTs are not equal");
            }
        }
    }

    #[test]
    fn one_line_cmd() {
        assert_ast("CMD true", vec![text("true")]);
    }

    #[test]
    fn cmd_over_several_lines() {
        assert_ast("CMD true \\\n && true", vec![text("true  && true")]);
    }

    #[test]
    fn quoted_command_params() {
        assert_ast("CMD [\"echo\",  \"1\"]", vec![list(&["echo", "1"])]);
    }

    #[test]
    fn parses_commas_correctly() {
        assert_ast("CMD [ \"echo\" ,\"-e\" , \"1\"]", vec![list(&["echo", "-e", "1"])]);
    }

    #[test]
    fn parse_exec_style_cmd_with_long_broken_lines() {
        let cmd = [
            "CMD [ \"/bin/sh\", \"-c\", \\",
            "      \"echo foo && \\",
            "       echo bar\" \\",
            "    ]",
        ]
        .iter()
        .map(|l| format!("{}\n", l))
        .collect::<String>();
        assert_ast(&cmd, vec![list(&["/bin/sh", "-c", "echo foo &&  echo bar"])]);
    }

    #[test]
    fn parse_exec_style_cmd_with_long_broken_lines_backtick_escape() {
        let cmd = [
            "# escape = `",
            "CMD [ \"/bin/sh\", \"-c\", `",
            "      \"echo foo && `",
            "       echo bar\" `",
            "    ]",
        ]
        .iter()
        .map(|l| format!("{}\n", l))
        .collect::<String>();
        assert_ast(
            &cmd,
            vec![
                Instruction::Pragma(PragmaDirective::Escape(EscapeChar { escape: '`' })),
                list(&["/bin/sh", "-c", "echo foo &&  echo bar"]),
            ],
        );
    }

    #[test]
    fn parse_exec_style_cmd_with_escaped_characters() {
        let cmd = "CMD [ \"sbt\", \"set reStart / mainClass := Some(\\\"Main\\\");~reStart\" ]\n";
        assert_ast(cmd, vec![list(&["sbt", "set reStart / mainClass := Some(\"Main\");~reStart"])]);
    }

    #[test]
    fn parse_exec_style_cmd_with_windows_style_escaped_characters() {
        let cmd = [
            "# escape = `",
            "CMD [ \"sbt\", \"set reStart / mainClass := Some(`\"Main`\");~reStart\" ]",
        ]
        .iter()
        .map(|l| format!("{}\n", l))
        .collect::<String>();
        assert_ast(
            &cmd,
            vec![
                Instruction::Pragma(PragmaDirective::Escape(EscapeChar { escape: '`' })),
                list(&["sbt", "set reStart / mainClass := Some(\"Main\");~reStart"]),
            ],
        );
    }
}
