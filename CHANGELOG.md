# Changelog

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
