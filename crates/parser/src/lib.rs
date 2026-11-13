// SPDX-License-Identifier: GPL-3.0-only
//! Port of language-docker's `Language.Docker.Parser*` (v16.0.0).
//!
//! # Parsing approach
//!
//! A hand-written combinator layer (`combinator.rs`) emulates megaparsec: character
//! offsets, `try`/backtracking (a failure after consuming is fatal unless wrapped in
//! `try_`), `label`, megaparsec's *hints*, and an error type
//! ([`combinator::PError`]/[`combinator::ParseError`]) that reproduces
//! `parseErrorTextPretty` / `errorBundlePretty` text exactly (hadolint prints that
//! for DL1000). Each Haskell parser module has a Rust module with the same
//! top-level functions, written as `fn(p: &mut Parser) -> PResult<T>`; the Haskell
//! implicit parameter `?esc` is the field `p.esc`. Haskell `do` blocks become
//! sequential statements with `?`; `<|>` is `p.alt(a, b)`, `<?>` is `p.label(..)`.
//!
//! Entry points: [`parse_text`] (and `parser::parse_file`/`parse_stdin`).

pub mod arguments;
pub mod cmd;
pub mod combinator;
pub mod copy;
pub mod expose;
pub mod from;
pub mod healthcheck;
pub mod instruction;
pub mod pairs;
pub mod parser;
pub mod prelude;
pub mod run;

pub use combinator::{PResult, ParseError, Parser};
pub use parser::{parse_file, parse_stdin, parse_text, parse_text_as};
pub use prelude::DockerfileError;
