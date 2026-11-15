# Changelog

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
