// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3016.hs.

use crate::rule::{DLSeverity, Rule, combine, fold_arguments, onbuild, simple_rule};
use crate::shell::{self, ParsedShell};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3016(), onbuild(dl3016()))
}

/// Rule for pinning NPM packages to version, tag, or commit.
fn dl3016() -> Box<dyn Rule> {
    let code = "DL3016";
    let severity = DLSeverity::DLWarningC;
    let message = "Pin versions in npm. Instead of `npm install <package>` use `npm install <package>@<version>`";

    let check = |instr: &Instruction<ParsedShell>| match instr {
        Instruction::Run(RunArgs(args, _)) => fold_arguments(
            |a: &ParsedShell| shell::no_commands(forgot_to_pin_version, a),
            args,
        ),
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

fn forgot_to_pin_version(cmd: &shell::Command) -> bool {
    is_npm_install(cmd) && install_is_first(cmd) && !packages(cmd).iter().all(|p| version_fixed(p))
}

fn is_npm_install(cmd: &shell::Command) -> bool {
    shell::cmd_has_args("npm", &["install"], cmd)
}

fn install_is_first(cmd: &shell::Command) -> bool {
    let a = shell::get_args_no_flags(&shell::drop_flag_arg(&IGNORE_FLAGS, cmd));
    a.first().map(|s| s == "install").unwrap_or(false)
}

fn packages(cmd: &shell::Command) -> Vec<String> {
    strip_install_prefix(&shell::get_args_no_flags(&shell::drop_flag_arg(
        &IGNORE_FLAGS,
        cmd,
    )))
}

fn version_fixed(package: &str) -> bool {
    if has_git_prefix(package) {
        is_versioned_git(package)
    } else if has_tarball_suffix(package) {
        true
    } else if is_folder(package) {
        true
    } else {
        has_version_symbol(package)
    }
}

fn has_git_prefix(package: &str) -> bool {
    GIT_PREFIXES.iter().any(|p| package.starts_with(p))
}

fn has_tarball_suffix(package: &str) -> bool {
    TARBALL_SUFFIXES.iter().any(|p| package.ends_with(p))
}

fn is_folder(package: &str) -> bool {
    PATH_PREFIXES.iter().any(|p| package.starts_with(p))
}

fn is_versioned_git(package: &str) -> bool {
    package.contains('#')
}

fn has_version_symbol(package: &str) -> bool {
    // dropScope: `Text.dropWhile ('/' <)` drops characters strictly greater than '/'.
    let drop_scope = |pkg: &str| -> String {
        if pkg.starts_with('@') {
            pkg.trim_start_matches(|c: char| '/' < c).to_string()
        } else {
            pkg.to_string()
        }
    };
    drop_scope(package).contains('@')
}

fn strip_install_prefix(cmd: &[String]) -> Vec<String> {
    let rest: Vec<String> = cmd
        .iter()
        .skip_while(|s| *s != "install")
        .cloned()
        .collect();
    rest.into_iter().skip_while(|s| s == "install").collect()
}

const IGNORE_FLAGS: [&str; 2] = ["loglevel", "registry"];
const GIT_PREFIXES: [&str; 4] = ["git://", "git+ssh://", "git+http://", "git+https://"];
const PATH_PREFIXES: [&str; 4] = ["/", "./", "../", "~/"];
const TARBALL_SUFFIXES: [&str; 3] = [".tar", ".tar.gz", ".tgz"];

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn version_pinned_in_package_json() {
        rule_catches_not("DL3016", "RUN npm install");
        on_build_rule_catches_not("DL3016", "RUN npm install");
    }

    #[test]
    fn version_pinned_in_package_json_with_arguments() {
        rule_catches_not("DL3016", "RUN npm install --progress=false");
        on_build_rule_catches_not("DL3016", "RUN npm install --progress=false");
    }

    #[test]
    fn version_pinned() {
        rule_catches_not("DL3016", "RUN npm install express@4.1.1");
        on_build_rule_catches_not("DL3016", "RUN npm install express@4.1.1");
    }

    #[test]
    fn version_pinned_with_scope() {
        rule_catches_not(
            "DL3016",
            "RUN npm install @myorg/privatepackage@\">=0.1.0\"",
        );
        on_build_rule_catches_not(
            "DL3016",
            "RUN npm install @myorg/privatepackage@\">=0.1.0\"",
        );
    }

    #[test]
    fn version_pinned_multiple_packages() {
        rule_catches_not("DL3016", "RUN npm install express@\"4.1.1\" sax@0.1.1");
        on_build_rule_catches_not("DL3016", "RUN npm install express@\"4.1.1\" sax@0.1.1");
    }

    #[test]
    fn version_pinned_with_global() {
        rule_catches_not("DL3016", "RUN npm install --global express@\"4.1.1\"");
        on_build_rule_catches_not("DL3016", "RUN npm install --global express@\"4.1.1\"");
    }

    #[test]
    fn version_pinned_with_g() {
        rule_catches_not("DL3016", "RUN npm install -g express@\"4.1.1\"");
        on_build_rule_catches_not("DL3016", "RUN npm install -g express@\"4.1.1\"");
    }

    #[test]
    fn version_does_not_have_to_be_pinned_for_tarball_suffix_tar() {
        rule_catches_not("DL3016", "RUN npm install package-v1.2.3.tar");
        on_build_rule_catches_not("DL3016", "RUN npm install package-v1.2.3.tar");
    }

    #[test]
    fn version_does_not_have_to_be_pinned_for_tarball_suffix_tar_gz() {
        rule_catches_not("DL3016", "RUN npm install package-v1.2.3.tar.gz");
        on_build_rule_catches_not("DL3016", "RUN npm install package-v1.2.3.tar.gz");
    }

    #[test]
    fn version_does_not_have_to_be_pinned_for_tarball_suffix_tgz() {
        rule_catches_not("DL3016", "RUN npm install package-v1.2.3.tgz");
        on_build_rule_catches_not("DL3016", "RUN npm install package-v1.2.3.tgz");
    }

    #[test]
    fn version_does_not_have_to_be_pinned_for_folder_absolute_path() {
        rule_catches_not("DL3016", "RUN npm install /folder");
        on_build_rule_catches_not("DL3016", "RUN npm install /folder");
    }

    #[test]
    fn version_does_not_have_to_be_pinned_for_folder_relative_path_from_current_folder() {
        rule_catches_not("DL3016", "RUN npm install ./folder");
        on_build_rule_catches_not("DL3016", "RUN npm install ./folder");
    }

    #[test]
    fn version_does_not_have_to_be_pinned_for_folder_relative_path_to_parent_folder() {
        rule_catches_not("DL3016", "RUN npm install ../folder");
        on_build_rule_catches_not("DL3016", "RUN npm install ../folder");
    }

    #[test]
    fn version_does_not_have_to_be_pinned_for_folder_relative_path_from_home() {
        rule_catches_not("DL3016", "RUN npm install ~/folder");
        on_build_rule_catches_not("DL3016", "RUN npm install ~/folder");
    }

    #[test]
    fn commit_pinned_for_git_ssh() {
        rule_catches_not(
            "DL3016",
            "RUN npm install git+ssh://git@github.com:npm/npm.git#v1.0.27",
        );
        on_build_rule_catches_not(
            "DL3016",
            "RUN npm install git+ssh://git@github.com:npm/npm.git#v1.0.27",
        );
    }

    #[test]
    fn commit_pinned_for_git_http() {
        rule_catches_not(
            "DL3016",
            "RUN npm install git+http://isaacs@github.com/npm/npm#semver:^5.0",
        );
        on_build_rule_catches_not(
            "DL3016",
            "RUN npm install git+http://isaacs@github.com/npm/npm#semver:^5.0",
        );
    }

    #[test]
    fn commit_pinned_for_git_https() {
        rule_catches_not(
            "DL3016",
            "RUN npm install git+https://isaacs@github.com/npm/npm.git#v1.0.27",
        );
        on_build_rule_catches_not(
            "DL3016",
            "RUN npm install git+https://isaacs@github.com/npm/npm.git#v1.0.27",
        );
    }

    #[test]
    fn commit_pinned_for_git() {
        rule_catches_not(
            "DL3016",
            "RUN npm install git://github.com/npm/npm.git#v1.0.27",
        );
        on_build_rule_catches_not(
            "DL3016",
            "RUN npm install git://github.com/npm/npm.git#v1.0.27",
        );
    }

    #[test]
    fn npm_run_install_is_fine() {
        rule_catches_not("DL3016", "RUN npm run --crazy install");
        on_build_rule_catches_not("DL3016", "RUN npm run --crazy install");
    }

    #[test]
    fn version_pinned_with_scope_2() {
        rule_catches_not(
            "DL3016",
            "RUN npm install @myorg/privatepackage@\">=0.1.0 <0.2.0\"",
        );
        on_build_rule_catches_not(
            "DL3016",
            "RUN npm install @myorg/privatepackage@\">=0.1.0 <0.2.0\"",
        );
    }

    #[test]
    fn version_not_pinned() {
        rule_catches("DL3016", "RUN npm install express");
        on_build_rule_catches("DL3016", "RUN npm install express");
    }

    #[test]
    fn version_not_pinned_with_scope() {
        rule_catches("DL3016", "RUN npm install @myorg/privatepackage");
        on_build_rule_catches("DL3016", "RUN npm install @myorg/privatepackage");
    }

    #[test]
    fn version_not_pinned_multiple_packages() {
        rule_catches("DL3016", "RUN npm install express sax@0.1.1");
        on_build_rule_catches("DL3016", "RUN npm install express sax@0.1.1");
    }

    #[test]
    fn version_not_pinned_with_global() {
        rule_catches("DL3016", "RUN npm install --global express");
        on_build_rule_catches("DL3016", "RUN npm install --global express");
    }

    #[test]
    fn commit_not_pinned_for_git_ssh() {
        rule_catches(
            "DL3016",
            "RUN npm install git+ssh://git@github.com:npm/npm.git",
        );
        on_build_rule_catches(
            "DL3016",
            "RUN npm install git+ssh://git@github.com:npm/npm.git",
        );
    }

    #[test]
    fn commit_not_pinned_for_git_http() {
        rule_catches(
            "DL3016",
            "RUN npm install git+http://isaacs@github.com/npm/npm",
        );
        on_build_rule_catches(
            "DL3016",
            "RUN npm install git+http://isaacs@github.com/npm/npm",
        );
    }

    #[test]
    fn commit_not_pinned_for_git_https() {
        rule_catches(
            "DL3016",
            "RUN npm install git+https://isaacs@github.com/npm/npm.git",
        );
        on_build_rule_catches(
            "DL3016",
            "RUN npm install git+https://isaacs@github.com/npm/npm.git",
        );
    }

    #[test]
    fn commit_not_pinned_for_git() {
        rule_catches("DL3016", "RUN npm install git://github.com/npm/npm.git");
        on_build_rule_catches("DL3016", "RUN npm install git://github.com/npm/npm.git");
    }

    #[test]
    fn don_t_fire_on_loglevel_flag() {
        rule_catches_not("DL3016", "RUN npm install --loglevel verbose sax@0.1.1");
        on_build_rule_catches_not("DL3016", "RUN npm install --loglevel verbose sax@0.1.1");
    }

    #[test]
    fn don_t_fire_on_registry_flag() {
        rule_catches_not(
            "DL3016",
            "RUN npm install --registry https://example.com sax@0.1.1",
        );
        on_build_rule_catches_not(
            "DL3016",
            "RUN npm install --registry https://example.com sax@0.1.1",
        );
    }

    #[test]
    fn version_not_pinned_with_registry_flag() {
        rule_catches(
            "DL3016",
            "RUN npm install --registry https://example.com express",
        );
        on_build_rule_catches(
            "DL3016",
            "RUN npm install --registry https://example.com express",
        );
    }

    #[test]
    fn version_not_pinned_with_registry_value_form() {
        rule_catches(
            "DL3016",
            "RUN npm install --registry=https://example.com express",
        );
        on_build_rule_catches(
            "DL3016",
            "RUN npm install --registry=https://example.com express",
        );
    }
}
