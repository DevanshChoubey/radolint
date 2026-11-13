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

use shelllint_core::ast::{Ast, Id, Token};
use shelllint_core::astlib;
use shelllint_core::interface::{
    PositionedComment, Severity, empty_check_spec, mocked_system_interface, new_parse_spec,
};
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
    let ShellOpts { shell_name: sh, env_vars: env } = opts;
    let txt = &script.original;
    if non_posix_shells().iter().any(|n| sh.contains(n)) || has_unsupported_shebang(txt) {
        return vec![]; // Do no run for non-posix shells i.e. powershell, cmd.exe
    }
    let si = mocked_system_interface(vec![(String::new(), String::new())]);
    let extract_shell = |s: &str| s.split_whitespace().next().unwrap_or("").to_string();
    let print_vars: String = env
        .iter()
        .map(|v| format!("export {}=1", v))
        .collect::<BTreeSet<String>>()
        .iter()
        .map(|l| format!("{}\n", l))
        .collect();
    let script_text = format!("#!{}\n{}{}", extract_shell(sh), print_vars, txt);
    let mut spec = empty_check_spec();
    spec.filename = String::new(); // filename can be omitted because we only want the parse results back
    spec.script = script_text;
    spec.check_sourced = false;
    spec.excluded_warnings = vec![
        2187, // exclude the warning about the ash shell not being supported
        1090, // requires a directive (shell comment) that can't be expressed in a Dockerfile
        1091, // requires a directive (shell comment) that can't be expressed in a Dockerfile
    ];
    spec.shell_type_override = None;
    spec.min_severity = Severity::StyleC;
    shelllint_checker::check_script(&si, &spec).comments
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
    let si = mocked_system_interface(vec![(String::new(), String::new())]);
    let mut spec = new_parse_spec();
    spec.filename = String::new(); // There is no filename
    spec.script = format!("#!/bin/bash\n{}", txt);
    spec.check_sourced = false;
    let parsed_result = shelllint_parser::parse_script(&si, &spec);

    let extract_names = |token: Id| -> Option<Command> {
        let n = astlib::get_command_name(&parsed_result.ast, token)?;
        let all_args = extract_all_args(&parsed_result.ast, token);
        let flags = get_all_flags(&all_args);
        Some(Command { name: n, arguments: all_args, flags })
    };
    let commands: Vec<Command> =
        find_commands_in_result(&parsed_result).into_iter().filter_map(extract_names).collect();
    ParsedShell { original: txt.to_string(), parsed: parsed_result, present_commands: commands }
}

/// Haskell `findCommandsInResult` (tokens that are commands, via `ASTLib.getCommand`).
pub fn find_commands_in_result(res: &ParseResult) -> Vec<Id> {
    extract_tokens_with(&|ast, t| astlib::get_command(ast, t), res)
}

/// Haskell `extractTokensWith` (pre-order walk of the root, as `doAnalysis`).
pub fn extract_tokens_with<A>(extractor: &dyn Fn(&Ast, Id) -> Option<A>, res: &ParseResult) -> Vec<A> {
    match res.root {
        None => vec![],
        Some(script) => {
            let mut out = vec![];
            res.ast.analyze(
                script,
                &mut |t| {
                    if let Some(x) = extractor(&res.ast, t) {
                        out.push(x);
                    }
                },
                &mut |_| {},
            );
            out
        }
    }
}

/// Haskell `findPipes`: all `T_Pipe` tokens.
pub fn find_pipes(shell: &ParsedShell) -> Vec<Id> {
    let pipes_extractor = |ast: &Ast, t: Id| match ast.get(t) {
        Token::T_Pipe(_) => Some(t),
        _ => None,
    };
    extract_tokens_with(&pipes_extractor, &shell.parsed)
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
    match ast.get(token) {
        Token::T_SimpleCommand(_, words) if !words.is_empty() => words[1..]
            .iter()
            .map(|&t| CmdPart { arg: simplify(ast, t).concat(), part_id: t.0 as i64 })
            .collect(),
        _ => vec![],
    }
}

/// Haskell `simplify` (modified `oversimplify` that keeps variable names).
pub fn simplify(ast: &Ast, token: Id) -> Vec<String> {
    let cm = |l: &[Id]| -> Vec<String> { l.iter().flat_map(|&x| simplify(ast, x)).collect() };
    match ast.get(token) {
        Token::T_NormalWord(l) => vec![cm(l).concat()],
        Token::T_DoubleQuoted(l) => vec![cm(l).concat()],
        Token::T_SingleQuoted(s) => vec![s.clone()],
        Token::T_DollarBraced(_, t) => vec![format!("${{{}}}", simplify(ast, *t).concat())],
        Token::T_DollarArithmetic(_) => vec!["${VAR}".to_string()],
        Token::T_DollarExpansion(t) => vec![format!("${{{}}}", cm(t).concat())],
        Token::T_Backticked(_) => vec!["${VAR}".to_string()],
        Token::T_Glob(s) => vec![s.clone()],
        Token::T_Pipeline(_, l) if l.len() == 1 => simplify(ast, l[0]),
        Token::T_Literal(x) => vec![x.clone()],
        Token::T_ParamSubSpecialChar(x) => vec![x.clone()],
        Token::T_SimpleCommand(_, wds) => cm(wds),
        Token::T_Redirecting(_, foo) => simplify(ast, *foo),
        Token::T_DollarSingleQuoted(s) => vec![s.clone()],
        Token::T_Annotation(_, s) => simplify(ast, *s),
        Token::TA_Sequence(l) if l.len() == 1 => match ast.get(l[0]) {
            Token::TA_Expansion(v) => cm(v),
            _ => vec![],
        },
        _ => vec![],
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn cp(a: &str, i: i64) -> CmdPart {
        CmdPart { arg: a.to_string(), part_id: i }
    }
    fn cmd(name: &str, args: Vec<CmdPart>, flags: Vec<CmdPart>) -> Command {
        Command { name: name.to_string(), arguments: args, flags }
    }

    #[test]
    fn get_flag_args() {
        let c = cmd("useradd", vec![cp("12345", 2), cp("67890", 4), cp("luser", 5)], vec![cp("u", 1), cp("u", 3)]);
        assert_eq!(get_flag_arg("u", &c), vec!["12345", "67890"]);
        let c = cmd("useradd", vec![cp("12345", 2), cp("luser", 3)], vec![cp("u", 1)]);
        assert_eq!(get_flag_arg("u", &c), vec!["12345"]);
        let c = cmd("useradd", vec![cp("12345", 2), cp("luser", 3)], vec![cp("u", 1)]);
        assert_eq!(get_flag_arg("f", &c), Vec::<String>::new());
    }

    #[test]
    fn has_flag_spec() {
        let c = cmd("useradd", vec![cp("luser", 1)], vec![cp("l", 0)]);
        assert!(has_flag("l", &c));
        let c = cmd("useradd", vec![cp("luser", 1)], vec![cp("l", 0)]);
        assert!(!has_flag("f", &c));
    }

    #[test]
    fn drop_flag_arg_spec() {
        let c = cmd("npm", vec![cp("verbose", 2), cp("install", 3), cp("bla@1.0.0", 4)], vec![cp("loglevel", 1)]);
        let res = cmd("npm", vec![cp("install", 3), cp("bla@1.0.0", 4)], vec![cp("loglevel", 1)]);
        assert_eq!(drop_flag_arg(&["loglevel"], &c), res);
    }

    #[test]
    fn drop_flag_arg_keeps_the_next_argument_when_the_flag_value_is_inline() {
        let cmds = parse_shell("npm --loglevel=info install bla@1.0.0").present_commands;
        assert_ne!(cmds, vec![]);
        let dropped: Vec<Command> = cmds.iter().map(|c| drop_flag_arg(&["loglevel"], c)).collect();
        assert_eq!(dropped, cmds);
    }
}
