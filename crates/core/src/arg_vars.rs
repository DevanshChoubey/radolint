// SPDX-License-Identifier: GPL-3.0-only
//! Build variables (ARG and ENV) and their expansion, the way BuildKit expands them in
//! instructions. Not in hadolint: used by the DIVERGENCE fixes in DL3026 (FROM) and DL3020
//! (ADD), see docs/divergences.md.

use std::collections::BTreeMap;

/// A set of build variables. A value of `None` is unknown: an ARG declared without a default,
/// which only `--build-arg` can set.
#[derive(Clone, Debug, Default)]
pub struct Vars {
    map: BTreeMap<String, Option<String>>,
    /// Whether a name that was never declared is unknown rather than unset. Inside a build
    /// stage it may come from the base image's ENV, so it is unknown there. Before the first
    /// FROM nothing else can set it, so it is unset.
    undeclared_unknown: bool,
}

impl Vars {
    /// Variables before the first FROM: undeclared names are unset.
    pub fn global() -> Self {
        Self::default()
    }

    /// Variables inside a build stage: undeclared names are unknown.
    pub fn stage() -> Self {
        Self {
            map: BTreeMap::new(),
            undeclared_unknown: true,
        }
    }

    /// Sets one variable.
    pub fn set(&mut self, name: &str, value: Option<String>) {
        self.map.insert(name.to_string(), value);
    }

    /// Records the variables of one `ARG` line. The parser keeps everything after the first `=`
    /// as one value, so `ARG A=x B=y` arrives as `A` = `x B=y` and is split here. Defaults that
    /// refer to earlier variables (`ARG IMG=${REG}/app`) are expanded now, as Docker does.
    ///
    /// Inside a stage, `outer` holds the global ARGs: an ARG without a default that names a
    /// global ARG brings that ARG's value into the stage.
    pub fn record_arg(&mut self, name: &str, value: Option<&str>, outer: Option<&Vars>) {
        let mut pairs: Vec<(String, Option<String>)> = Vec::new();
        match value {
            None => pairs.push((name.to_string(), None)),
            Some(v) => {
                let mut words = split_words(v).into_iter();
                pairs.push((name.to_string(), Some(words.next().unwrap_or_default())));
                for w in words {
                    match w.split_once('=') {
                        Some((k, v)) => pairs.push((k.to_string(), Some(unquote(v).to_string()))),
                        None => pairs.push((w, None)),
                    }
                }
            }
        }
        for (k, v) in pairs {
            let v = match v {
                Some(v) if v.contains('$') => self.expand(&v),
                Some(v) => Some(v),
                None => outer.and_then(|o| o.map.get(&k).cloned().flatten()),
            };
            self.map.insert(k, v);
        }
    }

    /// Expands `$NAME`, `${NAME}`, `${NAME:-word}`, `${NAME-word}`, `${NAME:+word}` and
    /// `${NAME+word}`. `None` means the result is unknown.
    pub fn expand(&self, s: &str) -> Option<String> {
        let chars: Vec<char> = s.chars().collect();
        let mut out = String::new();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] != '$' {
                out.push(chars[i]);
                i += 1;
                continue;
            }
            i += 1;
            if chars.get(i) == Some(&'{') {
                // Find the matching close brace.
                let start = i + 1;
                let mut depth = 1;
                let mut j = start;
                while j < chars.len() && depth > 0 {
                    match chars[j] {
                        '{' => depth += 1,
                        '}' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                if depth != 0 {
                    return None;
                }
                let body: String = chars[start..j - 1].iter().collect();
                out.push_str(&self.expand_braced(&body)?);
                i = j;
            } else {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                if start == i {
                    out.push('$');
                    continue;
                }
                let name: String = chars[start..i].iter().collect();
                out.push_str(&self.lookup(&name)?.unwrap_or_default());
            }
        }
        Some(out)
    }

    /// The value of a variable. `None`: unknown. `Some(None)`: unset.
    fn lookup(&self, name: &str) -> Option<Option<String>> {
        match self.map.get(name) {
            Some(v) => v.clone().map(Some),
            None if self.undeclared_unknown => None,
            None => Some(None),
        }
    }

    fn expand_braced(&self, body: &str) -> Option<String> {
        let name_len = body
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(body.len());
        let (name, rest) = body.split_at(name_len);
        let value = self.lookup(name)?;
        let (op, word) = if let Some(w) = rest.strip_prefix(":-") {
            (":-", w)
        } else if let Some(w) = rest.strip_prefix(":+") {
            (":+", w)
        } else if let Some(w) = rest.strip_prefix('-') {
            ("-", w)
        } else if let Some(w) = rest.strip_prefix('+') {
            ("+", w)
        } else if rest.is_empty() {
            return Some(value.unwrap_or_default());
        } else {
            // `${NAME:?msg}` and other forms: leave the value unknown.
            return None;
        };
        match (op, value) {
            (":-", None) | ("-", None) => self.expand(word),
            (":-", Some(v)) if v.is_empty() => self.expand(word),
            (":+", Some(v)) if !v.is_empty() => self.expand(word),
            ("+", Some(_)) => self.expand(word),
            (":+", _) | ("+", None) => Some(String::new()),
            (_, None) => None,
            (_, Some(v)) => Some(v),
        }
    }
}

/// Splits an ARG value on unquoted whitespace, removing the quotes.
fn split_words(v: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut in_word = false;
    for c in v.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => cur.push(c),
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                in_word = true;
            }
            None if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut cur));
                    in_word = false;
                }
            }
            None => {
                cur.push(c);
                in_word = true;
            }
        }
    }
    if in_word {
        words.push(cur);
    }
    words
}

fn unquote(v: &str) -> &str {
    // Values are unquoted by `split_words`; this handles `K="v"` pairs split on `=`.
    for q in ['"', '\''] {
        if v.len() >= 2 && v.starts_with(q) && v.ends_with(q) {
            return &v[1..v.len() - 1];
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undeclared_is_unset_globally_and_unknown_in_a_stage() {
        assert_eq!(Vars::global().expand("a$X"), Some("a".to_string()));
        assert_eq!(Vars::stage().expand("a$X"), None);
    }

    #[test]
    fn stage_arg_without_default_takes_the_global_value() {
        let mut g = Vars::global();
        g.record_arg("V", Some("1.0"), None);
        let mut s = Vars::stage();
        s.record_arg("V", None, Some(&g));
        assert_eq!(s.expand("x-${V}"), Some("x-1.0".to_string()));
        let mut s2 = Vars::stage();
        s2.record_arg("W", None, Some(&g));
        assert_eq!(s2.expand("$W"), None);
    }
}
