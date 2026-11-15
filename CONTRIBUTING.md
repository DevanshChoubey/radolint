# Contributing to radolint

Thanks for helping. radolint aims to behave exactly like hadolint v2.15.1,
so most contributions are one of these:

- **A bug where radolint and hadolint disagree.** Open an issue with the
  Dockerfile, the command line, and both outputs. If the difference is not
  listed in [docs/divergences.md](docs/divergences.md), it is a bug.
- **A fix for a known hadolint issue.** These change radolint's output on
  purpose. Open an issue first so we can agree on the behaviour. Every such
  change is marked with a `// DIVERGENCE:` comment in the code and described
  in `docs/divergences.md`.
- **Integrations.** Improvements to the GitHub Action, the pre-commit hook
  or the editor support.

## Building and testing

You need Rust 1.93 or newer.

```sh
cargo build
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

### Conformance suite

`conformance/corpus.jsonl` records what hadolint printed for 1,395
Dockerfiles. Every case must still pass after your change:

```sh
cargo run -p radolint-conformance -- --corpus conformance/corpus.jsonl
```

Use `--filter <id>` to run some of the cases, and `--verbose` to see diffs.
CI runs the same check on every push.

## Porting conventions

The code follows hadolint's Haskell sources closely. Functions keep their
Haskell names in snake_case, and each rule's hspec tests are ported into the
rule's `#[cfg(test)]` module. See [docs/porting.md](docs/porting.md) for the
details.

## Pull requests

- Keep each pull request to one change, and add tests for it.
- Add a line to [CHANGELOG.md](CHANGELOG.md) under the next version.
- By contributing, you agree that your work is licensed under
  GPL-3.0-only, the licence of the project.
