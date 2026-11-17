<div align="center">

# radolint

**A fast Dockerfile linter. A drop-in replacement for hadolint, written in Rust.**

[![CI](https://github.com/DevanshChoubey/radolint/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/DevanshChoubey/radolint/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/DevanshChoubey/radolint?sort=semver)](https://github.com/DevanshChoubey/radolint/releases/latest)
[![VS Code Marketplace](https://img.shields.io/visual-studio-marketplace/v/DevanshChoubey.radolint?label=VS%20Code)](https://marketplace.visualstudio.com/items?itemName=DevanshChoubey.radolint)
[![Open VSX](https://img.shields.io/open-vsx/v/DevanshChoubey/radolint?label=Open%20VSX)](https://open-vsx.org/extension/DevanshChoubey/radolint)
[![MSRV](https://img.shields.io/badge/rust-1.93%2B-orange)](#from-source)
[![Licence: GPL-3.0-only](https://img.shields.io/badge/licence-GPL--3.0--only-blue)](LICENSE)

[Install](#installation) ·
[Quick start](#quick-start) ·
[Integrations](#integrations) ·
[Configuration](#configuration) ·
[Compatibility](#compatibility) ·
[Benchmarks](#benchmarks) ·
[Contributing](CONTRIBUTING.md)

</div>

---

radolint is a line-by-line port of [hadolint](https://github.com/hadolint/hadolint)
v2.15.1. It has the same rules, flags, `.hadolint.yaml` config and
`# hadolint ignore=` pragmas as hadolint. Its output matches hadolint byte for
byte, except where it fixes
[long-standing hadolint bugs](#fixed-hadolint-issues). Shell code in `RUN`
instructions is checked by
[shelllint](https://github.com/DevanshChoubey/shelllint), a Rust port of
ShellCheck that is built in, so there is nothing else to install.

```console
$ radolint Dockerfile
Dockerfile:1 DL3006 warning: Always tag the version of an image explicitly
Dockerfile:2 DL3008 warning: Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`
Dockerfile:2 DL3015 info: Avoid additional packages by specifying `--no-install-recommends`
Dockerfile:2 DL3014 warning: Use the `-y` switch to avoid manual input `apt-get -y install <package>`
```

## Highlights

- **Drop-in compatible.** It gives the same findings, messages, exit codes
  and output formats as hadolint. Swap the binary and keep your CI config.
- **Fast and light.** It uses 3.5–6× less CPU than hadolint and less than
  half its memory ([benchmarks](#benchmarks)).
- **One small binary.** About 10 MB and statically linked on Linux, with
  ShellCheck built in. No runtime, Docker or Haskell toolchain is needed.
- **Fewer false alarms.** It fixes several of hadolint's most-reported issues,
  such as `localhost:5000` registries, trusted registry paths and `cd` in
  subshells.
- **Runs everywhere.** Linux (x86_64, arm64), macOS (Apple silicon, Intel)
  and Windows (x86_64).
- **Integrations ready to use.** A GitHub Action, a pre-commit hook and a
  VS Code extension, plus every editor plugin that already runs hadolint.

## Installation

### Prebuilt binaries

Download an archive for your platform from the
[latest release](https://github.com/DevanshChoubey/radolint/releases/latest).
Each archive has a `.sha256` checksum, and `SHA256SUMS` covers all of them.

| Platform | Archive |
|---|---|
| Linux x86_64 (static) | `radolint-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz` |
| Linux arm64 (static) | `radolint-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple silicon | `radolint-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `radolint-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| Windows x86_64 | `radolint-vX.Y.Z-x86_64-pc-windows-msvc.zip` |

For example, on Linux x86_64:

```sh
curl -fsSLO https://github.com/DevanshChoubey/radolint/releases/latest/download/radolint-latest-x86_64-unknown-linux-musl.tar.gz
tar -xzf radolint-latest-x86_64-unknown-linux-musl.tar.gz
sudo install radolint-*/radolint /usr/local/bin/
```

### From source

You need Rust 1.93 or newer:

```sh
cargo install --locked --git https://github.com/DevanshChoubey/radolint radolint-cli
```

### As a hadolint replacement

radolint accepts everything hadolint does. To move an existing setup over
without changing it, install radolint under the name `hadolint`:

```sh
ln -s "$(command -v radolint)" ~/.local/bin/hadolint
```

## Quick start

```sh
radolint Dockerfile                                   # lint a file
radolint Dockerfile docker/*.Dockerfile               # lint several files
cat Dockerfile | radolint -                           # read from stdin
radolint --format json Dockerfile                     # machine-readable output
radolint --ignore DL3008 --failure-threshold warning Dockerfile
radolint --trusted-registry gcr.io/distroless Dockerfile
```

radolint exits with status 1 when a finding reaches the failure threshold
(`info` by default), and 0 otherwise. Pass `--no-fail` to always exit 0.

## Integrations

### GitHub Actions

```yaml
- uses: actions/checkout@v4
- uses: DevanshChoubey/radolint@v0.3.0
  with:
    dockerfile: Dockerfile        # space-separated files or globs, e.g. "**/Dockerfile*"
    failure-threshold: warning    # error, warning, info (default), style, ignore, none
    ignore: DL3008 DL3013
```

Findings appear as annotations on the pull request. The Action downloads a
checksum-verified release binary, so it needs no Docker and runs on Linux,
macOS and Windows runners.

| Input | Description |
|---|---|
| `dockerfile` | Files or globs to lint (default `Dockerfile`) |
| `config` | Path to a `.hadolint.yaml` |
| `failure-threshold` | Lowest severity that fails the step |
| `ignore` | Space-separated rule codes to skip |
| `format` / `output-file` | For example `sarif` and `radolint.sarif` to upload to code scanning |
| `no-fail` | Report findings without failing the step |
| `extra-args` | Any other radolint flags |
| `version` | radolint version to download (defaults to the Action's release) |

### pre-commit

```yaml
repos:
  - repo: https://github.com/DevanshChoubey/radolint
    rev: v0.3.0
    hooks:
      - id: radolint
```

The `radolint` hook downloads and caches the checksum-verified release
binary for `rev` the first time it runs. You don't need to install Docker,
Rust or Python first. Use `id: radolint-system` instead to run a radolint
that is already on `PATH`. Arguments work as usual, for example
`args: [--ignore, DL3008]`.

### Editors

- **VS Code** and compatible editors: install
  [radolint](https://marketplace.visualstudio.com/items?itemName=DevanshChoubey.radolint)
  from the Marketplace or
  [Open VSX](https://open-vsx.org/extension/DevanshChoubey/radolint). It lints
  as you type and has quick fixes for ignore pragmas.
- **Neovim, Vim, Emacs, Sublime Text, Zed, Helix and JetBrains IDEs:** any
  plugin that runs hadolint works with radolint. See
  [editors/README.md](editors/README.md) for the settings.

### Other CI systems

Download a release archive in your pipeline, or call radolint wherever you
call hadolint today. `--format` supports `gitlab_codeclimate`, `sonarqube`,
`codacy`, `checkstyle`, `junit` and `sarif` for the usual report viewers.

## Configuration

radolint looks for config files in the same places hadolint does: a
`.hadolint.yaml` in the current directory, `$XDG_CONFIG_HOME/hadolint.yaml`,
`~/.hadolint.yaml`, or a file given with `--config`.

```yaml
# .hadolint.yaml
failure-threshold: warning
ignored:
  - DL3008
trustedRegistries:
  - docker.io
  - gcr.io/distroless
override:
  error:
    - DL3001
label-schema:
  maintainer: text
strict-labels: true
```

Rules can be switched off inline for one instruction or the whole file:

```dockerfile
# hadolint ignore=DL3008,DL3015
RUN apt-get update && apt-get install -y curl

# hadolint global ignore=DL3059
```

The `HADOLINT_*` environment variables (`HADOLINT_FAILURE_THRESHOLD`,
`HADOLINT_IGNORE`, `HADOLINT_TRUSTED_REGISTRIES` and the rest) work as they
do in hadolint. For the full list of rules, see the
[hadolint rule reference](https://github.com/hadolint/hadolint#rules). radolint
implements every rule on that list.

### Output formats

| Format | Use |
|---|---|
| `tty` | Human-readable (default) |
| `json` | Scripts and tooling |
| `sarif` | GitHub code scanning and other SARIF viewers |
| `checkstyle`, `junit` | CI test and report viewers |
| `codeclimate`, `gitlab_codeclimate` | Code Climate and GitLab code quality |
| `gnu` | Editors that parse `file:line: message` |
| `codacy`, `sonarqube` | Codacy and SonarQube imports |

## Compatibility

radolint is tested against the real hadolint, not against a reading of its
source:

- Every Haskell rule and its hspec tests are ported one to one, giving more
  than 990 Rust tests.
- 1,395 Dockerfiles from the test suites of hadolint and language-docker
  were run through hadolint, and CI checks on every commit that radolint
  prints identical text and JSON output for each of them.
- About 96,000 more generated, mutated and real-world Dockerfiles have been
  compared the same way, across every flag, config option and output format.

The places where radolint deliberately behaves differently are listed in
[docs/divergences.md](docs/divergences.md). They are inputs that make
hadolint crash, and the bug fixes below.

### Fixed hadolint issues

| hadolint issue | What radolint does |
|---|---|
| [#886](https://github.com/hadolint/hadolint/issues/886): the pre-commit hook needs hadolint installed first | The [`radolint` hook](#pre-commit) downloads the binary itself |
| [#339](https://github.com/hadolint/hadolint/issues/339): `FROM localhost:5000/img:tag` fails to parse, and `FROM $ARG` gets DL3026 | Registries with a port or `localhost` parse, and FROM variables are expanded from global `ARG` defaults |
| [#401](https://github.com/hadolint/hadolint/issues/401): trusted registries ignore paths like `gcr.io/distroless` | `--trusted-registry gcr.io/distroless` allows the images under that path |
| [#422](https://github.com/hadolint/hadolint/issues/422): DL3003 flags `cd` in a `( ... )` subshell | A `cd` in a subshell is not reported |
| [#960](https://github.com/hadolint/hadolint/issues/960), [#1060](https://github.com/hadolint/hadolint/issues/1060): a heredoc followed by a comment or instruction fails to parse | Heredocs end at their marker and the next line parses |
| [#1137](https://github.com/hadolint/hadolint/issues/1137), [#806](https://github.com/hadolint/hadolint/issues/806): `RUN <<EOF bash` is checked as sh, and `COPY foo\` line continuations fail to parse | The heredoc body is checked as bash, `set -o pipefail` in it satisfies DL4006, and the continuation parses |
| [#1139](https://github.com/hadolint/hadolint/issues/1139): DL3032 and DL3040 ignore yum and dnf4 cache mounts | Cache mounts of `/var/cache/yum` and `/var/cache/dnf` are accepted |
| [#769](https://github.com/hadolint/hadolint/issues/769): `ADD $URL` gets DL3020 | ADD sources are expanded from the stage's ARG and ENV values |

## Benchmarks

Measured on Linux x86_64 against the hadolint 2.15.1 release binary, best of
three runs. CPU is user plus system time, and memory is peak RSS. Both tools
printed identical JSON for both workloads.

| Workload | hadolint CPU | radolint CPU | hadolint memory | radolint memory |
|---|---|---|---|---|
| One 20,000-line Dockerfile | 38.0 s | 10.5 s | 73 MB | 28 MB |
| 1,298 Dockerfiles in one call | 8.7 s | 1.4 s | 63 MB | 15 MB |

The 1,298 files are every distinct Dockerfile in hadolint's own test suite.
The large file repeats a valid subset of them.

## Project layout

| Crate | Contents |
|---|---|
| [`radolint-cli`](crates/cli) | The `radolint` binary: argument parsing, config discovery, exit codes |
| [`radolint-core`](crates/core) | Rules, config, formatters and the lint pipeline (port of `hadolint/src`) |
| [`radolint-parser`](crates/parser) | Dockerfile parser (port of language-docker on a megaparsec-like layer) |
| [`radolint-syntax`](crates/syntax) | Dockerfile syntax tree |
| [`radolint-conformance`](crates/conformance) | Runs the 1,395-case conformance corpus |

The GitHub Action is defined in [`action.yml`](action.yml), the pre-commit
hook in [`packaging/pre-commit`](packaging/pre-commit), and the editor
integrations in [`editors/`](editors).

## Contributing

Bug reports and pull requests are welcome. If radolint's output differs from
hadolint's in a way that [docs/divergences.md](docs/divergences.md) does not
explain, that is a bug, so please
[open an issue](https://github.com/DevanshChoubey/radolint/issues) with the
Dockerfile and the command. See [CONTRIBUTING.md](CONTRIBUTING.md) for how
to build, test and run the conformance suite. Release notes are in
[CHANGELOG.md](CHANGELOG.md). To report a security problem, see
[SECURITY.md](SECURITY.md).

## Licence

radolint is released under the GPL-3.0-only licence, the same as hadolint.
See [LICENSE](LICENSE).

radolint is an independent port. It is not affiliated with or endorsed by the
hadolint project, and all credit for the rule set goes to the hadolint and
ShellCheck authors. Docker is a trademark of Docker, Inc.
