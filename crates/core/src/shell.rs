// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Shell.hs (v2.15.1), on top of the shelllint
//! ShellCheck port.
//!
//! Type mapping: ShellCheck's `Token` tree is the arena `shelllint_core::ast::Ast` and tokens
//! are `Id`s; `ParseResult` is `shelllint_parser::ParseResult` (holds the arena and root).
//! Functions that take a `Token` upstream take `(&Ast, Id)` here.
//!
//! Stub policy: the pure `Command` helpers are implemented; the ShellCheck-dependent functions
//! return neutral values (no commands, no comments) with a `// TODO(port)` marker.

use shelllint_core::ast::{Ast, Id};
use shelllint_core::interface::PositionedComment;
use shelllint_parser::ParseResult;
use std::collections::BTreeSet;

/// Haskell `CmdPart`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdPart {
    pub arg: String,
    pub part_id: i64,
}

/// Haskell `Command`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub arguments: Vec<CmdPart>,
    pub flags: Vec<CmdPart>,
}

/// Haskell `ParsedShell` (`Show` prints `presentCommands`).
#[derive(Clone)]
pub struct ParsedShell {
    pub original: String,
    pub parsed: ParseResult,
    pub present_commands: Vec<Command>,
}

impl std::fmt::Debug for ParsedShell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.present_commands)
    }
}

/// Haskell `ShellOpts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellOpts {
    pub shell_name: String,
    pub env_vars: BTreeSet<String>,
}

/// Haskell `defaultShellOpts`.
pub fn default_shell_opts() -> ShellOpts {
    ShellOpts {
        shell_name: "/bin/sh -c".to_string(),
        env_vars: [
            "HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy", "FTP_PROXY", "ftp_proxy",
            "NO_PROXY", "no_proxy",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
    }
}

/// Haskell `addVars`.
pub fn add_vars(vars: &[String], opts: ShellOpts) -> ShellOpts {
    let mut env_vars = opts.env_vars;
    env_vars.extend(vars.iter().cloned());
    ShellOpts { shell_name: opts.shell_name, env_vars }
}

/// Haskell `setShell`.
pub fn set_shell(s: &str, opts: ShellOpts) -> ShellOpts {
    ShellOpts { shell_name: s.to_string(), env_vars: opts.env_vars }
}

/// Haskell `shellcheck`: run ShellCheck over the script (mocked system interface, shebang from
/// the shell name, `export VAR=1` lines for the env vars, excluding 2187/1090/1091, min severity style).
pub fn shellcheck(opts: &ShellOpts, script: &ParsedShell) -> Vec<PositionedComment> {
    // TODO(port): upstream/hadolint/src/Hadolint/Shell.hs shellcheck
    let _ = (opts, script);
    vec![]
}

/// Haskell `nonPosixShells`.
pub fn non_posix_shells() -> &'static [&'static str] {
    &["pwsh", "powershell", "cmd"]
}

/// Haskell `hasUnsupportedShebang`.
pub fn has_unsupported_shebang(script: &str) -> bool {
    script.starts_with("#!")
        && ![
            "#!/bin/sh", "#!/bin/bash", "#!/bin/ksh", "#!/usr/bin/env sh", "#!/usr/bin/env bash",
            "#!/usr/bin/env ksh",
        ]
        .iter()
        .any(|p| script.starts_with(p))
}

/// Haskell `parseShell`: parse `#!/bin/bash\n<txt>` with ShellCheck and extract the commands.
pub fn parse_shell(txt: &str) -> ParsedShell {
    // TODO(port): upstream/hadolint/src/Hadolint/Shell.hs parseShell
    ParsedShell { original: txt.to_string(), parsed: ParseResult::default(), present_commands: vec![] }
}

/// Haskell `findCommandsInResult` (tokens that are commands, via `ASTLib.getCommand`).
pub fn find_commands_in_result(res: &ParseResult) -> Vec<Id> {
    // TODO(port): upstream/hadolint/src/Hadolint/Shell.hs findCommandsInResult
    let _ = res;
    vec![]
}

/// Haskell `extractTokensWith` (pre-order walk of the root, as `doAnalysis`).
pub fn extract_tokens_with<A>(extractor: &dyn Fn(&Ast, Id) -> Option<A>, res: &ParseResult) -> Vec<A> {
    // TODO(port): upstream/hadolint/src/Hadolint/Shell.hs extractTokensWith
    let _ = (extractor, res);
    vec![]
}

/// Haskell `findPipes`: all `T_Pipe` tokens.
pub fn find_pipes(shell: &ParsedShell) -> Vec<Id> {
    // TODO(port): upstream/hadolint/src/Hadolint/Shell.hs findPipes
    let _ = shell;
    vec![]
}

/// Haskell `hasPipes`.
pub fn has_pipes(shell: &ParsedShell) -> bool {
    !find_pipes(shell).is_empty()
}

/// Haskell `allCommands`.
pub fn all_commands(check: impl Fn(&Command) -> bool, script: &ParsedShell) -> bool {
    script.present_commands.iter().all(check)
}

/// Haskell `noCommands`.
pub fn no_commands(check: impl Fn(&Command) -> bool, script: &ParsedShell) -> bool {
    all_commands(|c| !check(c), script)
}

/// Haskell `anyCommands`.
pub fn any_commands(check: impl Fn(&Command) -> bool, script: &ParsedShell) -> bool {
    script.present_commands.iter().any(check)
}

/// Haskell `findCommandIndex`.
pub fn find_command_index(check: impl Fn(&Command) -> bool, script: &ParsedShell) -> Option<usize> {
    script.present_commands.iter().position(check)
}

/// Haskell `findCommandNames`.
pub fn find_command_names(script: &ParsedShell) -> Vec<String> {
    script.present_commands.iter().map(|c| c.name.clone()).collect()
}

/// Haskell `cmdHasArgs`.
pub fn cmd_has_args(expected_name: &str, expected_args: &[&str], cmd: &Command) -> bool {
    cmds_have_args(&[expected_name], expected_args, cmd)
}

/// Haskell `cmdsHaveArgs`.
pub fn cmds_have_args(expected_names: &[&str], expected_args: &[&str], cmd: &Command) -> bool {
    if !expected_names.contains(&cmd.name.as_str()) {
        return false;
    }
    cmd.arguments.iter().any(|p| expected_args.contains(&p.arg.as_str()))
}

/// Haskell `cmdHasPrefixArg`.
pub fn cmd_has_prefix_arg(expected_name: &str, expected_arg: &str, cmd: &Command) -> bool {
    if expected_name != cmd.name {
        return false;
    }
    cmd.arguments.iter().any(|p| p.arg.starts_with(expected_arg))
}

/// Haskell `extractAllArgs`: the arguments of a `T_SimpleCommand` token (empty for other tokens).
pub fn extract_all_args(ast: &Ast, token: Id) -> Vec<CmdPart> {
    // TODO(port): upstream/hadolint/src/Hadolint/Shell.hs extractAllArgs
    let _ = (ast, token);
    vec![]
}

/// Haskell `simplify` (modified `oversimplify` that keeps variable names).
pub fn simplify(ast: &Ast, token: Id) -> Vec<String> {
    // TODO(port): upstream/hadolint/src/Hadolint/Shell.hs simplify
    let _ = (ast, token);
    vec![]
}

/// Haskell `getArgs`.
pub fn get_args(cmd: &Command) -> Vec<String> {
    cmd.arguments.iter().map(|p| p.arg.clone()).collect()
}

/// Haskell `getAllFlags`.
pub fn get_all_flags(args: &[CmdPart]) -> Vec<CmdPart> {
    let mut out = vec![];
    for CmdPart { arg, part_id } in args {
        if arg == "--" || arg == "-" {
            continue;
        } else if let Some(rest) = arg.strip_prefix("--") {
            let name: String = rest.chars().take_while(|c| *c != '=').collect();
            out.push(CmdPart { arg: name, part_id: *part_id });
        } else if let Some(rest) = arg.strip_prefix('-') {
            for ch in rest.chars() {
                out.push(CmdPart { arg: ch.to_string(), part_id: *part_id });
            }
        }
    }
    out
}

/// Haskell `getArgsNoFlags`.
pub fn get_args_no_flags(cmd: &Command) -> Vec<String> {
    let flag_ids: Vec<i64> = cmd.flags.iter().map(|f| f.part_id).collect();
    cmd.arguments.iter().filter(|a| !flag_ids.contains(&a.part_id)).map(|a| a.arg.clone()).collect()
}

/// Haskell `hasFlag`.
pub fn has_flag(flag: &str, cmd: &Command) -> bool {
    count_flag(flag, cmd) > 0
}

/// Haskell `countFlag`.
pub fn count_flag(flag: &str, cmd: &Command) -> usize {
    cmd.flags.iter().filter(|f| f.arg == flag).count()
}

/// Haskell `hasAnyFlag`.
pub fn has_any_flag(fs: &[&str], cmd: &Command) -> bool {
    cmd.flags.iter().any(|f| fs.contains(&f.arg.as_str()))
}

/// Haskell `hasArg`.
pub fn has_arg(arg: &str, cmd: &Command) -> bool {
    cmd.arguments.iter().any(|a| a.arg == arg)
}

/// Haskell `dropFlagArg`.
pub fn drop_flag_arg(flags_to_drop: &[&str], cmd: &Command) -> Command {
    let has_inline_value =
        |f_id: i64| cmd.arguments.iter().any(|a| a.part_id == f_id && a.arg.contains('='));
    let ids_to_drop: BTreeSet<i64> = cmd
        .flags
        .iter()
        .filter(|f| flags_to_drop.contains(&f.arg.as_str()) && !has_inline_value(f.part_id))
        .map(|f| get_value_id(f.part_id, &cmd.arguments))
        .collect();
    Command {
        name: cmd.name.clone(),
        arguments: cmd.arguments.iter().filter(|a| !ids_to_drop.contains(&a.part_id)).cloned().collect(),
        flags: cmd.flags.clone(),
    }
}

/// Haskell `getFlagArg`: the arguments of a flag, e.g. `useradd -u 12345 luser` with `u` gives `["12345"]`.
pub fn get_flag_arg(flag: &str, cmd: &Command) -> Vec<String> {
    let ids: BTreeSet<i64> = cmd
        .flags
        .iter()
        .filter(|f| f.arg == flag)
        .map(|f| get_value_id(f.part_id, &cmd.arguments))
        .collect();
    cmd.arguments.iter().filter(|a| ids.contains(&a.part_id)).map(|a| a.arg.clone()).collect()
}

/// Haskell `getValueId` (`maxBound :: Int` when there is none).
pub fn get_value_id(f_id: i64, flags: &[CmdPart]) -> i64 {
    flags.iter().map(|p| p.part_id).filter(|i| *i > f_id).fold(i64::MAX, i64::min)
}

/// Haskell `usingProgram`: does the Run instruction call the program?
pub fn using_program(prog: &str, args: &ParsedShell) -> bool {
    find_command_names(args).iter().any(|c| c == prog)
}

/// Haskell `isPipInstall`.
pub fn is_pip_install(cmd: &Command) -> bool {
    let args = get_args(cmd);
    let contains = |needle: &[&str]| args.windows(needle.len()).any(|w| w.iter().zip(needle).all(|(a, b)| a == b));
    let std = cmd.name.starts_with("pip") && !cmd.name.starts_with("pipenv") && contains(&["install"]);
    let py = cmd.name.starts_with("python") && contains(&["-m", "pip", "install"]);
    std || py
}
