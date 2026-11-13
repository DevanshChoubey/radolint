# radolint

**A fast Dockerfile linter: a drop-in hadolint replacement written in Rust.**

radolint is a line-by-line port of [hadolint](https://github.com/hadolint/hadolint)
v2.15.1. It has the same rules, the same command line, the same
`.hadolint.yaml` config and the same `# hadolint ignore=` pragmas, and it
produces byte-for-byte the same output. The shell in `RUN` instructions is
checked by [shelllint](https://github.com/DevanshChoubey/shelllint), a Rust
port of ShellCheck, so there is nothing else to install.

```console
$ radolint Dockerfile
Dockerfile:1 DL3006 warning: Always tag the version of an image explicitly
Dockerfile:2 DL3008 warning: Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`
Dockerfile:2 DL3015 info: Avoid additional packages by specifying `--no-install-recommends`
Dockerfile:2 DL3014 warning: Use the `-y` switch to avoid manual input `apt-get -y install <package>`
```

## Why

- **Faster.** About 3.5× less CPU than hadolint on one large Dockerfile and
  about 6× less across many small ones (see [Benchmarks](#benchmarks)).
- **Smaller.** Under half of hadolint's memory, and a single binary of
  about 10 MB (hadolint's is about 55 MB).
- **Same results.** Every finding, message, exit code and output format
  matches hadolint. Swap the binary and keep your CI config.

## Install

Build from source with Rust 1.93 or newer:

```sh
cargo install --locked --git https://github.com/DevanshChoubey/radolint radolint-cli
```

Or download a prebuilt binary from
[Releases](https://github.com/DevanshChoubey/radolint/releases): static
Linux builds (x86_64 and arm64), macOS (Apple silicon and Intel) and Windows
(x86_64).

## GitHub Action

```yaml
- uses: actions/checkout@v4
- uses: DevanshChoubey/radolint@v0.1.0
  with:
    dockerfile: Dockerfile        # space-separated files or globs, e.g. "**/Dockerfile*"
    failure-threshold: warning    # error, warning, info (default), style, ignore, none
    ignore: DL3008 DL3013
```

Findings show up as annotations on the pull request. Other inputs: `config`,
`format`, `output-file` (for example `format: sarif` with
`output-file: radolint.sarif` for code scanning), `no-fail`, `extra-args` and
`version`. The Action downloads a checksum-verified release binary, so it
needs no Docker and works on Linux, macOS and Windows runners.

## Use it like hadolint

Every hadolint flag works the same way:

```sh
radolint Dockerfile
radolint --format json Dockerfile
radolint --ignore DL3008 --failure-threshold warning Dockerfile
cat Dockerfile | radolint -
```

Output formats: `tty`, `json`, `checkstyle`, `codeclimate`,
`gitlab_codeclimate`, `gnu`, `codacy`, `sonarqube`, `sarif` and `junit`.

radolint finds config files the same way hadolint does (`.hadolint.yaml` in
the current directory, `$XDG_CONFIG_HOME/hadolint.yaml`, `~/.hadolint.yaml`
and so on, or a file given with `--config`). It also reads the same
`HADOLINT_*` environment variables and the same inline pragmas:

```dockerfile
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y curl
```

To switch an existing pipeline over, install radolint under the name
`hadolint` or point your tooling at the `radolint` binary. Nothing else
changes.

## Benchmarks

Measured on Linux x86_64 with the hadolint 2.15.1 release binary, best of
three runs. CPU is user plus system time and memory is peak RSS. Both tools
printed identical JSON for both workloads.

| Workload | hadolint CPU | radolint CPU | hadolint memory | radolint memory |
|---|---|---|---|---|
| One 20,000-line Dockerfile | 38.0 s | 10.5 s | 73 MB | 28 MB |
| 1,298 Dockerfiles in one call | 8.7 s | 1.4 s | 63 MB | 15 MB |

The 1,298 files are every distinct Dockerfile in hadolint's own test suite.
The large file repeats a valid subset of them.

## How it stays compatible

radolint is checked against the real hadolint, not against a reading of its
source:

- each Haskell rule and its hspec tests are ported one to one (976 Rust tests);
- 1,395 Dockerfiles from hadolint's and language-docker's test suites are run
  through both tools and must give identical text and JSON output;
- about 84,000 generated Dockerfiles were compared the same way while porting.

The few places where radolint deliberately differs, all of them inputs that
make hadolint crash, are listed in [docs/divergences.md](docs/divergences.md).

## Licence

GPL-3.0-only, the same licence as hadolint. See [LICENSE](LICENSE).

radolint is an independent port and is not affiliated with the hadolint
project. Docker is a trademark of Docker, Inc.
