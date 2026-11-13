# Porting one file

Goal: a faithful Rust port of hadolint v2.15.1 (+ language-docker v16.0.0). The oracle is
`bench/.bin/hadolint-hs`; `conformance/corpus.jsonl` (what the oracle printed for 1395 inputs) is the
parity authority, not your reading of the Haskell. Never build or run the Haskell under `upstream/`.

## Rules of the road

- You own exactly ONE file (its stub already exists with final signatures). Do not edit any other file,
  Cargo manifest, or `mod.rs`. If a signature must change, stop and report it instead of editing.
- Work in your private copy of the repo; the build has no scripts and no absolute paths inside the sources.
- Replace `todo!(..)` / `// TODO(port)` bodies. Keep the Haskell names (snake_case), the order of
  definitions and the structure of each function; keep behaviour identical, including odd corners.
- Port the file's Spec (`upstream/hadolint/test/...Spec.hs`, `upstream/language-docker/test/...`) into
  `#[cfg(test)] mod tests` in the same file, one `#[test]` per `it`, using `crate::testing`
  (`rule_catches`, `rule_catches_not`, `on_build_rule_catches(_not)`, `*_with(&config, ..)`, `assert_checks`,
  `fails_shellcheck`, ...). Parser modules test through `dlint_parser::combinator::run_parser`.
- Where Rust cannot match Haskell exactly, leave a `// DIVERGENCE: <why>` comment.
- Parsers use the megaparsec emulation in `crates/parser/src/combinator.rs` (direct style, `&mut Parser`,
  `p.try_`, `p.alt`, `p.label`, ...); error text must match megaparsec ("unexpected ..., expecting ...").

## Build and check (always through the slot script)

    cd <your copy> && scripts/cargo-slot.sh check -q -j 4 --workspace --all-targets
    scripts/cargo-slot.sh test -q -p dlint-core <module>          # e.g. rules::dl3008
    scripts/cargo-slot.sh test -q -p dlint-parser <module>
    scripts/cargo-slot.sh run -q -p dlint-conformance -- --verbose --filter <id-substring>

`dlint-conformance` prints `pass N / fail M`; `--verbose` shows per-case diffs (exact, order-sensitive),
`--filter` selects ids such as `rule:DL3008`. Panics from unported stubs count as failures.

## Layout

`crates/syntax` Syntax.hs (done) | `crates/parser` combinator + language-docker Parser/*.hs |
`crates/core` Rule.hs, Shell.hs, rules/ (one file per rule), process, pragma, config, formatter, lint |
`crates/cli` app/Main.hs | `crates/conformance` the corpus runner (done).
