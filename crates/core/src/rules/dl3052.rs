// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3052.hs.

use crate::rule::LabelSchema;
use crate::rule::{mconcat, simple_rule, DLSeverity, LabelName, LabelType, Rule};
use dlint_syntax::{Instruction, Pairs};

/// Haskell `rule`.
pub fn rule(label_schema: &LabelSchema) -> Box<dyn Rule> {
    // BTreeMap iterates in key order, like `Map.keys`.
    mconcat(
        label_schema
            .iter()
            .filter(|(_, t)| **t == LabelType::Url)
            .map(|(k, _)| label_is_not_url_rule(k))
            .collect(),
    )
}

/// Haskell `labelIsNotUrlRule`.
fn label_is_not_url_rule(label: &LabelName) -> Box<dyn Rule> {
    let code = "DL3052";
    let severity = DLSeverity::DLWarningC;
    let message = format!("Label `{}` is not a valid URL.", label);
    let label = label.clone();
    let check = move |instr: &Instruction<_>| match instr {
        Instruction::Label(ls) => get_bad_url_labels(&label, ls).is_empty(),
        _ => true,
    };
    simple_rule(code, severity, &message, check)
}

/// Haskell `getBadURLLabels`.
fn get_bad_url_labels(lbl: &str, prs: &Pairs) -> Pairs {
    prs.iter()
        .filter(|(l, u)| l == lbl && !parse_uri(u))
        .cloned()
        .collect()
}

// DIVERGENCE: `Network.URI.parseURI` is emulated by a validity check (RFC 3986 absolute URI
// as accepted by network-uri); only whether parsing succeeds is needed.
fn parse_uri(s: &str) -> bool {
    let b = s.as_bytes();
    // scheme
    if b.is_empty() || !b[0].is_ascii_alphabetic() {
        return false;
    }
    let mut i = 1;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || matches!(b[i], b'+' | b'-' | b'.')) {
        i += 1;
    }
    if i >= b.len() || b[i] != b':' {
        return false;
    }
    let rest = &s[i + 1..];
    // fragment, then query
    let (rest, frag) = match rest.find('#') {
        Some(p) => (&rest[..p], Some(&rest[p + 1..])),
        None => (rest, None),
    };
    let (hier, query) = match rest.find('?') {
        Some(p) => (&rest[..p], Some(&rest[p + 1..])),
        None => (rest, None),
    };
    if let Some(f) = frag {
        if !all_chars(f, |c| c == b'/' || c == b'?' || pchar(c)) {
            return false;
        }
    }
    if let Some(q) = query {
        if !all_chars(q, |c| c == b'/' || c == b'?' || pchar(c)) {
            return false;
        }
    }
    hier_part(hier)
}

fn hier_part(h: &str) -> bool {
    if let Some(r) = h.strip_prefix("//") {
        let end = r.find('/').unwrap_or(r.len());
        authority(&r[..end]) && path_chars(&r[end..])
    } else if h.starts_with('/') {
        // path-absolute: no empty first segment
        !h.starts_with("//") && path_chars(h)
    } else if h.is_empty() {
        true
    } else {
        // path-rootless: first segment non-empty and, in network-uri, any pchar
        path_chars(h)
    }
}

fn path_chars(p: &str) -> bool {
    all_chars(p, |c| c == b'/' || pchar(c))
}

fn authority(a: &str) -> bool {
    let hostport = match a.find('@') {
        Some(p) => {
            if !all_chars(&a[..p], |c| unreserved(c) || sub_delim(c) || c == b':') {
                return false;
            }
            &a[p + 1..]
        }
        None => a,
    };
    if let Some(r) = hostport.strip_prefix('[') {
        let Some(e) = r.find(']') else { return false };
        let lit = &r[..e];
        let after = &r[e + 1..];
        if !(ipv6(lit) || ipv_future(lit)) {
            return false;
        }
        return port_ok(after);
    }
    let (host, port) = match hostport.find(':') {
        Some(p) => (&hostport[..p], &hostport[p..]),
        None => (hostport, ""),
    };
    all_chars(host, |c| unreserved(c) || sub_delim(c)) && port_ok(port)
}

fn port_ok(p: &str) -> bool {
    match p.strip_prefix(':') {
        None => p.is_empty(),
        Some(d) => d.bytes().all(|c| c.is_ascii_digit()),
    }
}

fn ipv_future(s: &str) -> bool {
    let Some(r) = s.strip_prefix('v') else { return false };
    let Some(p) = r.find('.') else { return false };
    p > 0
        && r[..p].bytes().all(|c| c.is_ascii_hexdigit())
        && p + 1 < r.len()
        && r[p + 1..].bytes().all(|c| unreserved(c) || sub_delim(c) || c == b':')
}

fn ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 4
        && parts.iter().all(|p| {
            !p.is_empty() && p.len() <= 3 && p.bytes().all(|c| c.is_ascii_digit()) && p.parse::<u32>().map_or(false, |n| n <= 255)
        })
}

fn ipv6(s: &str) -> bool {
    if !s.is_ascii() {
        return false;
    }
    let (head, tail) = match s.find("::") {
        Some(p) => (&s[..p], Some(&s[p + 2..])),
        None => (s, None),
    };
    if tail.map_or(false, |t| t.contains("::")) {
        return false;
    }
    let groups = |x: &str| -> Option<usize> {
        if x.is_empty() {
            return Some(0);
        }
        let parts: Vec<&str> = x.split(':').collect();
        let mut n = 0;
        for (i, p) in parts.iter().enumerate() {
            if i == parts.len() - 1 && p.contains('.') {
                if !ipv4(p) {
                    return None;
                }
                n += 2;
            } else if !p.is_empty() && p.len() <= 4 && p.bytes().all(|c| c.is_ascii_hexdigit()) {
                n += 1;
            } else {
                return None;
            }
        }
        Some(n)
    };
    match tail {
        None => groups(head) == Some(8),
        Some(t) => match (groups(head), groups(t)) {
            (Some(a), Some(b)) => a + b <= 7,
            _ => false,
        },
    }
}

fn all_chars(s: &str, f: impl Fn(u8) -> bool) -> bool {
    // percent-encoded triplets are allowed everywhere pchar is
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            if i + 2 < b.len() + 0 && b[i + 1].is_ascii_hexdigit() && b[i + 2].is_ascii_hexdigit() {
                i += 3;
                continue;
            }
            return false;
        }
        if !f(b[i]) {
            return false;
        }
        i += 1;
    }
    true
}

fn unreserved(c: u8) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, b'-' | b'.' | b'_' | b'~')
}
fn sub_delim(c: u8) -> bool {
    matches!(c, b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'=')
}
fn pchar(c: u8) -> bool {
    unreserved(c) || sub_delim(c) || c == b':' || c == b'@'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::configuration::Configuration;
    use crate::testing::*;

    fn config() -> Configuration {
        let mut c = Configuration::default();
        c.label_schema = [("urllabel".to_string(), LabelType::Url)].into_iter().collect();
        c
    }

    #[test]
    fn not_ok_with_label_not_containing_url() {
        let c = config();
        rule_catches_with(&c, "DL3052", "LABEL urllabel=\"not-url\"");
        on_build_rule_catches_with(&c, "DL3052", "LABEL urllabel=\"not-url\"");
    }

    #[test]
    fn ok_with_label_containing_url() {
        let c = config();
        rule_catches_not_with(&c, "DL3052", "LABEL urllabel=\"http://example.com\"");
        on_build_rule_catches_not_with(&c, "DL3052", "LABEL urllabel=\"http://example.com\"");
    }

    #[test]
    fn ok_with_other_label_not_containing_url() {
        let c = config();
        rule_catches_not_with(&c, "DL3052", "LABEL other=\"foo\"");
        on_build_rule_catches_not_with(&c, "DL3052", "LABEL other=\"bar\"");
    }

    #[test]
    fn parse_uri_basics() {
        assert!(parse_uri("http://example.com"));
        assert!(parse_uri("https://a.b:80/x?y#z"));
        assert!(!parse_uri("not-url"));
        assert!(!parse_uri("/relative"));
        assert!(!parse_uri("http://exa mple.com"));
    }
}
