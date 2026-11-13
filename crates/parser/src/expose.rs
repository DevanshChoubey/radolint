// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Expose.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use dlint_syntax::*;
use crate::combinator::{PResult, Parser};


/// Haskell `parseExpose` (Expose.hs).
pub fn parse_expose(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Expose.hs parseExpose")
}

/// Haskell `ports` (Expose.hs).
pub fn ports(p: &mut Parser) -> PResult<Ports> {
    todo!("port Expose.hs ports")
}

/// Haskell `portspec` (Expose.hs).
pub fn portspec(p: &mut Parser) -> PResult<PortSpec> {
    todo!("port Expose.hs portspec")
}

/// Haskell `parsePortRangeSpec` (Expose.hs).
pub fn parse_port_range_spec(p: &mut Parser) -> PResult<PortSpec> {
    todo!("port Expose.hs parsePortRangeSpec")
}

/// Haskell `parsePortSpec` (Expose.hs).
pub fn parse_port_spec(p: &mut Parser) -> PResult<PortSpec> {
    todo!("port Expose.hs parsePortSpec")
}

/// Haskell `port` (Expose.hs).
pub fn port(p: &mut Parser) -> PResult<Port> {
    todo!("port Expose.hs port")
}

/// Haskell `portRangeLimit` (Expose.hs).
pub fn port_range_limit(p: &mut Parser) -> PResult<Port> {
    todo!("port Expose.hs portRangeLimit")
}

/// Haskell `portRange` (Expose.hs).
pub fn port_range(p: &mut Parser) -> PResult<PortRange> {
    todo!("port Expose.hs portRange")
}

/// Haskell `protocol` (Expose.hs).
pub fn protocol(p: &mut Parser) -> PResult<Protocol> {
    todo!("port Expose.hs protocol")
}

/// Haskell `portInt` (Expose.hs).
pub fn port_int(p: &mut Parser) -> PResult<Port> {
    todo!("port Expose.hs portInt")
}

/// Haskell `portWithProtocol` (Expose.hs).
pub fn port_with_protocol(p: &mut Parser) -> PResult<Port> {
    todo!("port Expose.hs portWithProtocol")
}

/// Haskell `portVariable` (Expose.hs).
pub fn port_variable(p: &mut Parser) -> PResult<Port> {
    todo!("port Expose.hs portVariable")
}

