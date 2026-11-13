# Divergences from hadolint

radolint matches hadolint v2.15.1 output exactly on the conformance corpus. The
cases below are where hadolint itself fails, and radolint deliberately does not.

## DL3000 on a one-character relative WORKDIR

hadolint crashes on `WORKDIR x` (any relative path shorter than two
characters): `Data.Text.index` in `Hadolint/Rule/DL3000.hs` reads past the end
of the text while checking for a Windows drive letter, and the process exits 1
with a Haskell call stack instead of a report.

radolint treats such a path as not absolute, so it reports DL3000 for it like any
other relative path and the rest of the file is still checked. See the
`// DIVERGENCE:` comment in `crates/core/src/rules/dl3000.rs`.

## DL3021 on a COPY target that is only quotes

hadolint crashes on `COPY foo bar "`: the target `"` is not empty, but after
`dropQuotes` it is, and `Data.Text.last` in `Hadolint/Rule/DL3021.hs` fails on
the empty text.

radolint treats the empty target as not ending with `/`, so it reports DL3021 and
keeps checking the file. See the `// DIVERGENCE:` comment in
`crates/core/src/rules/dl3021.rs`.

## ShellCheck findings follow a newer ShellCheck

hadolint v2.15.1 links the ShellCheck 0.11.0 release. radolint runs `RUN`
scripts through shelllint v0.2.1, a port of ShellCheck v0.11.0-171-ge11f964,
so checks changed in those 171 commits can report differently. For example,
upstream commit 776ed08 ("issue SC2114 against glob") makes `rm -r /*` report
SC2114 ("deletes a system directory") where hadolint reports SC2115. Every
ShellCheck case in the conformance corpus matches.

## The `--help` header names radolint

hadolint's `--help` starts with `hadolint - Dockerfile Linter written in
Haskell`; radolint's starts with `radolint - Dockerfile Linter written in
Rust`. The rest of `--help` and the usage errors are laid out exactly as
optparse-applicative lays them out, under whatever name the binary is started
as. Inline pragmas keep hadolint's spelling
(`# hadolint ignore=DL3008`), so existing Dockerfiles need no changes.
