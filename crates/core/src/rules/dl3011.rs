// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3011.hs.

use crate::rule::{simple_rule, DLSeverity, Rule};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3011";
    let severity = DLSeverity::DLErrorC;
    let message = "Valid UNIX ports range from 0 to 65535";
    simple_rule(code, severity, message, |instr| match instr {
        Instruction::Expose(Ports { un_ports: ports }) => {
            ports.iter().all(|s| match s {
                PortSpec::PortSpec(Port::Port(p, _)) => *p <= 65535,
                _ => true,
            }) && ports.iter().all(|s| match s {
                PortSpec::PortRangeSpec(PortRange(Port::Port(l, _), Port::Port(m, _))) => {
                    *l <= 65535 && *m <= 65535
                }
                // Haskell list-comprehension pattern: non-matching shapes are skipped.
                _ => true,
            })
        }
        _ => true,
    })
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn invalid_port() {
        rule_catches("DL3011", "EXPOSE 80000");
    }
    #[test]
    fn valid_port() {
        rule_catches_not("DL3011", "EXPOSE 60000");
    }
    #[test]
    fn valid_port_variable() {
        rule_catches_not("DL3011", "EXPOSE ${FOOBAR}");
    }
    #[test]
    fn invalid_port_in_range() {
        rule_catches("DL3011", "EXPOSE 40000-80000/tcp");
    }
    #[test]
    fn valid_port_range() {
        rule_catches_not("DL3011", "EXPOSE 40000-60000/tcp");
    }
    #[test]
    fn valid_port_range_variable() {
        rule_catches_not("DL3011", "EXPOSE 40000-${FOOBAR}");
    }
}
