// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3053.hs.

use crate::rule::LabelSchema;
use crate::rule::*;
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule(label_schema: &LabelSchema) -> Box<dyn Rule> {
    // `Map.keys (Map.filter (== Rfc3339) labelschema)` (ascending key order)
    mconcat(
        label_schema
            .iter()
            .filter(|(_, t)| **t == LabelType::Rfc3339)
            .map(|(k, _)| label_is_not_rfc3339_rule(k.clone()))
            .collect(),
    )
}

/// Haskell `labelIsNotRFC3339Rule`.
fn label_is_not_rfc3339_rule(label: LabelName) -> Box<dyn Rule> {
    let code = "DL3053";
    let severity = DLSeverity::DLWarningC;
    let message = format!(
        "Label `{}` is not a valid time format - must conform to RFC3339.",
        label
    );
    let check = move |instr: &Instruction<crate::shell::ParsedShell>| -> bool {
        match instr {
            Instruction::Label(ls) => get_bad_timeformat_labels(&label, ls).is_empty(),
            _ => true,
        }
    };
    simple_rule(code, severity, &message, check)
}

/// Haskell `getBadTimeformatLabels`.
fn get_bad_timeformat_labels(lbl: &str, pairs: &Pairs) -> Pairs {
    pairs
        .iter()
        .filter(|(l, v)| l == lbl && parse_time_rfc3339(v).is_none())
        .cloned()
        .collect()
}

/// Haskell `Data.Time.RFC3339.parseTimeRFC3339` (timerep), reduced to validity.
// DIVERGENCE: no time library; returns Some(()) when the text is a valid RFC3339 timestamp.
fn parse_time_rfc3339(s: &str) -> Option<()> {
    let b = s.as_bytes();
    let num = |from: usize, n: usize| -> Option<u32> {
        let sl = b.get(from..from + n)?;
        if !sl.iter().all(|c| c.is_ascii_digit()) {
            return None;
        }
        std::str::from_utf8(sl).ok()?.parse().ok()
    };
    let year = num(0, 4)?;
    if b.get(4) != Some(&b'-') {
        return None;
    }
    let month = num(5, 2)?;
    if b.get(7) != Some(&b'-') {
        return None;
    }
    let day = num(8, 2)?;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let dim = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        _ => return None,
    };
    if day < 1 || day > dim {
        return None;
    }
    if !matches!(b.get(10), Some(b'T') | Some(b't')) {
        return None;
    }
    let hour = num(11, 2)?;
    if b.get(13) != Some(&b':') {
        return None;
    }
    let min = num(14, 2)?;
    if b.get(16) != Some(&b':') {
        return None;
    }
    let sec = num(17, 2)?;
    if hour > 23 || min > 59 || sec > 60 {
        return None;
    }
    let mut i = 19;
    if b.get(i) == Some(&b'.') {
        i += 1;
        let st = i;
        while b.get(i).is_some_and(|c| c.is_ascii_digit()) {
            i += 1;
        }
        if i == st {
            return None;
        }
    }
    match b.get(i) {
        Some(b'Z') | Some(b'z') => {
            if i + 1 == b.len() {
                Some(())
            } else {
                None
            }
        }
        Some(b'+') | Some(b'-') => {
            let zh = num(i + 1, 2)?;
            if b.get(i + 3) != Some(&b':') {
                return None;
            }
            let zm = num(i + 4, 2)?;
            if zh > 23 || zm > 59 || i + 6 != b.len() {
                return None;
            }
            Some(())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::config::configuration::Configuration;
    use crate::rule::LabelType;
    use crate::testing::*;

    fn config() -> Configuration {
        let mut c = Configuration::default();
        c.label_schema
            .insert("datelabel".into(), LabelType::Rfc3339);
        c
    }

    #[test]
    fn not_ok_with_label_not_containing_rfc3339_date() {
        let c = config();
        rule_catches_with(&c, "DL3053", "LABEL datelabel=\"not-date\"");
        on_build_rule_catches_with(&c, "DL3053", "LABEL datelabel=\"not-date\"");
    }
    #[test]
    fn ok_with_label_containing_rfc3339_date() {
        let c = config();
        rule_catches_not_with(
            &c,
            "DL3053",
            "LABEL datelabel=\"2021-03-10T10:26:33.564595127+01:00\"",
        );
        on_build_rule_catches_not_with(
            &c,
            "DL3053",
            "LABEL datelabel=\"2021-03-10T10:26:33.564595127+01:00\"",
        );
    }
    #[test]
    fn ok_with_other_label_not_containing_rfc3339_date() {
        let c = config();
        rule_catches_not_with(&c, "DL3053", "LABEL other=\"doo\"");
        on_build_rule_catches_not_with(&c, "DL3053", "LABEL other=\"bar\"");
    }
}
