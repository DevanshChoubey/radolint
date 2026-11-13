// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule.hs.
//!
//! Haskell `type Rule args = Foldl.Fold (InstructionPos args) Failures` is modelled as
//! the trait [`Rule`] (a mutable fold: `step` for each instruction, `done` at the end).
//! The `Monoid` instance of `Fold` (`<>`, `mconcat`) is [`combine`]/[`mconcat`]:
//! every sub-rule sees every instruction, and the failures are concatenated in rule order.
//! Rules are generic over `args` upstream; here they are all over `ParsedShell`.

use crate::shell::ParsedShell;
use dlint_syntax::*;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

/// Haskell `DLSeverity` (derived `Ord`: Error < Warning < Info < Style < Ignore).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DLSeverity {
    DLErrorC,
    DLWarningC,
    DLInfoC,
    DLStyleC,
    DLIgnoreC,
}

impl Default for DLSeverity {
    /// `instance Default DLSeverity` (`DLInfoC`).
    fn default() -> Self {
        DLSeverity::DLInfoC
    }
}

impl DLSeverity {
    /// `instance Pretty DLSeverity`
    pub fn pretty(&self) -> &'static str {
        match self {
            DLSeverity::DLErrorC => "error",
            DLSeverity::DLWarningC => "warning",
            DLSeverity::DLInfoC => "info",
            DLSeverity::DLStyleC => "style",
            DLSeverity::DLIgnoreC => "ignore",
        }
    }
    /// `instance Monoid DLSeverity` `mempty`.
    pub fn mempty() -> Self {
        DLSeverity::DLIgnoreC
    }
    /// `instance Semigroup DLSeverity`: `_ <> s2 = s2`.
    pub fn append(self, s2: DLSeverity) -> DLSeverity {
        s2
    }
}

impl<'de> Deserialize<'de> for DLSeverity {
    /// `instance FromYAML DLSeverity` (`withSeverity`): a string scalar naming a severity.
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        read_either_severity(&s).map_err(serde::de::Error::custom)
    }
}

/// Haskell `readEitherSeverity`.
pub fn read_either_severity(t: &str) -> Result<DLSeverity, String> {
    read_maybe_severity(t).ok_or_else(|| format!("Invalid severity: {t}"))
}

/// Haskell `readMaybeSeverity`.
pub fn read_maybe_severity(t: &str) -> Option<DLSeverity> {
    match t {
        "error" => Some(DLSeverity::DLErrorC),
        "warning" => Some(DLSeverity::DLWarningC),
        "info" => Some(DLSeverity::DLInfoC),
        "style" => Some(DLSeverity::DLStyleC),
        "ignore" => Some(DLSeverity::DLIgnoreC),
        "none" => Some(DLSeverity::DLIgnoreC),
        _ => None,
    }
}

/// Haskell `newtype RuleCode` (field `unRuleCode`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleCode {
    pub un_rule_code: String,
}

impl RuleCode {
    /// `instance Show RuleCode`: `show (unRuleCode rc)`, i.e. the code in double quotes.
    pub fn show(&self) -> String {
        format!("{:?}", self.un_rule_code)
    }
    /// `instance Pretty RuleCode`: `pretty (show rc)` (also quoted).
    pub fn pretty(&self) -> String {
        self.show()
    }
}

impl From<&str> for RuleCode {
    /// `instance IsString RuleCode`
    fn from(s: &str) -> Self {
        RuleCode { un_rule_code: s.to_string() }
    }
}
impl From<String> for RuleCode {
    fn from(s: String) -> Self {
        RuleCode { un_rule_code: s }
    }
}

/// Haskell `CheckFailure`. `Eq` is structural; `Ord` compares only `line` (as upstream).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckFailure {
    pub code: RuleCode,
    pub severity: DLSeverity,
    pub message: String,
    pub line: Linenumber,
}

impl PartialOrd for CheckFailure {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CheckFailure {
    /// `a `compare` b = line a `compare` line b`
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.line.cmp(&other.line)
    }
}

/// Haskell `type Failures = Seq CheckFailure`.
pub type Failures = Vec<CheckFailure>;

/// Haskell `data State a`.
#[derive(Debug, Clone)]
pub struct State<A> {
    pub failures: Failures,
    pub state: A,
}

/// Haskell `type LabelName`.
pub type LabelName = String;

/// Haskell `data LabelType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelType {
    Email,
    GitHash,
    RawText,
    Rfc3339,
    SemVer,
    Spdx,
    Url,
}

impl LabelType {
    /// `instance Pretty LabelType`
    pub fn pretty(&self) -> &'static str {
        match self {
            LabelType::RawText => "text",
            LabelType::Url => "url",
            LabelType::Spdx => "spdx",
            LabelType::GitHash => "hash",
            LabelType::Rfc3339 => "rfc3339",
            LabelType::SemVer => "semver",
            LabelType::Email => "email",
        }
    }
}

impl<'de> Deserialize<'de> for LabelType {
    /// `instance FromYAML LabelType` (`withLabelType`).
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        read_either_label_type(&s).map_err(serde::de::Error::custom)
    }
}

/// Haskell `readEitherLabelType`.
pub fn read_either_label_type(t: &str) -> Result<LabelType, String> {
    match t {
        "email" => Ok(LabelType::Email),
        "hash" => Ok(LabelType::GitHash),
        "text" => Ok(LabelType::RawText),
        "rfc3339" => Ok(LabelType::Rfc3339),
        "semver" => Ok(LabelType::SemVer),
        "spdx" => Ok(LabelType::Spdx),
        "url" => Ok(LabelType::Url),
        "" => Ok(LabelType::RawText),
        _ => Err(format!("invalid label type: {t}")),
    }
}

/// Haskell `type LabelSchema = Map LabelName LabelType` (ordered by key).
pub type LabelSchema = BTreeMap<LabelName, LabelType>;

/// Haskell `withLineNumber`.
pub fn with_line_number<T1, T2>(
    f: impl Fn(Linenumber, T1, &Instruction<ParsedShell>) -> T2,
    state: T1,
    ip: &InstructionPos<ParsedShell>,
) -> T2 {
    f(ip.line_number, state, &ip.instruction)
}

/// Haskell `addFail`.
pub fn add_fail<A>(failure: CheckFailure, mut state: State<A>) -> State<A> {
    state.failures.push(failure);
    state
}

/// Haskell `emptyState`.
pub fn empty_state<A>(a: A) -> State<A> {
    State { failures: vec![], state: a }
}

/// Haskell `simpleState`.
pub fn simple_state() -> State<()> {
    State { failures: vec![], state: () }
}

/// Haskell `modify`.
pub fn modify<A>(f: impl FnOnce(A) -> A, s: State<A>) -> State<A> {
    State { failures: s.failures, state: f(s.state) }
}

/// Haskell `replaceWith`.
pub fn replace_with<A>(new_state: A, s: State<A>) -> State<A> {
    State { failures: s.failures, state: new_state }
}

/// Haskell `type Rule args = Fold (InstructionPos args) Failures`.
pub trait Rule {
    /// Feed one instruction.
    fn step(&mut self, ip: &InstructionPos<ParsedShell>);
    /// Finish the fold and return the failures.
    fn done(self: Box<Self>) -> Failures;
}

/// A rule built from a step function and a done callback (`Foldl.Fold`).
struct FoldRule<A> {
    state: Option<State<A>>,
    step: Box<dyn Fn(Linenumber, State<A>, &Instruction<ParsedShell>) -> State<A>>,
    done: Box<dyn Fn(State<A>) -> Failures>,
}

impl<A> Rule for FoldRule<A> {
    fn step(&mut self, ip: &InstructionPos<ParsedShell>) {
        let s = self.state.take().expect("rule state");
        self.state = Some((self.step)(ip.line_number, s, &ip.instruction));
    }
    fn done(self: Box<Self>) -> Failures {
        let this = *self;
        (this.done)(this.state.expect("rule state"))
    }
}

/// A rule that produces no failures (placeholder used by not-yet-ported rules).
pub fn no_failures() -> Box<dyn Rule> {
    mconcat(vec![])
}

/// Haskell `simpleRule`: a rule returning True or False for each instruction.
pub fn simple_rule(
    code: &str,
    severity: DLSeverity,
    message: &str,
    checker: impl Fn(&Instruction<ParsedShell>) -> bool + 'static,
) -> Box<dyn Rule> {
    let code = RuleCode::from(code);
    let message = message.to_string();
    custom_rule(
        move |line, s: State<()>, instr| {
            if checker(instr) {
                s
            } else {
                add_fail(CheckFailure { code: code.clone(), severity, message: message.clone(), line }, s)
            }
        },
        simple_state(),
    )
}

/// Haskell `customRule`: `veryCustomRule step initial failures`.
pub fn custom_rule<A: 'static>(
    step: impl Fn(Linenumber, State<A>, &Instruction<ParsedShell>) -> State<A> + 'static,
    initial: State<A>,
) -> Box<dyn Rule> {
    very_custom_rule(step, initial, |s| s.failures)
}

/// Haskell `veryCustomRule`: like `customRule` plus a `done` callback run on the final state.
pub fn very_custom_rule<A: 'static>(
    step: impl Fn(Linenumber, State<A>, &Instruction<ParsedShell>) -> State<A> + 'static,
    initial: State<A>,
    done: impl Fn(State<A>) -> Failures + 'static,
) -> Box<dyn Rule> {
    Box::new(FoldRule { state: Some(initial), step: Box::new(step), done: Box::new(done) })
}

struct Combined(Vec<Box<dyn Rule>>);

impl Rule for Combined {
    fn step(&mut self, ip: &InstructionPos<ParsedShell>) {
        for r in self.0.iter_mut() {
            r.step(ip);
        }
    }
    fn done(self: Box<Self>) -> Failures {
        let mut out = vec![];
        for r in self.0 {
            out.extend(r.done());
        }
        out
    }
}

/// Haskell `mconcat` of rules (`<>` on `Fold`): failures concatenated in rule order.
pub fn mconcat(rules: Vec<Box<dyn Rule>>) -> Box<dyn Rule> {
    Box::new(Combined(rules))
}

/// Haskell `a <> b` on rules.
pub fn combine(a: Box<dyn Rule>, b: Box<dyn Rule>) -> Box<dyn Rule> {
    mconcat(vec![a, b])
}

struct OnBuild(Box<dyn Rule>);

impl Rule for OnBuild {
    fn step(&mut self, ip: &InstructionPos<ParsedShell>) {
        if let Instruction::OnBuild(inner) = &ip.instruction {
            let unwrapped = InstructionPos {
                instruction: (**inner).clone(),
                sourcename: ip.sourcename.clone(),
                line_number: ip.line_number,
            };
            self.0.step(&unwrapped);
        }
    }
    fn done(self: Box<Self>) -> Failures {
        self.0.done()
    }
}

/// Haskell `onbuild`: unwraps ONBUILD instructions and applies the rule to the content.
pub fn onbuild(rule: Box<dyn Rule>) -> Box<dyn Rule> {
    Box::new(OnBuild(rule))
}

/// Haskell `foldArguments`.
pub fn fold_arguments<A, B>(apply_rule: impl FnOnce(&A) -> B, args: &Arguments<A>) -> B {
    match args {
        Arguments::ArgumentsText(a) => apply_rule(a),
        Arguments::ArgumentsList(a) => apply_rule(a),
    }
}

/// Haskell `aliasMustBe`: the check on the stage alias of a FROM; `true` otherwise.
pub fn alias_must_be<A>(predicate: impl Fn(&str) -> bool, from_instr: &Instruction<A>) -> bool {
    match from_instr {
        Instruction::From(BaseImage { alias: Some(ImageAlias { un_image_alias }), .. }) => {
            predicate(un_image_alias)
        }
        _ => true,
    }
}

/// Haskell `archiveFileFormatExtensions`.
pub fn archive_file_format_extensions() -> &'static [&'static str] {
    &[
        ".tar", ".Z", ".bz2", ".gz", ".lz", ".lzma", ".tZ", ".tb2", ".tbz", ".tbz2", ".tgz", ".tlz",
        ".tpz", ".txz", ".xz",
    ]
}

/// Haskell `dropQuotes` (`Text.dropAround` of `"` and `'`).
pub fn drop_quotes(t: &str) -> String {
    t.trim_matches(|c| c == '"' || c == '\'').to_string()
}

/// Haskell `|>` (pipe-forward).
pub fn pipe<A, B>(x: A, f: impl FnOnce(A) -> B) -> B {
    f(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(i: Instruction<ParsedShell>, line: i64) -> InstructionPos<ParsedShell> {
        InstructionPos { instruction: i, sourcename: "<string>".into(), line_number: line }
    }

    #[test]
    fn simple_rule_and_combine_order() {
        let a = simple_rule("DL0001", DLSeverity::DLInfoC, "a", |i| !matches!(i, Instruction::User(_)));
        let b = simple_rule("DL0002", DLSeverity::DLInfoC, "b", |_| false);
        let mut r = combine(a, b);
        r.step(&ip(Instruction::User("x".into()), 1));
        r.step(&ip(Instruction::User("y".into()), 2));
        let codes: Vec<(String, i64)> =
            r.done().into_iter().map(|f| (f.code.un_rule_code, f.line)).collect();
        // rule A's failures first, then rule B's (Fold `<>` concatenates per rule)
        assert_eq!(
            codes,
            vec![("DL0001".into(), 1), ("DL0001".into(), 2), ("DL0002".into(), 1), ("DL0002".into(), 2)]
        );
    }

    #[test]
    fn onbuild_unwraps() {
        let r = simple_rule("DL0001", DLSeverity::DLInfoC, "a", |i| !matches!(i, Instruction::User(_)));
        let mut r = onbuild(r);
        r.step(&ip(Instruction::User("x".into()), 1));
        r.step(&ip(Instruction::OnBuild(Box::new(Instruction::User("x".into()))), 2));
        assert_eq!(r.done().len(), 1);
    }

    #[test]
    fn helpers() {
        assert_eq!(drop_quotes("\"'a'\""), "a");
        assert_eq!(read_maybe_severity("none"), Some(DLSeverity::DLIgnoreC));
        assert_eq!(RuleCode::from("DL3000").show(), "\"DL3000\"");
    }
}
