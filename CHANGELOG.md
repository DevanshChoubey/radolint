# Changelog

## Unreleased

More fixes for frequently reported hadolint issues (see
docs/divergences.md):

- A `RUN` or `COPY` heredoc followed by a comment or another instruction
  parses (hadolint #960, #1060).
- `COPY foo\` followed by an indented line parses (hadolint #1137).
- `RUN <<EOF bash` heredocs are checked as bash, and DL4006 accepts
  `set -o pipefail` in the heredoc body (hadolint #1137, #806).
- DL3032 accepts a cache mount of `/var/cache/yum`, and DL3040 one of
  `/var/cache/dnf` (hadolint #1139).
- DL3020 expands ARG and ENV values in ADD sources, so `ADD ${URL}` with a
  URL default is not reported (hadolint #769).

## 0.2.2

- GitHub Action: a shorter description, so the Action can be listed on the
  GitHub Marketplace.
- The GitHub Action and the VS Code extension download 0.2.2 by default.

## 0.2.1

- DL3026: a literal registry in front of a variable path
  (`FROM evil.com/${ORG}/img`) is checked; `ARG A=x B=y` sets both
  variables; a name that is not a global `ARG` is unset, so
  `${X:-evil.com}` is checked; `ARG` defaults may refer to earlier `ARG`s.
- pre-commit hook: the download directory is removed after install, and a
  cache directory left by an interrupted install no longer breaks the hook.
- checkstyle and junit: attribute values are escaped as hadolint does
  (`>` and `'` are left as they are), so the output is byte-identical again.
- The conformance suite now runs in CI and fails on any mismatch.
- The GitHub Action and the VS Code extension download 0.2.1 by default.

## 0.2.0

Fixes for some of hadolint's most-reported issues. These are the first
deliberate output differences from hadolint on inputs it does not crash on;
each is listed in docs/divergences.md.

- pre-commit: a `radolint` hook that downloads the matching release binary on
  first use, and `radolint-system` for a radolint on `PATH`
  (hadolint/hadolint#886).
- FROM: a registry with a port or `localhost` (`FROM localhost:5000/app:1`)
  parses instead of failing (hadolint/hadolint#339).
- DL3026: a trusted registry can include a path (`gcr.io/distroless`), and
  FROM images built from variables are expanded with the defaults of global
  `ARG`s; an image whose registry depends on an unknown build argument is not
  reported (hadolint/hadolint#401, hadolint/hadolint#339).
- DL3003: a `cd` inside a `( ... )` subshell is not reported
  (hadolint/hadolint#422).
- The GitHub Action and the VS Code extension download 0.2.0 by default.

## 0.1.1

- GitHub Action: fix the run step on macOS runners, whose bash 3.2 has no
  `globstar` (`**` matches one directory level there).
- The VS Code extension downloads 0.1.1 by default.

## 0.1.0

First release: a Rust port of hadolint v2.15.1.

- Every hadolint rule (DL3000–DL4006) with the same messages, severities and
  exit codes; `RUN` scripts are checked by shelllint v0.2.1, a Rust port of
  ShellCheck.
- The full hadolint command line, `.hadolint.yaml` config files,
  `HADOLINT_*` environment variables and `# hadolint ignore=` pragmas.
- All ten output formats: tty, json, checkstyle, codeclimate,
  gitlab_codeclimate, gnu, codacy, sonarqube, sarif and junit.
- Output matches hadolint byte for byte on 1,395 conformance cases. Known
  differences, all on inputs that crash hadolint, are listed in
  docs/divergences.md.
- A GitHub Action (`uses: DevanshChoubey/radolint@v0.1.0`).
