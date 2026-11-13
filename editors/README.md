# Editor integration

## VS Code

The extension in [vscode/](vscode/) runs radolint on each Dockerfile as you
type and has quick fixes for ignore pragmas. Install the `.vsix` from the
[latest release](https://github.com/DevanshChoubey/radolint/releases/latest):

```sh
code --install-extension radolint-0.1.0.vsix
```

If you already use the hadolint extension (`exiasr.hadolint`), you can keep it
and point it at radolint instead:

```json
{ "hadolint.hadolintPath": "/path/to/radolint" }
```

## Everything else

radolint takes the same flags and prints the same output as hadolint, so any
editor plugin that runs hadolint works with radolint. Either point the plugin
at the `radolint` binary, or install radolint under the name `hadolint`
(a symlink or a copy).

- **Neovim (nvim-lint):**
  ```lua
  require("lint").linters.hadolint.cmd = "radolint"
  require("lint").linters_by_ft.dockerfile = { "hadolint" }
  ```
- **Vim/Neovim (ALE):** ALE runs `hadolint` from PATH, so install radolint
  under that name (`ln -s "$(command -v radolint)" ~/.local/bin/hadolint`).
- **Emacs (Flycheck):** `(setq flycheck-dockerfile-hadolint-executable "radolint")`
- **Sublime Text (SublimeLinter-hadolint), Zed, Helix (efm-langserver),
  JetBrains (Hadolint plugin) and others:** set the plugin's hadolint
  executable path to radolint, or put radolint on PATH as `hadolint`.
