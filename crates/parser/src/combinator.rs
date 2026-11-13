// SPDX-License-Identifier: GPL-3.0-only
//! A hand-written emulation of the parts of megaparsec (9.x) that language-docker uses.
//!
//! # Model
//!
//! A parser is any `Fn(&mut Parser) -> PResult<T>`. Sequencing is plain Rust
//! (`let a = p1(p)?; let b = p2(p)?;`), exactly like a Haskell `do` block.
//!
//! * `Parser.offset` is the position in *characters* (megaparsec's `stateOffset`).
//! * A parser has "consumed" input iff `offset` moved. A parser that fails after
//!   consuming (and was not wrapped in [`Parser::try_`]) is a hard failure: [`Parser::alt`]
//!   will not try its second branch. `try_` rewinds the offset, which turns the failure
//!   back into an "empty" one (megaparsec's `cerr -> eerr`).
//! * megaparsec's *hints* (the "expecting ..." items contributed by parsers that
//!   succeeded without consuming, or that stopped at the point of failure) are tracked
//!   in `Parser.hints` with the same rules as megaparsec's `pBind`/`pPlus`/`pLabel`, so
//!   the `expecting` list of a [`PError`] matches `Text.Megaparsec.Error`. All primitives
//!   and combinators in this file implement the hint protocol; user parsers just call them.
//! * [`PError`] is megaparsec's `ParseError`; [`ParseError`] is the
//!   `ParseErrorBundle`. [`PError::text_pretty`] is `parseErrorTextPretty` (what hadolint
//!   prints for DL1000) and [`ParseError::bundle_pretty`] is `errorBundlePretty`.
//!
//! Known limits: `Lexer.float` returns an `f32`; `string ""` counts as empty.

use crate::prelude::DockerfileError;
use std::collections::BTreeSet;
use std::rc::Rc;

/// `Text.Megaparsec.Error.ErrorItem Char`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorItem {
    /// `Tokens (NonEmpty Char)`
    Tokens(Vec<char>),
    /// `Label (NonEmpty Char)`
    Label(String),
    EndOfInput,
}

/// `Text.Megaparsec.Error.ErrorFancy DockerfileError`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorFancy {
    ErrorFail(String),
    ErrorCustom(DockerfileError),
}

/// `Text.Megaparsec.Error.ParseError Text DockerfileError`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PError {
    Trivial {
        offset: usize,
        unexpected: Option<ErrorItem>,
        expected: BTreeSet<ErrorItem>,
    },
    Fancy {
        offset: usize,
        errors: BTreeSet<ErrorFancy>,
    },
}

pub type PResult<T> = Result<T, PError>;
/// megaparsec `Hints` (a list of expected-sets; list order matters for `refreshLastHint`).
pub type Hints = Vec<BTreeSet<ErrorItem>>;

/// `Text.Megaparsec.Pos.SourcePos` (line and column are 1-based).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePos {
    pub name: String,
    pub line: usize,
    pub column: usize,
}

/// A borrowed parser, as taken by [`Parser::choice`].
pub type ParserFn<'a, T> = dyn Fn(&mut Parser) -> PResult<T> + 'a;

impl SourcePos {
    /// `sourcePosPretty`
    pub fn pretty(&self) -> String {
        if self.name.is_empty() {
            format!("{}:{}", self.line, self.column)
        } else {
            format!("{}:{}:{}", self.name, self.line, self.column)
        }
    }
}

const TAB_WIDTH: usize = 8;

impl PError {
    pub fn offset(&self) -> usize {
        match self {
            PError::Trivial { offset, .. } | PError::Fancy { offset, .. } => *offset,
        }
    }

    /// `mergeError e1 e2` (the `Semigroup ParseError` instance).
    pub fn merge(e1: PError, e2: PError) -> PError {
        use std::cmp::Ordering::*;
        match e1.offset().cmp(&e2.offset()) {
            Less => e2,
            Greater => e1,
            Equal => match (e1, e2) {
                (
                    PError::Trivial {
                        offset,
                        unexpected: u1,
                        expected: mut p1,
                    },
                    PError::Trivial {
                        unexpected: u2,
                        expected: p2,
                        ..
                    },
                ) => {
                    p1.extend(p2);
                    let u = match (u1, u2) {
                        (None, None) => None,
                        (Some(x), None) | (None, Some(x)) => Some(x),
                        (Some(x), Some(y)) => Some(x.max(y)),
                    };
                    PError::Trivial {
                        offset,
                        unexpected: u,
                        expected: p1,
                    }
                }
                (e @ PError::Fancy { .. }, PError::Trivial { .. }) => e,
                (PError::Trivial { .. }, e @ PError::Fancy { .. }) => e,
                (
                    PError::Fancy {
                        offset,
                        errors: mut x1,
                    },
                    PError::Fancy { errors: x2, .. },
                ) => {
                    x1.extend(x2);
                    PError::Fancy { offset, errors: x1 }
                }
            },
        }
    }

    /// `parseErrorTextPretty`
    pub fn text_pretty(&self) -> String {
        match self {
            PError::Trivial {
                unexpected,
                expected,
                ..
            } => {
                if unexpected.is_none() && expected.is_empty() {
                    "unknown parse error\n".to_string()
                } else {
                    let us: Vec<String> = unexpected.iter().map(show_error_item).collect();
                    let ps: Vec<String> = expected.iter().map(show_error_item).collect();
                    messages_pretty("unexpected ", &us) + &messages_pretty("expecting ", &ps)
                }
            }
            PError::Fancy { errors, .. } => {
                if errors.is_empty() {
                    "unknown fancy parse error\n".to_string()
                } else {
                    errors.iter().map(|e| show_error_fancy(e) + "\n").collect()
                }
            }
        }
    }

    /// hadolint `Formatter.Format.errorMessage`: `parseErrorTextPretty` without trailing newlines.
    pub fn message(&self) -> String {
        self.text_pretty().trim_end_matches('\n').to_string()
    }

    fn pointer_len(&self) -> usize {
        match self {
            PError::Trivial {
                unexpected: None, ..
            } => 1,
            PError::Trivial {
                unexpected: Some(x),
                ..
            } => match x {
                ErrorItem::Tokens(ts) => ts.len(),
                _ => 1,
            },
            PError::Fancy { .. } => 1, // errorFancyLength is 1 for every error component here
        }
    }
}

fn messages_pretty(prefix: &str, items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }
    let set: BTreeSet<&String> = items.iter().collect();
    let v: Vec<&str> = set.into_iter().map(|s| s.as_str()).collect();
    format!("{}{}\n", prefix, or_list(&v))
}

fn or_list(xs: &[&str]) -> String {
    match xs {
        [x] => x.to_string(),
        [x, y] => format!("{x} or {y}"),
        _ => {
            let (last, init) = xs.split_last().unwrap();
            format!("{}, or {}", init.join(", "), last)
        }
    }
}

fn show_error_item(i: &ErrorItem) -> String {
    match i {
        ErrorItem::Tokens(ts) => string_pretty(ts),
        ErrorItem::Label(l) => l.clone(),
        ErrorItem::EndOfInput => "end of input".to_string(),
    }
}

fn show_error_fancy(e: &ErrorFancy) -> String {
    match e {
        ErrorFancy::ErrorFail(m) => m.clone(),
        ErrorFancy::ErrorCustom(c) => c.show_error_component(),
    }
}

/// `Text.Megaparsec.Error.Builder`-adjacent `stringPretty` (`showTokens` for `Text`).
fn string_pretty(xs: &[char]) -> String {
    match xs {
        [c] => char_pretty(*c),
        ['\r', '\n'] => "crlf newline".to_string(),
        _ => {
            let mut s = String::from("\"");
            for &c in xs {
                match char_pretty_(c) {
                    Some(p) => {
                        s.push('<');
                        s.push_str(p);
                        s.push('>');
                    }
                    None => s.push(c),
                }
            }
            s.push('"');
            s
        }
    }
}

fn char_pretty(c: char) -> String {
    match char_pretty_(c) {
        Some(p) => p.to_string(),
        None => format!("'{c}'"),
    }
}

fn char_pretty_(c: char) -> Option<&'static str> {
    Some(match c {
        '\0' => "null",
        '\u{1}' => "start of heading",
        '\u{2}' => "start of text",
        '\u{3}' => "end of text",
        '\u{4}' => "end of transmission",
        '\u{5}' => "enquiry",
        '\u{6}' => "acknowledge",
        '\u{7}' => "bell",
        '\u{8}' => "backspace",
        '\t' => "tab",
        '\n' => "newline",
        '\u{b}' => "vertical tab",
        '\u{c}' => "form feed",
        '\r' => "carriage return",
        '\u{e}' => "shift out",
        '\u{f}' => "shift in",
        '\u{10}' => "data link escape",
        '\u{11}' => "device control one",
        '\u{12}' => "device control two",
        '\u{13}' => "device control three",
        '\u{14}' => "device control four",
        '\u{15}' => "negative acknowledge",
        '\u{16}' => "synchronous idle",
        '\u{17}' => "end of transmission block",
        '\u{18}' => "cancel",
        '\u{19}' => "end of medium",
        '\u{1a}' => "substitute",
        '\u{1b}' => "escape",
        '\u{1c}' => "file separator",
        '\u{1d}' => "group separator",
        '\u{1e}' => "record separator",
        '\u{1f}' => "unit separator",
        ' ' => "space",
        '\u{7f}' => "delete",
        '\u{a0}' => "non-breaking space",
        _ => return None,
    })
}

/// `ParseErrorBundle Text DockerfileError` (always holds the errors of one `parse` call).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub errors: Vec<PError>,
    pub source_name: String,
    pub input: String,
}

impl ParseError {
    /// The first error (`NE.head bundleErrors`).
    pub fn first(&self) -> &PError {
        &self.errors[0]
    }

    /// hadolint `errorPosition` of the first error.
    pub fn position(&self) -> SourcePos {
        let chars: Vec<char> = self.input.chars().collect();
        let (line, column, _) = reach_offset(&chars, self.first().offset());
        SourcePos {
            name: self.source_name.clone(),
            line,
            column,
        }
    }

    /// hadolint `errorPositionPretty` (`file:line:col`).
    pub fn position_pretty(&self) -> String {
        self.position().pretty()
    }

    /// hadolint `errorMessage`.
    pub fn message(&self) -> String {
        self.first().message()
    }

    /// hadolint `errorMessageLine`.
    pub fn message_line(&self) -> String {
        format!("{} {}", self.position_pretty(), self.first().text_pretty())
    }

    /// `errorBundlePretty` using the stored source name and input.
    pub fn bundle_pretty(&self) -> String {
        self.render(&self.source_name, &self.input)
    }

    /// `errorBundlePretty` for the given file name and source text.
    pub fn render(&self, file_name: &str, src: &str) -> String {
        let chars: Vec<char> = src.chars().collect();
        let mut out = String::new();
        for e in &self.errors {
            let (line, col, sline) = reach_offset(&chars, e.offset());
            let epos = SourcePos {
                name: file_name.to_string(),
                line,
                column: col,
            };
            let line_number = line.to_string();
            let padding = " ".repeat(line_number.len() + 1);
            let rpshift = col - 1;
            let sline_len = sline.chars().count();
            let elen = e.pointer_len();
            let pointer_len = if rpshift + elen > sline_len {
                sline_len - rpshift.min(sline_len) + 1
            } else {
                elen
            };
            let rpadding = if pointer_len > 0 {
                " ".repeat(rpshift)
            } else {
                String::new()
            };
            out.push('\n');
            out.push_str(&epos.pretty());
            out.push_str(":\n");
            out.push_str(&format!(
                "{padding}|\n{line_number} | {sline}\n{padding}| {rpadding}{}\n",
                "^".repeat(pointer_len)
            ));
            out.push_str(&e.text_pretty());
        }
        out.strip_prefix('\n').unwrap_or(&out).to_string()
    }
}

/// `reachOffset` for `Text`: (line, column, offending line with tabs expanded).
fn reach_offset(chars: &[char], offset: usize) -> (usize, usize, String) {
    let offset = offset.min(chars.len());
    let mut line = 1;
    let mut col = 1;
    let mut line_start = 0;
    for (i, &c) in chars[..offset].iter().enumerate() {
        match c {
            '\n' => {
                line += 1;
                col = 1;
                line_start = i + 1;
            }
            '\t' => col += TAB_WIDTH - ((col - 1) % TAB_WIDTH),
            _ => col += 1,
        }
    }
    let mut end = offset;
    while end < chars.len() && chars[end] != '\n' {
        end += 1;
    }
    let raw: String = chars[line_start..end].iter().collect();
    let mut sline = String::new();
    let mut w = 0usize;
    for c in raw.chars() {
        if c == '\t' {
            let n = TAB_WIDTH - (w % TAB_WIDTH);
            sline.push_str(&" ".repeat(n));
            w += n;
        } else {
            sline.push(c);
            w += 1;
        }
    }
    if sline.is_empty() {
        sline = "<empty line>".to_string();
    }
    (line, col, sline)
}

fn with_hints(e: PError, hints: &Hints) -> PError {
    match e {
        PError::Trivial {
            offset,
            unexpected,
            mut expected,
        } => {
            for h in hints {
                expected.extend(h.iter().cloned());
            }
            PError::Trivial {
                offset,
                unexpected,
                expected,
            }
        }
        f => f,
    }
}

/// megaparsec `toHints`
fn to_hints(stream_offset: usize, e: &PError) -> Hints {
    match e {
        PError::Trivial {
            offset, expected, ..
        } if *offset == stream_offset && !expected.is_empty() => {
            vec![expected.clone()]
        }
        _ => vec![],
    }
}

fn concat(mut a: Hints, b: Hints) -> Hints {
    a.extend(b);
    a
}

/// megaparsec 9.7 `refreshHints`: no label clears the hints; a label
/// replaces non-empty hints with itself.
fn refresh_hints(hs: Hints, new: Option<ErrorItem>) -> Hints {
    match new {
        None => Vec::new(),
        Some(_) if hs.iter().all(|h| h.is_empty()) => hs,
        Some(n) => vec![BTreeSet::from([n])],
    }
}

/// The parser state: immutable input plus mutable offset, hints and escape character.
pub struct Parser {
    input: Rc<Vec<char>>,
    /// Offset in characters.
    pub offset: usize,
    hints: Hints,
    /// The `?esc` implicit parameter of language-docker (the escape character).
    pub esc: char,
    pub source_name: String,
    src: Rc<str>,
}

impl Parser {
    /// A parser over `src`, named `source_name` (megaparsec's `parse name src`).
    pub fn new(src: &str, source_name: &str, esc: char) -> Parser {
        Parser {
            input: Rc::new(src.chars().collect()),
            offset: 0,
            hints: vec![],
            esc,
            source_name: source_name.to_string(),
            src: Rc::from(src),
        }
    }

    /// `getOffset`
    pub fn get_offset(&self) -> usize {
        self.offset
    }

    /// `getSourcePos`
    pub fn get_source_pos(&self) -> SourcePos {
        let (line, column, _) = reach_offset(&self.input, self.offset);
        SourcePos {
            name: self.source_name.clone(),
            line,
            column,
        }
    }

    /// Wrap a failure into the error bundle `parse` returns.
    pub fn bundle(&self, e: PError) -> ParseError {
        ParseError {
            errors: vec![e],
            source_name: self.source_name.clone(),
            input: self.src.to_string(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.offset).copied()
    }

    /// Core primitive wrapper implementing the hint protocol for leaf parsers.
    /// `f` inspects the state and returns `(value, chars consumed, own hints)` or an error.
    fn prim<T>(
        &mut self,
        f: impl FnOnce(&Parser) -> Result<(T, usize, Hints), PError>,
    ) -> PResult<T> {
        let inh = std::mem::take(&mut self.hints);
        match f(self) {
            Ok((x, n, own)) => {
                self.offset += n;
                self.hints = if n > 0 { own } else { concat(inh, own) };
                Ok(x)
            }
            Err(e) => Err(with_hints(e, &inh)),
        }
    }

    fn trivial_here(&self, unexpected: Option<ErrorItem>, expected: BTreeSet<ErrorItem>) -> PError {
        PError::Trivial {
            offset: self.offset,
            unexpected,
            expected,
        }
    }

    fn next_item(&self) -> ErrorItem {
        match self.peek() {
            None => ErrorItem::EndOfInput,
            Some(c) => ErrorItem::Tokens(vec![c]),
        }
    }

    // ------------------------------------------------------------ primitives

    /// `eof`
    pub fn eof(&mut self) -> PResult<()> {
        self.prim(|p| match p.peek() {
            None => Ok(((), 0, vec![])),
            Some(c) => Err(p.trivial_here(
                Some(ErrorItem::Tokens(vec![c])),
                BTreeSet::from([ErrorItem::EndOfInput]),
            )),
        })
    }

    /// `satisfy`
    pub fn satisfy(&mut self, f: impl Fn(char) -> bool) -> PResult<char> {
        self.prim(|p| match p.peek() {
            Some(c) if f(c) => Ok((c, 1, vec![])),
            _ => Err(p.trivial_here(Some(p.next_item()), BTreeSet::new())),
        })
    }

    /// `anySingle`
    pub fn any_single(&mut self) -> PResult<char> {
        self.satisfy(|_| true)
    }

    /// `anySingleBut c`
    pub fn any_single_but(&mut self, c: char) -> PResult<char> {
        self.satisfy(move |x| x != c)
    }

    /// `char c`
    pub fn char(&mut self, c: char) -> PResult<char> {
        self.prim(|p| match p.peek() {
            Some(x) if x == c => Ok((c, 1, vec![])),
            _ => Err(p.trivial_here(
                Some(p.next_item()),
                BTreeSet::from([ErrorItem::Tokens(vec![c])]),
            )),
        })
    }

    /// `string s` (`tokens (==)`)
    pub fn string(&mut self, s: &str) -> PResult<String> {
        self.tokens(s, |a, b| a == b)
    }

    /// `string' s` (case-insensitive; returns the text actually matched)
    pub fn string_ci(&mut self, s: &str) -> PResult<String> {
        self.tokens(s, |a, b| a.to_lowercase().eq(b.to_lowercase()))
    }

    fn tokens(&mut self, s: &str, eq: impl Fn(char, char) -> bool) -> PResult<String> {
        let want: Vec<char> = s.chars().collect();
        self.prim(|p| {
            let avail = &p.input[p.offset..];
            let take = want.len().min(avail.len());
            let got = &avail[..take];
            let expected = BTreeSet::from([ErrorItem::Tokens(want.clone())]);
            if want.is_empty() {
                return Ok((String::new(), 0, vec![]));
            }
            if got.is_empty() {
                return Err(p.trivial_here(Some(ErrorItem::EndOfInput), expected));
            }
            if got.len() == want.len() && got.iter().zip(&want).all(|(a, b)| eq(*a, *b)) {
                Ok((got.iter().collect(), want.len(), vec![]))
            } else {
                Err(p.trivial_here(Some(ErrorItem::Tokens(got.to_vec())), expected))
            }
        })
    }

    /// `takeWhileP label f` (never fails; the label becomes a hint)
    pub fn take_while_p(
        &mut self,
        label: Option<&str>,
        f: impl Fn(char) -> bool,
    ) -> PResult<String> {
        self.prim(|p| {
            let n = p.input[p.offset..].iter().take_while(|c| f(**c)).count();
            let text: String = p.input[p.offset..p.offset + n].iter().collect();
            let hs = match label {
                Some(l) if !l.is_empty() => vec![BTreeSet::from([ErrorItem::Label(l.to_string())])],
                _ => vec![],
            };
            Ok((text, n, hs))
        })
    }

    /// `takeWhile1P label f`
    pub fn take_while1_p(
        &mut self,
        label: Option<&str>,
        f: impl Fn(char) -> bool,
    ) -> PResult<String> {
        self.prim(|p| {
            let n = p.input[p.offset..].iter().take_while(|c| f(**c)).count();
            let el = label
                .filter(|l| !l.is_empty())
                .map(|l| ErrorItem::Label(l.to_string()));
            if n == 0 {
                let ps = el.into_iter().collect::<BTreeSet<_>>();
                Err(p.trivial_here(Some(p.next_item()), ps))
            } else {
                let text: String = p.input[p.offset..p.offset + n].iter().collect();
                let hs = el.map(|l| vec![BTreeSet::from([l])]).unwrap_or_default();
                Ok((text, n, hs))
            }
        })
    }

    /// `spaceChar` (`satisfy isSpace <?> "white space"`)
    pub fn space_char(&mut self) -> PResult<char> {
        self.label("white space", |p| p.satisfy(|c| c.is_whitespace()))
    }

    /// `newline` (`char '\n' <?> "newline"`)
    pub fn newline(&mut self) -> PResult<char> {
        self.label("newline", |p| p.char('\n'))
    }

    /// `crlf` (`string "\r\n" <?> "crlf newline"`)
    pub fn crlf(&mut self) -> PResult<String> {
        self.label("crlf newline", |p| p.string("\r\n"))
    }

    /// `space` (`takeWhileP (Just "white space") isSpace`)
    pub fn space(&mut self) -> PResult<()> {
        self.take_while_p(Some("white space"), |c| c.is_whitespace())
            .map(|_| ())
    }

    /// `space1`
    pub fn space1(&mut self) -> PResult<()> {
        self.take_while1_p(Some("white space"), |c| c.is_whitespace())
            .map(|_| ())
    }

    /// `empty` / `mzero`
    pub fn empty<T>(&mut self) -> PResult<T> {
        self.prim(|p| Err(p.trivial_here(None, BTreeSet::new())))
    }

    /// `fail msg`
    pub fn fail<T>(&mut self, msg: &str) -> PResult<T> {
        self.fancy_failure(ErrorFancy::ErrorFail(msg.to_string()))
    }

    /// `customFailure e` / Prelude `customError`
    pub fn custom_failure<T>(&mut self, e: DockerfileError) -> PResult<T> {
        self.fancy_failure(ErrorFancy::ErrorCustom(e))
    }

    fn fancy_failure<T>(&mut self, e: ErrorFancy) -> PResult<T> {
        self.prim(|p| {
            Err(PError::Fancy {
                offset: p.offset,
                errors: BTreeSet::from([e]),
            })
        })
    }

    /// `unexpected item` (`failure (Just item) mempty`)
    pub fn unexpected<T>(&mut self, item: ErrorItem) -> PResult<T> {
        self.prim(|p| Err(p.trivial_here(Some(item), BTreeSet::new())))
    }

    // ------------------------------------------------------------ combinators

    /// `try p`
    pub fn try_<T>(&mut self, f: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
        let inh = std::mem::take(&mut self.hints);
        let start = self.offset;
        match f(self) {
            Ok(x) => {
                if self.offset == start {
                    let own = std::mem::take(&mut self.hints);
                    self.hints = concat(inh, own);
                }
                Ok(x)
            }
            Err(e) => {
                self.offset = start;
                Err(with_hints(e, &inh))
            }
        }
    }

    /// `a <|> b`
    pub fn alt<T>(
        &mut self,
        a: impl Fn(&mut Parser) -> PResult<T>,
        b: impl Fn(&mut Parser) -> PResult<T>,
    ) -> PResult<T> {
        let inh = std::mem::take(&mut self.hints);
        let start = self.offset;
        match a(self) {
            Ok(x) => {
                if self.offset == start {
                    let own = std::mem::take(&mut self.hints);
                    self.hints = concat(inh, own);
                }
                Ok(x)
            }
            Err(e) => {
                if self.offset != start {
                    return Err(e);
                }
                self.hints = vec![];
                match b(self) {
                    Ok(x) => {
                        if self.offset == start {
                            let own = std::mem::take(&mut self.hints);
                            self.hints = concat(concat(inh, to_hints(start, &e)), own);
                        }
                        Ok(x)
                    }
                    Err(e2) => {
                        // megaparsec `pPlus`: `ncerr err' = cerr (err' <> err)`.
                        if self.offset != start {
                            return Err(PError::merge(e2, e));
                        }
                        Err(with_hints(PError::merge(e2, e), &inh))
                    }
                }
            }
        }
    }

    /// `choice [p1, p2, ...]` (`foldr (<|>) empty`)
    pub fn choice<T>(&mut self, ps: &[&ParserFn<T>]) -> PResult<T> {
        match ps {
            [] => self.empty(),
            [p, rest @ ..] => self.alt(|s| p(s), |s| s.choice(rest)),
        }
    }

    /// `p <?> l` / `label l p`
    pub fn label<T>(&mut self, l: &str, f: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
        let inh = std::mem::take(&mut self.hints);
        let start = self.offset;
        let el = if l.is_empty() {
            None
        } else {
            Some(ErrorItem::Label(l.to_string()))
        };
        match f(self) {
            Ok(x) => {
                let own = std::mem::take(&mut self.hints);
                if self.offset == start {
                    self.hints = concat(inh, refresh_hints(own, el));
                } else if el.is_none() {
                    // megaparsec 9.7 `pLabel` `cok'`: a hidden parser drops its hints,
                    // a labelled one keeps them unchanged.
                    self.hints = Vec::new();
                } else {
                    self.hints = own;
                }
                Ok(x)
            }
            Err(e) => {
                if self.offset != start {
                    return Err(e);
                }
                let e = match e {
                    PError::Trivial {
                        offset, unexpected, ..
                    } => PError::Trivial {
                        offset,
                        unexpected,
                        expected: el.into_iter().collect(),
                    },
                    f => f,
                };
                Err(with_hints(e, &inh))
            }
        }
    }

    /// `hidden p` (`label "" p`)
    pub fn hidden<T>(&mut self, f: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
        self.label("", f)
    }

    /// `optional p`
    pub fn optional<T>(&mut self, f: impl Fn(&mut Parser) -> PResult<T>) -> PResult<Option<T>> {
        self.alt(|p| f(p).map(Some), |_| Ok(None))
    }

    /// `option x p`
    pub fn option<T: Clone>(&mut self, x: T, f: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
        self.alt(|p| f(p), |_| Ok(x.clone()))
    }

    /// `many p`
    pub fn many<T>(&mut self, f: impl Fn(&mut Parser) -> PResult<T>) -> PResult<Vec<T>> {
        let mut v = vec![];
        loop {
            let before = self.offset;
            match self.optional(&f)? {
                Some(x) => {
                    v.push(x);
                    if self.offset == before {
                        break; // megaparsec would loop forever on an empty-succeeding parser
                    }
                }
                None => break,
            }
        }
        Ok(v)
    }

    /// `some p`
    pub fn some<T>(&mut self, f: impl Fn(&mut Parser) -> PResult<T>) -> PResult<Vec<T>> {
        let first = f(self)?;
        let mut v = vec![first];
        v.extend(self.many(f)?);
        Ok(v)
    }

    /// `skipMany p`
    pub fn skip_many<T>(&mut self, f: impl Fn(&mut Parser) -> PResult<T>) -> PResult<()> {
        self.many(f).map(|_| ())
    }

    /// `sepBy1 p sep`
    pub fn sep_by1<T, S>(
        &mut self,
        p: impl Fn(&mut Parser) -> PResult<T>,
        sep: impl Fn(&mut Parser) -> PResult<S>,
    ) -> PResult<Vec<T>> {
        let first = p(self)?;
        let mut v = vec![first];
        let rest = self.many(|s| {
            sep(s)?;
            p(s)
        })?;
        v.extend(rest);
        Ok(v)
    }

    /// `sepBy p sep`
    pub fn sep_by<T, S>(
        &mut self,
        p: impl Fn(&mut Parser) -> PResult<T>,
        sep: impl Fn(&mut Parser) -> PResult<S>,
    ) -> PResult<Vec<T>> {
        self.alt(|s| s.sep_by1(&p, &sep), |_| Ok(vec![]))
    }

    /// `sepEndBy1 p sep`
    pub fn sep_end_by1<T, S>(
        &mut self,
        p: impl Fn(&mut Parser) -> PResult<T>,
        sep: impl Fn(&mut Parser) -> PResult<S>,
    ) -> PResult<Vec<T>> {
        let mut v = vec![p(self)?];
        loop {
            let step = self.alt(
                |s| {
                    sep(s)?;
                    s.optional(&p).map(Some)
                },
                |_| Ok(None),
            )?;
            match step {
                Some(Some(x)) => v.push(x),
                _ => break,
            }
        }
        Ok(v)
    }

    /// `sepEndBy p sep`
    pub fn sep_end_by<T, S>(
        &mut self,
        p: impl Fn(&mut Parser) -> PResult<T>,
        sep: impl Fn(&mut Parser) -> PResult<S>,
    ) -> PResult<Vec<T>> {
        self.alt(|s| s.sep_end_by1(&p, &sep), |_| Ok(vec![]))
    }

    /// `manyTill p end`
    pub fn many_till<T, E>(
        &mut self,
        p: impl Fn(&mut Parser) -> PResult<T>,
        end: impl Fn(&mut Parser) -> PResult<E>,
    ) -> PResult<Vec<T>> {
        let mut v = vec![];
        loop {
            let r = self.alt(|s| end(s).map(|_| None), |s| p(s).map(Some))?;
            match r {
                None => break,
                Some(x) => v.push(x),
            }
        }
        Ok(v)
    }

    /// `between open close p`
    pub fn between<O, C, T>(
        &mut self,
        open: impl Fn(&mut Parser) -> PResult<O>,
        close: impl Fn(&mut Parser) -> PResult<C>,
        p: impl Fn(&mut Parser) -> PResult<T>,
    ) -> PResult<T> {
        open(self)?;
        let x = p(self)?;
        close(self)?;
        Ok(x)
    }

    /// `notFollowedBy p`
    pub fn not_followed_by<T>(&mut self, f: impl Fn(&mut Parser) -> PResult<T>) -> PResult<()> {
        let inh = std::mem::take(&mut self.hints);
        let start = self.offset;
        let what = self.next_item();
        let saved = std::mem::take(&mut self.hints);
        let r = f(self);
        self.offset = start;
        self.hints = saved;
        match r {
            Ok(_) => Err(with_hints(
                self.trivial_here(Some(what), BTreeSet::new()),
                &inh,
            )),
            Err(_) => {
                self.hints = inh;
                Ok(())
            }
        }
    }

    /// `lookAhead p`
    pub fn look_ahead<T>(&mut self, f: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
        let inh = std::mem::take(&mut self.hints);
        let start = self.offset;
        match f(self) {
            Ok(x) => {
                self.offset = start;
                self.hints = inh;
                Ok(x)
            }
            Err(e) => {
                if self.offset == start {
                    Err(with_hints(e, &inh))
                } else {
                    Err(e)
                }
            }
        }
    }

    /// `observing p`
    pub fn observing<T>(
        &mut self,
        f: impl Fn(&mut Parser) -> PResult<T>,
    ) -> PResult<Result<T, PError>> {
        let inh = std::mem::take(&mut self.hints);
        let start = self.offset;
        match f(self) {
            Ok(x) => {
                if self.offset == start {
                    let own = std::mem::take(&mut self.hints);
                    self.hints = concat(inh, own);
                }
                Ok(Ok(x))
            }
            Err(e) => {
                if self.offset != start {
                    self.hints = vec![];
                    Ok(Err(e))
                } else {
                    let h = to_hints(self.offset, &e);
                    self.hints = concat(inh, h);
                    Ok(Err(e))
                }
            }
        }
    }

    /// `count n p`
    pub fn count<T>(&mut self, n: usize, f: impl Fn(&mut Parser) -> PResult<T>) -> PResult<Vec<T>> {
        let mut v = vec![];
        for _ in 0..n {
            v.push(f(self)?);
        }
        Ok(v)
    }

    // ------------------------------------------------------------ Text.Megaparsec.Char.Lexer

    /// `L.decimal` (`decimal_ <?> "integer"`; digits are ASCII 0-9)
    pub fn decimal(&mut self) -> PResult<u128> {
        self.label("integer", |p| p.decimal_())
    }

    fn decimal_(&mut self) -> PResult<u128> {
        let s = self.take_while1_p(Some("digit"), |c| c.is_ascii_digit())?;
        Ok(s.chars().fold(0u128, |a, c| {
            a.wrapping_mul(10)
                .wrapping_add(c.to_digit(10).unwrap() as u128)
        }))
    }

    /// `L.float` at type `Float` (label "floating point number"): digits followed by a
    /// fractional part and/or an exponent.
    pub fn float(&mut self) -> PResult<f32> {
        self.label("floating point number", |p| {
            let int = p.take_while1_p(Some("digit"), |c| c.is_ascii_digit())?;
            let frac_branch = |s: &mut Parser| -> PResult<String> {
                s.char('.')?;
                let d = s.take_while1_p(Some("digit"), |c| c.is_ascii_digit())?;
                let e = s.option(String::new(), |s| s.try_(|s| s.exponent_text()))?;
                Ok(format!(".{d}{e}"))
            };
            let exp_branch = |s: &mut Parser| -> PResult<String> { s.exponent_text() };
            let tail = p.alt(frac_branch, exp_branch)?;
            Ok(format!("{int}{tail}").parse::<f32>().unwrap_or(0.0))
        })
    }

    fn exponent_text(&mut self) -> PResult<String> {
        self.satisfy(|c| c == 'e' || c == 'E')?;
        let sign = self.option(String::new(), |s| {
            s.satisfy(|c| c == '-' || c == '+').map(|c| {
                if c == '-' {
                    "-".to_string()
                } else {
                    String::new()
                }
            })
        })?;
        let d = self.take_while1_p(Some("digit"), |c| c.is_ascii_digit())?;
        Ok(format!("e{sign}{d}"))
    }

    /// `L.charLiteral` (Haskell `readLitChar` escapes), label "literal character".
    pub fn char_literal(&mut self) -> PResult<char> {
        self.label("literal character", |p| {
            let start = p.offset;
            let ahead: Vec<char> = p.input[start..].iter().take(10).copied().collect();
            if ahead.is_empty() {
                return p
                    .prim(|s| Err(s.trivial_here(Some(ErrorItem::EndOfInput), BTreeSet::new())));
            }
            match read_lit_char(&ahead) {
                Some((c, n)) => p.prim(|_| Ok((c, n, vec![]))),
                None => {
                    let c = ahead[0];
                    p.prim(|s| {
                        Err(s.trivial_here(Some(ErrorItem::Tokens(vec![c])), BTreeSet::new()))
                    })
                }
            }
        })
    }
}

/// Haskell `Data.Char.readLitChar`: returns the character and how many chars were read.
fn read_lit_char(s: &[char]) -> Option<(char, usize)> {
    if s.first() != Some(&'\\') {
        return s.first().map(|c| (*c, 1));
    }
    let rest = &s[1..];
    let c = *rest.first()?;
    let simple = match c {
        'a' => Some('\u{7}'),
        'b' => Some('\u{8}'),
        'f' => Some('\u{c}'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        'v' => Some('\u{b}'),
        '\\' => Some('\\'),
        '"' => Some('"'),
        '\'' => Some('\''),
        _ => None,
    };
    if let Some(ch) = simple {
        return Some((ch, 2));
    }
    if c == '^' {
        let d = *rest.get(1)?;
        if ('@'..='_').contains(&d) {
            return char::from_u32(d as u32 - 64).map(|ch| (ch, 3));
        }
        return None;
    }
    let num = |radix: u32, skip: usize| -> Option<(char, usize)> {
        let digits: String = rest[skip..]
            .iter()
            .take_while(|c| c.is_digit(radix))
            .collect();
        if digits.is_empty() {
            return None;
        }
        let v = u32::from_str_radix(&digits, radix).ok()?;
        char::from_u32(v).map(|ch| (ch, 1 + skip + digits.len()))
    };
    if c.is_ascii_digit() {
        return num(10, 0);
    }
    if c == 'o' {
        return num(8, 1);
    }
    if c == 'x' {
        return num(16, 1);
    }
    const NAMES: [(&str, u32); 34] = [
        ("NUL", 0),
        ("SOH", 1),
        ("STX", 2),
        ("ETX", 3),
        ("EOT", 4),
        ("ENQ", 5),
        ("ACK", 6),
        ("BEL", 7),
        ("BS", 8),
        ("HT", 9),
        ("LF", 10),
        ("VT", 11),
        ("FF", 12),
        ("CR", 13),
        ("SO", 14),
        ("SI", 15),
        ("DLE", 16),
        ("DC1", 17),
        ("DC2", 18),
        ("DC3", 19),
        ("DC4", 20),
        ("NAK", 21),
        ("SYN", 22),
        ("ETB", 23),
        ("CAN", 24),
        ("EM", 25),
        ("SUB", 26),
        ("ESC", 27),
        ("FS", 28),
        ("GS", 29),
        ("RS", 30),
        ("US", 31),
        ("SP", 32),
        ("DEL", 127),
    ];
    let text: String = rest.iter().collect();
    let mut best: Option<(&str, u32)> = None;
    for (n, v) in NAMES {
        if text.starts_with(n) && best.is_none_or(|(b, _)| n.len() > b.len()) {
            best = Some((n, v));
        }
    }
    best.and_then(|(n, v)| char::from_u32(v).map(|ch| (ch, 1 + n.len())))
}

/// megaparsec `parse p name input` followed by "must consume all"? No: plain `parse`
/// (use `eof` inside `p` if needed). Returns the value or the error bundle.
pub fn run_parser<T>(
    src: &str,
    source_name: &str,
    esc: char,
    p: impl Fn(&mut Parser) -> PResult<T>,
) -> Result<T, ParseError> {
    let mut st = Parser::new(src, source_name, esc);
    match p(&mut st) {
        Ok(x) => Ok(x),
        Err(e) => Err(st.bundle(e)),
    }
}

#[cfg(test)]
mod combinator_tests {
    use super::*;

    fn msg<T: std::fmt::Debug>(src: &str, p: impl Fn(&mut Parser) -> PResult<T>) -> String {
        match run_parser(src, "<string>", '\\', p) {
            Ok(x) => format!("ok {x:?}"),
            Err(e) => e.message(),
        }
    }

    /// megaparsec 9.7 `pLabel`: after consuming input a label keeps the inner
    /// hints as they are and adds no "the rest of ..." item.
    #[test]
    fn label_after_consuming_keeps_hints() {
        let p = |p: &mut Parser| {
            p.label("ab", |p| {
                p.char('a')?;
                p.optional(|p| p.char('b'))
            })?;
            p.eof()
        };
        assert_eq!(msg("ax", p), "unexpected 'x'\nexpecting 'b' or end of input");
    }

    #[test]
    fn string_and_eof() {
        assert_eq!(msg("abc", |p| p.string("abc")), "ok \"abc\"");
        assert_eq!(
            msg("abd", |p| p.string("abc")),
            "unexpected \"abd\"\nexpecting \"abc\""
        );
        assert_eq!(
            msg("", |p| p.string("abc")),
            "unexpected end of input\nexpecting \"abc\""
        );
        assert_eq!(
            msg("x", |p| p.eof()),
            "unexpected 'x'\nexpecting end of input"
        );
    }

    #[test]
    fn alt_merges_expected_and_try_backtracks() {
        // consumed failure: second branch not tried
        let r = msg("ab", |p| p.alt(|p| p.string("ac"), |p| p.string("ab")));
        assert_eq!(
            r, "ok \"ab\"",
            "string fails empty, so alt tries the other branch"
        );
        let r = msg("ab", |p| {
            p.alt(
                |p| {
                    p.char('a')?;
                    p.char('c').map(|c| c.to_string())
                },
                |p| p.string("ab"),
            )
        });
        assert_eq!(r, "unexpected 'b'\nexpecting 'c'");
        let r = msg("ab", |p| {
            p.alt(
                |p| {
                    p.try_(|p| {
                        p.char('a')?;
                        p.char('c').map(|c| c.to_string())
                    })
                },
                |p| p.string("ab"),
            )
        });
        assert_eq!(r, "ok \"ab\"");
        let r = msg("z", |p| p.alt(|p| p.char('a'), |p| p.char('b')));
        assert_eq!(r, "unexpected 'z'\nexpecting 'a' or 'b'");
    }

    #[test]
    fn labels_and_hints() {
        let r = msg("1", |p| {
            p.label("a letter", |p| p.satisfy(|c| c.is_alphabetic()))
        });
        assert_eq!(r, "unexpected '1'\nexpecting a letter");
        // hints from a successful empty `many` are added to the following failure
        let r = msg("1", |p| {
            p.many(|p| p.char('a'))?;
            p.char('b')
        });
        assert_eq!(r, "unexpected '1'\nexpecting 'a' or 'b'");
        // hints from takeWhileP with a label
        let r = msg("1", |p| {
            p.take_while_p(Some("spaces"), |c| c == ' ')?;
            p.eof()
        });
        assert_eq!(r, "unexpected '1'\nexpecting end of input or spaces");
    }

    #[test]
    fn many_some_sep() {
        assert_eq!(
            msg("aaab", |p| p.many(|p| p.char('a'))),
            "ok ['a', 'a', 'a']"
        );
        assert_eq!(
            msg("a,a,", |p| p.sep_end_by(|p| p.char('a'), |p| p.char(','))),
            "ok ['a', 'a']"
        );
        assert_eq!(
            msg("xxy", |p| p.many_till(|p| p.any_single(), |p| p.char('y'))),
            "ok ['x', 'x']"
        );
        assert_eq!(
            msg("", |p| p.some(|p| p.char('a'))),
            "unexpected end of input\nexpecting 'a'"
        );
    }

    #[test]
    fn not_followed_by_and_look_ahead() {
        assert_eq!(
            msg("ab", |p| {
                p.not_followed_by(|p| p.char('b'))?;
                p.char('a')
            }),
            "ok 'a'"
        );
        assert_eq!(
            msg("ab", |p| p.not_followed_by(|p| p.char('a'))),
            "unexpected 'a'"
        );
        assert_eq!(
            msg("ab", |p| {
                p.look_ahead(|p| p.char('a'))?;
                p.string("ab")
            }),
            "ok \"ab\""
        );
    }

    #[test]
    fn fancy_errors_and_positions() {
        let e = run_parser("a\n  b", "f", '\\', |p| {
            p.string("a\n  ")?;
            p.fail::<()>("boom")
        })
        .unwrap_err();
        assert_eq!(e.message(), "boom");
        assert_eq!(e.position_pretty(), "f:2:3");
        assert_eq!(e.bundle_pretty(), "f:2:3:\n  |\n2 |   b\n  |   ^\nboom\n");
    }

    #[test]
    fn bundle_pretty_trivial() {
        let e = run_parser("ab", "<string>", '\\', |p| p.string("abc")).unwrap_err();
        assert_eq!(
            e.bundle_pretty(),
            "<string>:1:1:\n  |\n1 | ab\n  | ^^\nunexpected \"ab\"\nexpecting \"abc\"\n"
        );
    }

    #[test]
    fn lexer() {
        assert_eq!(msg("12x", |p| p.decimal()), "ok 12");
        assert_eq!(
            msg("x", |p| p.decimal()),
            "unexpected 'x'\nexpecting integer"
        );
        assert_eq!(msg("1.5s", |p| p.float()), "ok 1.5");
        assert_eq!(msg("\\n", |p| p.char_literal()), "ok '\\n'");
        assert_eq!(msg("a", |p| p.char_literal()), "ok 'a'");
        assert_eq!(msg("\\x41", |p| p.char_literal()), "ok 'A'");
    }
}
