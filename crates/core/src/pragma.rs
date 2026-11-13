// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Pragma.hs.

use crate::rule::RuleCode;
use dlint_parser::combinator::{PResult, Parser, run_parser};
use dlint_syntax::{Instruction, InstructionPos};
use std::collections::{BTreeMap, BTreeSet};

/// `Map.IntMap (Set RuleCode)`.
pub type IgnoreMap = BTreeMap<i64, BTreeSet<RuleCode>>;

fn codes(ignores: Vec<String>) -> BTreeSet<RuleCode> {
    ignores.into_iter().map(RuleCode::from).collect()
}

/// Haskell `ignored` (fold over the instructions).
pub fn ignored(dockerfile: &[InstructionPos<String>]) -> IgnoreMap {
    let mut acc = IgnoreMap::new();
    for ip in dockerfile {
        if let Instruction::Comment(comment) = &ip.instruction
            && let Some(ignores) = parse_ignore_pragma(comment)
            && !ignores.is_empty()
        {
            acc.insert(ip.line_number + 1, codes(ignores));
        }
    }
    acc
}

/// Haskell `stageIgnored`.
pub fn stage_ignored(dockerfile: &[InstructionPos<String>]) -> IgnoreMap {
    let mut acc = IgnoreMap::new();
    for ip in dockerfile {
        match &ip.instruction {
            Instruction::Comment(comment) => {
                if let Some(ignores) = parse_stage_ignore_pragma(comment)
                    && !ignores.is_empty()
                {
                    acc.insert(ip.line_number + 1, codes(ignores));
                }
            }
            Instruction::From(_) => {
                acc.entry(ip.line_number).or_default();
            }
            _ => {}
        }
    }
    acc
}

/// Haskell `globalIgnored`.
pub fn global_ignored(dockerfile: &[InstructionPos<String>]) -> BTreeSet<RuleCode> {
    let mut acc = BTreeSet::new();
    for ip in dockerfile {
        if let Instruction::Comment(comment) = &ip.instruction
            && let Some(ignores) = parse_global_ignore_pragma(comment)
            && !ignores.is_empty()
        {
            acc.extend(codes(ignores));
        }
    }
    acc
}

/// `Megaparsec.parseMaybe p` = `parse (p <* eof)`.
fn parse_maybe<T>(s: &str, p: impl Fn(&mut Parser) -> PResult<T>) -> Option<T> {
    run_parser(s, "", '\\', |q| {
        let x = p(q)?;
        q.eof()?;
        Ok(x)
    })
    .ok()
}

/// Haskell `parseIgnorePragma`.
pub fn parse_ignore_pragma(t: &str) -> Option<Vec<String>> {
    parse_maybe(t, |p| {
        hadolint_pragma(p)?;
        ignore(p)
    })
}

/// Haskell `parseStageIgnorePragma`.
pub fn parse_stage_ignore_pragma(t: &str) -> Option<Vec<String>> {
    parse_maybe(t, |p| {
        hadolint_pragma(p)?;
        stage(p)?;
        ignore(p)
    })
}

/// Haskell `parseGlobalIgnorePragma`.
pub fn parse_global_ignore_pragma(t: &str) -> Option<Vec<String>> {
    parse_maybe(t, |p| {
        hadolint_pragma(p)?;
        global(p)?;
        ignore(p)
    })
}

/// Haskell `parseShell`.
pub fn parse_shell(t: &str) -> Option<String> {
    parse_maybe(t, |p| {
        hadolint_pragma(p)?;
        p.string("shell")?;
        spaces(p)?;
        p.string("=")?;
        spaces(p)?;
        shell_name(p)
    })
}

fn hadolint_pragma(p: &mut Parser) -> PResult<String> {
    spaces(p)?;
    p.string("hadolint")?;
    spaces1(p)
}

fn stage(p: &mut Parser) -> PResult<String> {
    p.string("stage")?;
    spaces1(p)
}

fn global(p: &mut Parser) -> PResult<String> {
    p.string("global")?;
    spaces1(p)
}

fn ignore(p: &mut Parser) -> PResult<Vec<String>> {
    p.string("ignore")?;
    spaces(p)?;
    p.string("=")?;
    spaces(p)?;
    rule_list(p)
}

fn rule_list(p: &mut Parser) -> PResult<Vec<String>> {
    p.sep_by1(rule_name, |q| {
        spaces(q)?;
        q.string(",")?;
        spaces(q)
    })
}

fn rule_name(p: &mut Parser) -> PResult<String> {
    let r = p.take_while1_p(None, |c| "DLSC0123456789".contains(c))?;
    inline_comment(p)?;
    Ok(r)
}

fn shell_name(p: &mut Parser) -> PResult<String> {
    let r = p.take_while1_p(None, |c| !"\n\t ".contains(c))?;
    inline_comment(p)?;
    Ok(r)
}

fn inline_comment(p: &mut Parser) -> PResult<Option<String>> {
    spaces(p)?;
    p.optional(|q| {
        q.string("#")?;
        q.take_while_p(None, |c| c != '\n')
    })
}

fn spaces(p: &mut Parser) -> PResult<String> {
    p.take_while_p(None, space)
}

fn spaces1(p: &mut Parser) -> PResult<String> {
    p.take_while1_p(None, space)
}

fn space(c: char) -> bool {
    c == ' ' || c == '\t'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignore_pragma() {
        assert_eq!(
            parse_ignore_pragma(" hadolint ignore=DL3000,DL3001"),
            Some(vec!["DL3000".into(), "DL3001".into()])
        );
        assert_eq!(
            parse_ignore_pragma("hadolint ignore=DL3000 # why"),
            Some(vec!["DL3000".into()])
        );
        assert_eq!(parse_ignore_pragma("hadolint ignore=nope"), None);
        assert_eq!(
            parse_global_ignore_pragma("hadolint global ignore=DL3000"),
            Some(vec!["DL3000".into()])
        );
        assert_eq!(
            parse_shell("hadolint shell=powershell"),
            Some("powershell".into())
        );
    }
}
