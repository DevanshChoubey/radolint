# Divergences from hadolint

dlint matches hadolint v2.15.1 output exactly on the conformance corpus. The
cases below are where hadolint itself fails, and dlint deliberately does not.

## DL3000 on a one-character relative WORKDIR

hadolint crashes on `WORKDIR x` (any relative path shorter than two
characters): `Data.Text.index` in `Hadolint/Rule/DL3000.hs` reads past the end
of the text while checking for a Windows drive letter, and the process exits 1
with a Haskell call stack instead of a report.

dlint treats such a path as not absolute, so it reports DL3000 for it like any
other relative path and the rest of the file is still checked. See the
`// DIVERGENCE:` comment in `crates/core/src/rules/dl3000.rs`.
