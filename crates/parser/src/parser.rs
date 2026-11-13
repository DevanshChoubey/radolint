// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser.hs (and the entry
//! points of Language/Docker.hs).
//!
//! The functions here are fully implemented; the parsers they call
//! (`parseInstruction`, `parseComment`, `eol`, ...) are stubs until ported.

use crate::combinator::{run_parser, PResult, ParseError, Parser};
use crate::instruction::{parse_comment, parse_instruction};
use crate::prelude::{eol, only_whitespaces};
use dlint_syntax::*;

/// Haskell `contents`.
pub fn contents<T>(p: &mut Parser, inner: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
    only_whitespaces(p)?;
    let r = inner(p)?;
    p.eof()?;
    Ok(r)
}

/// Haskell `dockerfile`.
pub fn dockerfile(p: &mut Parser) -> PResult<Vec<InstructionPos<String>>> {
    p.many(|p| {
        let pos = p.get_source_pos();
        let i = parse_instruction(p)?;
        p.label("a new line followed by the next instruction", |p| {
            p.alt(|p| eol(p), |p| p.eof())
        })?;
        Ok(InstructionPos { instruction: i, sourcename: pos.name, line_number: pos.line as i64 })
    })
}

/// Haskell `parseText`.
pub fn parse_text(src: &str) -> Result<Vec<InstructionPos<String>>, ParseError> {
    parse_text_as(src, "<string>")
}

/// `parseText` with an explicit source name (e.g. "/dev/stdin", as `parseStdin` reports).
pub fn parse_text_as(src: &str, name: &str) -> Result<Vec<InstructionPos<String>>, ParseError> {
    let txt = dos2unix(src);
    let lines: Vec<&str> = haskell_lines(&txt);
    let esc = find_escape_pragma(&lines);
    run_parser(&txt, name, esc, |p| contents(p, dockerfile))
}

/// Haskell `parseFile`.
pub fn parse_file(file: &str) -> std::io::Result<Result<Vec<InstructionPos<String>>, ParseError>> {
    let bytes = std::fs::read(file)?;
    Ok(do_parse(file, &bytes))
}

/// Haskell `parseStdin`.
pub fn parse_stdin() -> std::io::Result<Result<Vec<InstructionPos<String>>, ParseError>> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::io::stdin().read_to_end(&mut bytes)?;
    Ok(do_parse("/dev/stdin", &bytes))
}

/// Haskell `findEscapePragma`.
pub fn find_escape_pragma(lines: &[&str]) -> char {
    for l in lines {
        let r = run_parser(l, "<line>", DEFAULT_ESC, |p| contents(p, parse_comment));
        match r {
            Err(_) => return DEFAULT_ESC,
            Ok(Instruction::Pragma(PragmaDirective::Escape(EscapeChar { escape }))) => return escape,
            Ok(Instruction::Pragma(_)) => continue,
            Ok(_) => return DEFAULT_ESC,
        }
    }
    DEFAULT_ESC
}

/// Haskell `doParse`.
pub fn do_parse(path: &str, bytes: &[u8]) -> Result<Vec<InstructionPos<String>>, ParseError> {
    let src = dos2unix(&decode(bytes));
    let esc = find_escape_pragma(&haskell_lines(&src));
    run_parser(&src, path, esc, |p| contents(p, dockerfile))
}

/// Haskell `Data.Text.lines`: split on '\n', no trailing empty piece.
fn haskell_lines(s: &str) -> Vec<&str> {
    if s.is_empty() {
        return vec![];
    }
    let mut v: Vec<&str> = s.split('\n').collect();
    if v.last() == Some(&"") {
        v.pop();
    }
    v
}

/// Haskell `decode`: determine encoding from the byte order mark (lenient decoding).
pub fn decode(txt: &[u8]) -> String {
    const BOM_UTF32_LE: &[u8] = b"\xff\xfe\0\0";
    const BOM_UTF32_BE: &[u8] = b"\0\0\xfe\xff";
    const BOM_UTF16_LE: &[u8] = b"\xff\xfe";
    const BOM_UTF16_BE: &[u8] = b"\xfe\xff";
    const BOM_UTF8: &[u8] = b"\xef\xbb\xbf";
    let lossy32 = |b: &[u8], le: bool| -> String {
        b.chunks(4)
            .map(|c| {
                if c.len() < 4 {
                    return '\u{fffd}';
                }
                let a = [c[0], c[1], c[2], c[3]];
                let v = if le { u32::from_le_bytes(a) } else { u32::from_be_bytes(a) };
                char::from_u32(v).unwrap_or('\u{fffd}')
            })
            .collect()
    };
    let lossy16 = |b: &[u8], le: bool| -> String {
        let units: Vec<u16> = b
            .chunks(2)
            .map(|c| if c.len() < 2 { 0xfffd } else if le { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) })
            .collect();
        char::decode_utf16(units).map(|r| r.unwrap_or('\u{fffd}')).collect()
    };
    if txt.starts_with(BOM_UTF32_LE) {
        lossy32(&txt[4..], true)
    } else if txt.starts_with(BOM_UTF32_BE) {
        lossy32(&txt[4..], false)
    } else if txt.starts_with(BOM_UTF16_LE) {
        lossy16(&txt[2..], true)
    } else if txt.starts_with(BOM_UTF16_BE) {
        lossy16(&txt[2..], false)
    } else if txt.starts_with(BOM_UTF8) {
        String::from_utf8_lossy(&txt[3..]).into_owned()
    } else {
        String::from_utf8_lossy(txt).into_owned()
    }
}

/// Haskell `dos2unix`: changes crlf line endings to simple line endings.
pub fn dos2unix(t: &str) -> String {
    t.replace("\r\n", "\n")
}
