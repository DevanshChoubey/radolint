// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3028.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{self, ParsedShell};
use radolint_syntax::{Instruction, RunArgs};

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3028(), onbuild(dl3028()))
}

/// Haskell `dl3028`.
fn dl3028() -> Box<dyn Rule> {
    let code = "DL3028";
    let severity = DLSeverity::DLWarningC;
    let message = "Pin versions in gem install. Instead of `gem install <gem>` use `gem install <gem>:<version>`";

    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => fold_arguments(
            |a: &ParsedShell| gems(a).iter().all(|p| version_fixed(p)),
            args,
        ),
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

fn version_fixed(package: &str) -> bool {
    package.contains(':')
}

/// Haskell `gems`.
fn gems(shell: &ParsedShell) -> Vec<String> {
    let mut out = vec![];
    for cmd in &shell.present_commands {
        if shell::cmd_has_args("gem", &["install", "i"], cmd)
            && !shell::cmd_has_args("gem", &["-v"], cmd)
            && !shell::cmd_has_args("gem", &["--version"], cmd)
            && !shell::cmd_has_prefix_arg("gem", "--version=", cmd)
        {
            let args = shell::get_args(cmd);
            let args_until_double_dash: Vec<String> =
                args.into_iter().take_while(|a| a != "--").collect();
            for arg in remove_options(&args_until_double_dash) {
                if arg != "install" && arg != "i" {
                    out.push(arg);
                }
            }
        }
    }
    out
}

fn remove_options(xs: &[String]) -> Vec<String> {
    let mut out = vec![];
    let mut i = 0;
    while i < xs.len() {
        let x = &xs[i];
        if x == "--" {
            i += 1;
        } else if x.starts_with('-') {
            i += 2;
        } else {
            out.push(x.clone());
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    fn both_catch(d: &str) {
        rule_catches("DL3028", d);
        on_build_rule_catches("DL3028", d);
    }
    fn both_not(d: &str) {
        rule_catches_not("DL3028", d);
        on_build_rule_catches_not("DL3028", d);
    }

    #[test]
    fn i_unpinned() {
        both_catch("RUN gem i bundler");
    }
    #[test]
    fn i_pinned() {
        both_not("RUN gem i bundler:1");
    }
    #[test]
    fn i_multi() {
        both_catch("RUN gem i bunlder:1 nokogiri");
        both_not("RUN gem i bunlder:1 nokogirii:1");
        on_build_rule_catches_not("DL3028", "RUN gem i bunlder:1 nokogiri:1");
        rule_catches_not("DL3028", "RUN gem i bunlder:1 nokogiri:1");
    }
    #[test]
    fn install_unpinned() {
        both_catch("RUN gem install bundler");
    }
    #[test]
    fn install_pinned() {
        both_not("RUN gem install bundler:1");
    }
    #[test]
    fn no_warn_on_v() {
        both_not("RUN gem install bundler -v '2.0.1'");
    }
    #[test]
    fn no_warn_on_version_without_eq() {
        both_not("RUN gem install bundler --version '2.0.1'");
    }
    #[test]
    fn no_warn_on_version_with_eq() {
        both_not("RUN gem install bundler --version='2.0.1'");
    }
    #[test]
    fn extra_flags_eq_double_dash() {
        both_not("RUN gem install bundler:2.0.1 -- --use-system-libraries=true");
    }
    #[test]
    fn extra_flags_double_dash() {
        both_not("RUN gem install bundler:2.0.1 -- --use-system-libraries true");
    }
    #[test]
    fn extra_flags() {
        both_not("RUN gem install bundler:2.0.1 --use-system-libraries true");
    }
    #[test]
    fn extra_flags_eq() {
        both_not("RUN gem install bundler:2.0.1 --use-system-libraries=true");
    }
}
