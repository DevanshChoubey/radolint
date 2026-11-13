// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Healthcheck.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use dlint_syntax::*;
use crate::combinator::{PResult, Parser};

/// Haskell `data CheckFlag` (Healthcheck.hs, module-private upstream).
#[derive(Debug, Clone)]
pub enum CheckFlag {
    FlagInterval(Duration),
    FlagTimeout(Duration),
    FlagStartPeriod(Duration),
    FlagStartInterval(Duration),
    FlagRetries(Retries),
    CFlagInvalid((String, String)),
}

/// Haskell `parseHealthcheck` (Healthcheck.hs).
pub fn parse_healthcheck(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Healthcheck.hs parseHealthcheck")
}

/// Haskell `checkFlag` (Healthcheck.hs).
pub fn check_flag(p: &mut Parser) -> PResult<CheckFlag> {
    todo!("port Healthcheck.hs checkFlag")
}

/// Haskell `durationFlag` (Healthcheck.hs).
pub fn duration_flag(p: &mut Parser, flag_name: &str) -> PResult<Duration> {
    todo!("port Healthcheck.hs durationFlag")
}

/// Haskell `retriesFlag` (Healthcheck.hs).
pub fn retries_flag(p: &mut Parser) -> PResult<Retries> {
    todo!("port Healthcheck.hs retriesFlag")
}

/// Haskell `anyFlag` (Healthcheck.hs).
pub fn any_flag(p: &mut Parser) -> PResult<(String, String)> {
    todo!("port Healthcheck.hs anyFlag")
}

/// Haskell `unexpectedFlag` (Healthcheck.hs).
pub fn unexpected_flag<T>(p: &mut Parser, name: &str, value: &str) -> PResult<T> {
    todo!("port Healthcheck.hs unexpectedFlag")
}

