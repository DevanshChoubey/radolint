// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Expose.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use dlint_syntax::*;
use crate::combinator::{PResult, Parser};
use crate::prelude::*;


/// Haskell `parseExpose` (Expose.hs).
pub fn parse_expose(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "EXPOSE")?;
    Ok(Instruction::Expose(ports(p)?))
}

/// Haskell `ports` (Expose.hs).
pub fn ports(p: &mut Parser) -> PResult<Ports> {
    let un_ports = p.sep_end_by(portspec, required_whitespace)?;
    Ok(Ports { un_ports })
}

/// Haskell `portspec` (Expose.hs).
pub fn portspec(p: &mut Parser) -> PResult<PortSpec> {
    p.alt(
        |s| {
            s.label("A range of ports optionally followed by the protocol", |s| {
                s.try_(parse_port_range_spec)
            })
        },
        |s| s.label("A port optionally followed by the protocol", parse_port_spec),
    )
}

/// Haskell `parsePortRangeSpec` (Expose.hs).
pub fn parse_port_range_spec(p: &mut Parser) -> PResult<PortSpec> {
    Ok(PortSpec::PortRangeSpec(port_range(p)?))
}

/// Haskell `parsePortSpec` (Expose.hs).
pub fn parse_port_spec(p: &mut Parser) -> PResult<PortSpec> {
    Ok(PortSpec::PortSpec(port(p)?))
}

/// Haskell `port` (Expose.hs).
pub fn port(p: &mut Parser) -> PResult<Port> {
    p.choice(&[
        &|s| s.label("a variable", |s| s.try_(port_variable)),
        &|s| s.label("a port with its protocol (udp/tcp)", |s| s.try_(port_with_protocol)),
        &|s| s.label("a valid port number", port_int),
    ])
}

/// Haskell `portRangeLimit` (Expose.hs).
pub fn port_range_limit(p: &mut Parser) -> PResult<Port> {
    fn number(p: &mut Parser) -> PResult<Port> {
        let num = natural(p)?;
        Ok(Port::Port(num as i64, Protocol::TCP))
    }
    fn variable(p: &mut Parser) -> PResult<Port> {
        p.char('$')?;
        let var = some_unless(p, "the variable name", &|c| c == '-' || c == '/')?;
        Ok(Port::PortStr(format!("${var}")))
    }
    p.alt(number, variable)
}

/// Haskell `portRange` (Expose.hs).
pub fn port_range(p: &mut Parser) -> PResult<PortRange> {
    fn set_proto(port: Port, prot: Protocol) -> Port {
        match port {
            Port::Port(n, _) => Port::Port(n, prot),
            Port::PortStr(s) => Port::PortStr(s),
        }
    }
    let start = port_range_limit(p)?;
    p.char('-')?;
    let finish = p.try_(port_range_limit)?;
    let proto = p.alt(|s| s.try_(protocol), |_| Ok(Protocol::TCP))?;
    Ok(PortRange(set_proto(start, proto.clone()), set_proto(finish, proto)))
}

/// Haskell `protocol` (Expose.hs).
pub fn protocol(p: &mut Parser) -> PResult<Protocol> {
    p.char('/')?;
    p.alt(
        |s| {
            s.try_(|s| {
                s.alt(
                    |s| {
                        case_insensitive_string(s, "tcp")?;
                        Ok(Protocol::TCP)
                    },
                    |s| {
                        case_insensitive_string(s, "udp")?;
                        Ok(Protocol::UDP)
                    },
                )
            })
        },
        |s| s.fail("invalid protocol"),
    )
}

/// Haskell `portInt` (Expose.hs).
pub fn port_int(p: &mut Parser) -> PResult<Port> {
    let port_number = natural(p)?;
    p.not_followed_by(|s| s.alt(|s| s.string("/"), |s| s.string("-")))?;
    Ok(Port::Port(port_number as i64, Protocol::TCP))
}

/// Haskell `portWithProtocol` (Expose.hs).
pub fn port_with_protocol(p: &mut Parser) -> PResult<Port> {
    let port_number = natural(p)?;
    Ok(Port::Port(port_number as i64, protocol(p)?))
}

/// Haskell `portVariable` (Expose.hs).
pub fn port_variable(p: &mut Parser) -> PResult<Port> {
    p.char('$')?;
    let variable = some_unless(p, "the variable name", &|c| c == '$')?;
    Ok(Port::PortStr(format!("${variable}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_text;

    fn ast(content: &str) -> Vec<Instruction<String>> {
        parse_text(content).unwrap().into_iter().map(|i| i.instruction).collect()
    }
    fn tcp(n: i64) -> Port { Port::Port(n, Protocol::TCP) }
    fn udp(n: i64) -> Port { Port::Port(n, Protocol::UDP) }
    fn sv(s: &str) -> Port { Port::PortStr(s.to_string()) }
    fn ex(v: Vec<PortSpec>) -> Vec<Instruction<String>> {
        vec![Instruction::Expose(Ports { un_ports: v })]
    }
    fn ps(p: Port) -> PortSpec { PortSpec::PortSpec(p) }
    fn pr(a: Port, b: Port) -> PortSpec { PortSpec::PortRangeSpec(PortRange(a, b)) }

    #[test]
    fn should_handle_number_ports() {
        assert_eq!(ast("EXPOSE 8080"), ex(vec![ps(tcp(8080))]));
    }

    #[test]
    fn should_handle_many_number_ports() {
        assert_eq!(ast("EXPOSE 8080 8081"), ex(vec![ps(tcp(8080)), ps(tcp(8081))]));
    }

    #[test]
    fn should_handle_ports_with_protocol() {
        assert_eq!(ast("EXPOSE 8080/TCP 8081/UDP"), ex(vec![ps(tcp(8080)), ps(udp(8081))]));
    }

    #[test]
    fn should_handle_ports_with_protocol_and_variables() {
        assert_eq!(
            ast("EXPOSE $PORT 8080 8081/UDP"),
            ex(vec![ps(sv("$PORT")), ps(tcp(8080)), ps(udp(8081))])
        );
    }

    #[test]
    fn should_handle_port_ranges() {
        assert_eq!(
            ast("EXPOSE 80 81 8080-8085"),
            ex(vec![ps(tcp(80)), ps(tcp(81)), pr(tcp(8080), tcp(8085))])
        );
    }

    #[test]
    fn should_handle_udp_port_ranges() {
        assert_eq!(
            ast("EXPOSE 80 81 8080-8085/udp"),
            ex(vec![ps(tcp(80)), ps(tcp(81)), pr(udp(8080), udp(8085))])
        );
    }

    #[test]
    fn should_handle_one_variable_port_ranges_1() {
        assert_eq!(ast("EXPOSE 8080-${bar}/udp"), ex(vec![pr(udp(8080), sv("${bar}"))]));
    }

    #[test]
    fn should_handle_one_variable_port_ranges_2() {
        assert_eq!(ast("EXPOSE ${foo}-8080/udp"), ex(vec![pr(sv("${foo}"), udp(8080))]));
    }

    #[test]
    fn should_handle_two_variables_port_ranges() {
        assert_eq!(ast("EXPOSE ${foo}-${bar}/udp"), ex(vec![pr(sv("${foo}"), sv("${bar}"))]));
    }

    #[test]
    fn should_handle_multiline_variables() {
        let content = "EXPOSE  ${PORT} ${PORT_SSL} \\\n        ${PORT_HTTP} ${PORT_HTTPS} \\\n        ${PORT_REP} \\\n        ${PORT_ADMIN} ${PORT_ADMIN_HTTP}";
        assert_eq!(
            ast(content),
            ex(vec![
                ps(sv("${PORT}")),
                ps(sv("${PORT_SSL}")),
                ps(sv("${PORT_HTTP}")),
                ps(sv("${PORT_HTTPS}")),
                ps(sv("${PORT_REP}")),
                ps(sv("${PORT_ADMIN}")),
                ps(sv("${PORT_ADMIN_HTTP}")),
            ])
        );
    }

    #[test]
    fn should_fail_with_wrong_protocol() {
        assert!(parse_text("EXPOSE 80/ip").is_err());
    }
}
