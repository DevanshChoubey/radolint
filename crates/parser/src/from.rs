// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/From.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use dlint_syntax::*;
use crate::combinator::{PResult, Parser};


/// Haskell `parseRegistry` (From.hs).
pub fn parse_registry(p: &mut Parser) -> PResult<Registry> {
    todo!("port From.hs parseRegistry")
}

/// Haskell `parsePlatform` (From.hs).
pub fn parse_platform(p: &mut Parser) -> PResult<Platform> {
    todo!("port From.hs parsePlatform")
}

/// Haskell `someUnlessExpanded` (From.hs).
pub fn some_unless_expanded(p: &mut Parser, name: &str, predicate: &dyn Fn(char) -> bool) -> PResult<String> {
    todo!("port From.hs someUnlessExpanded")
}

/// Haskell `variableExpansion` (From.hs).
pub fn variable_expansion(p: &mut Parser) -> PResult<String> {
    todo!("port From.hs variableExpansion")
}

/// Haskell `parseBaseImage` (From.hs).
pub fn parse_base_image(p: &mut Parser, tag_parser: &dyn Fn(&mut Parser, &str) -> PResult<Option<Tag>>) -> PResult<BaseImage> {
    todo!("port From.hs parseBaseImage")
}

/// Haskell `taggedImage` (From.hs).
pub fn tagged_image(p: &mut Parser) -> PResult<BaseImage> {
    todo!("port From.hs taggedImage")
}

/// Haskell `parseDigest` (From.hs).
pub fn parse_digest(p: &mut Parser) -> PResult<Digest> {
    todo!("port From.hs parseDigest")
}

/// Haskell `untaggedImage` (From.hs).
pub fn untagged_image(p: &mut Parser) -> PResult<BaseImage> {
    todo!("port From.hs untaggedImage")
}

/// Haskell `imageAlias` (From.hs).
pub fn image_alias(p: &mut Parser) -> PResult<ImageAlias> {
    todo!("port From.hs imageAlias")
}

/// Haskell `baseImage` (From.hs).
pub fn base_image(p: &mut Parser) -> PResult<BaseImage> {
    todo!("port From.hs baseImage")
}

/// Haskell `parseFrom` (From.hs).
pub fn parse_from(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port From.hs parseFrom")
}

