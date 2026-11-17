# Divergences from hadolint

radolint matches hadolint v2.15.1 output exactly on the conformance corpus. The
cases below are where hadolint itself fails, and radolint deliberately does not:
first inputs that crash hadolint, then long-standing hadolint bugs that
radolint fixes. Each is marked with a `DIVERGENCE:` comment in the code.

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

## Registries with a port or `localhost` (hadolint/hadolint#339)

hadolint only reads the first part of a FROM image as a registry when it has a
dot, so `FROM localhost:5000/bash:1.0` is a parse error (`unexpected ':'`):
`localhost` becomes the image and `5000/bash` its tag. radolint follows
Docker: the first part is a registry when it has a dot, a port, or is
`localhost`. The same rule applies to images named in `COPY --from` and
`RUN --mount=from=`. See `parse_port_registry` in `crates/parser/src/from.rs`.

## DL3026 trusted registries with a path (hadolint/hadolint#401)

hadolint compares a trusted registry only with the host of the image, so
`--trusted-registry gcr.io/distroless` never matches
`gcr.io/distroless/base-debian10` and the only way to allow it was trusting
all of `gcr.io`. In radolint an entry with a `/` also matches image paths at
whole path components: `gcr.io/distroless` allows `gcr.io/distroless/base`
but not `gcr.io/distroless-x/base`; `reg.io/team/*` works as a prefix; Docker
Hub images are `docker.io/library/<name>` or `docker.io/<user>/<name>`.
Entries without a `/` work exactly as in hadolint.

## DL3026 on FROM images built from variables (hadolint/hadolint#339)

hadolint takes the text of `FROM $BASE` as a Docker Hub image name, so with
any trusted registry other than docker.io it reports DL3026 even when
`ARG BASE=trusted.com/app:1` says otherwise. radolint expands `$NAME`,
`${NAME}`, `${NAME:-word}`, `${NAME-word}`, `${NAME:+word}` and
`${NAME+word}` with the defaults of the ARGs declared before the first FROM,
as Docker does, and checks the result. Defaults may refer to earlier ARGs, and
a name that is not a global ARG is unset. When the registry depends on a build
argument without a default, it cannot be known, so DL3026 is not reported; a
literal registry in front of a variable path (`evil.com/${ORG}/img`) is still
checked.

## DL3003 and `cd` in a subshell (hadolint/hadolint#422)

hadolint's DL3003 documentation suggests running `cd` in a subshell, as in
`RUN (cd /src && make) && rm -rf /src`, but the rule reports every `cd`.
radolint does not report a `cd` inside `( ... )`, which cannot change the
directory of the commands after it. A `cd` in a `{ ...; }` group or at the top
level is still reported.

## Heredocs followed by another line (hadolint/hadolint#960, #1060)

hadolint fails to parse a Dockerfile in which a `RUN` or `COPY` heredoc is
followed by a comment or another instruction, because the parser consumes the
line break after the closing marker. radolint leaves that line break in place,
as language-docker PR #115 does, so the next line parses.

## COPY and ADD with a line continuation (hadolint/hadolint#1137)

hadolint fails to parse `COPY foo.json\` followed by an indented
`/root/foo.json`: the line break is pulled into the source path. radolint
ends the token at an escaped line break followed by whitespace, as
language-docker PR #114 does. A continuation line that starts without
whitespace still joins the two words, as Docker does, and is an error.

## RUN heredocs fed to a shell (hadolint/hadolint#1137, #806)

For `RUN <<EOF bash`, the heredoc body runs under bash, not under the image
SHELL. hadolint checks the body as POSIX sh and reports DL4006 even when the
body starts with `set -o pipefail`. Following hadolint PR #1277, radolint
checks the body with ShellCheck in the dialect of the named shell (bash, ksh,
ash, sh or dash; a zsh heredoc keeps the image shell), and DL4006 is not
reported when a bash, zsh, ash or ksh heredoc sets `pipefail` before its first
pipe.

## Cache mounts for yum and dnf (hadolint/hadolint#1139)

DL3032 now accepts a cache or tmpfs mount of `/var/cache/yum` in place of
`yum clean all`, and DL3040 accepts a mount of `/var/cache/dnf` (dnf4, as on
RHEL, Rocky and Alma 8 and 9) as well as `/var/cache/libdnf5`. DL3036 and
DL3040 already allowed this for zypper and dnf5.

## DL3020 on ADD sources built from variables (hadolint/hadolint#769)

hadolint checks the text of `ADD ${URL} /app/`, so it always reports DL3020
even when `ARG URL=https://...` makes the source a URL. radolint expands the
source with the ARG and ENV values of its build stage, as Docker does: values
reset at each FROM, and a global ARG applies only once the stage declares it
again with `ARG NAME`. When a value is unknown (an ARG without a default, or a
variable that may come from the base image), DL3020 is not reported, unless
the literal text around the variables already rules out a URL and an archive,
as in `ADD conf/${ENV}.json /etc/`.
