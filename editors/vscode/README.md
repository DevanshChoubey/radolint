# radolint for VS Code

Lint Dockerfiles as you type with [radolint](https://github.com/DevanshChoubey/radolint),
a fast drop-in hadolint replacement written in Rust. You get hadolint's rules,
`.hadolint.yaml` config and `# hadolint ignore=` pragmas, and ShellCheck
findings for `RUN` instructions.

## Features

- Findings as you type (or on save), with links to each rule's documentation.
- Quick fixes: **Ignore DLxxxx for this instruction** adds or extends a
  `# hadolint ignore=` pragma, and **Ignore DLxxxx in this file** adds a
  `# hadolint global ignore=` pragma.
- Parse errors are shown where they happen.
- No setup: if radolint is not on your PATH, the extension offers to download
  the release binary for your platform from GitHub and checks its SHA-256
  checksum first.

## Settings

| Setting | Default | Meaning |
|---|---|---|
| `radolint.path` | empty | Binary to run. Empty means `radolint` on PATH, then the downloaded copy. A `hadolint` binary works too. |
| `radolint.run` | `onType` | `onType` (debounced) or `onSave`. |
| `radolint.config` | empty | Config file to pass with `--config`. Empty means hadolint's usual lookup from the workspace folder. |
| `radolint.ignore` | `[]` | Rule codes to ignore, such as `DL3008`. |
| `radolint.trustedRegistries` | `[]` | Registries allowed in `FROM` (DL3026). |
| `radolint.extraArgs` | `[]` | Any other hadolint flags, one per item. |

Commands: **radolint: Lint Open Dockerfiles**, **radolint: Download radolint
Binary** and **radolint: Show Output**.

## Development

```sh
npm ci
npm run compile
RADOLINT_BIN=/path/to/radolint npm test   # runs the extension in a throwaway VS Code
npm run package                           # builds radolint-<version>.vsix
```
