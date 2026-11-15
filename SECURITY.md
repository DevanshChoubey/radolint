# Security policy

## Supported versions

Security fixes go into the latest release. Please upgrade to it before
reporting a problem.

## Reporting a vulnerability

Please do not open a public issue for a security problem. Report it
privately through GitHub instead:

1. Go to the [Security tab](https://github.com/DevanshChoubey/radolint/security)
   of this repository.
2. Click **Report a vulnerability**.

Include the radolint version, how radolint was run (the CLI, the GitHub
Action, the pre-commit hook or the VS Code extension), and the steps or
input needed to reproduce the problem.

You should get a reply within a week. Once a fix is released, the advisory
is published with credit to you, unless you would rather stay anonymous.

## Scope

These count as security problems:

- the GitHub Action, the pre-commit hook or the VS Code extension
  downloading or running a binary without verifying its checksum;
- any way a Dockerfile or config file can make radolint run commands, or
  read or write files it should not.

radolint only reads the Dockerfiles and config files you give it. It never
runs the commands inside them.
