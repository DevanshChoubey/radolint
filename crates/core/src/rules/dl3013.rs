// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3013.hs.

use crate::rule::{combine, fold_arguments, onbuild, simple_rule, DLSeverity, Rule};
use crate::shell::{self, Command};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3013(), onbuild(dl3013()))
}

/// Haskell `dl3013`.
pub fn dl3013() -> Box<dyn Rule> {
    let code = "DL3013";
    let severity = DLSeverity::DLWarningC;
    let message = "Pin versions in pip. Instead of `pip install <package>` use `pip install \
                   <package>==<version>` or `pip install --requirement <requirements file>`";

    let check = |instr: &Instruction<shell::ParsedShell>| -> bool {
        match instr {
            Instruction::Run(RunArgs(args, _)) => {
                fold_arguments(|s| shell::no_commands(forgot_to_pin_version, s), args)
            }
            _ => true,
        }
    };
    simple_rule(code, severity, message, check)
}

fn forgot_to_pin_version(cmd: &Command) -> bool {
    is_pip_install(cmd) && !has_build_constraint(cmd) && !packages(cmd).iter().all(|p| version_fixed(p))
}

// Check if the command is a pip* install command, and that specific packages are being listed
fn is_pip_install(cmd: &Command) -> bool {
    (shell::is_pip_install(cmd) && !has_build_constraint(cmd) && !packages(cmd).iter().all(|p| version_fixed(p)))
        && !requirement_install(cmd)
}

// If the user is installing requirements from a file or just the local module, then we are not interested
// in running this rule
fn requirement_install(cmd: &Command) -> bool {
    let args = shell::get_args(cmd);
    args.iter().any(|a| a == "--requirement") || args.iter().any(|a| a == "-r") || args.iter().any(|a| a == ".")
}

fn has_build_constraint(cmd: &Command) -> bool {
    shell::has_flag("constraint", cmd) || shell::has_flag("c", cmd)
}

fn version_fixed(package: &str) -> bool {
    has_version_symbol(package)
        || is_versioned_vcs(package)
        || is_local_package(package)
        || is_no_vcs_path_source(package)
}

fn is_versioned_vcs(package: &str) -> bool {
    is_vcs(package) && package.contains('@')
}

const VERSION_SYMBOLS: [&str; 8] = ["==", ">=", "<=", ">", "<", "!=", "~=", "==="];

fn has_version_symbol(package: &str) -> bool {
    VERSION_SYMBOLS.iter().any(|s| package.contains(s))
}

const LOCAL_PACKAGE_FILE_EXTENSIONS: [&str; 2] = [".whl", ".tar.gz"];

fn is_local_package(package: &str) -> bool {
    LOCAL_PACKAGE_FILE_EXTENSIONS.iter().any(|s| package.ends_with(s))
}

fn is_no_vcs_path_source(package: &str) -> bool {
    !is_vcs(package) && package.contains('/')
}

fn is_vcs(package: &str) -> bool {
    VCS_SCHEMES.iter().any(|s| package.starts_with(s))
}

/// Haskell `packages`.
fn packages(cmd: &Command) -> Vec<String> {
    strip_install_prefix(shell::get_args_no_flags(&shell::drop_flag_arg(
        &[
            "abi",
            "b",
            "build",
            "e",
            "editable",
            "extra-index-url",
            "f",
            "find-links",
            "i",
            "index-url",
            "implementation",
            "no-binary",
            "only-binary",
            "platform",
            "prefix",
            "progress-bar",
            "proxy",
            "python",
            "python-version",
            "root",
            "root-user-action",
            "src",
            "t",
            "target",
            "trusted-host",
            "upgrade-strategy",
        ],
        cmd,
    )))
}

// Supported schemes vcs[+protocol] are found here:
// https://pip.pypa.io/en/stable/topics/vcs-support/
const VCS_SCHEMES: [&str; 22] = [
    "git+file",
    "git+https",
    "git+ssh",
    "git+http",
    "git+git",
    "git",
    "hg+file",
    "hg+http",
    "hg+https",
    "hg+ssh",
    "hg+static-http",
    "svn",
    "svn+svn",
    "svn+http",
    "svn+https",
    "svn+ssh",
    "bzr+http",
    "bzr+https",
    "bzr+ssh",
    "bzr+sftp",
    "bzr+ftp",
    "bzr+lp",
];

/// Haskell `stripInstallPrefix`.
fn strip_install_prefix(cmd: Vec<String>) -> Vec<String> {
    let rest: Vec<String> = cmd.into_iter().skip_while(|a| a != "install").collect();
    rest.into_iter().skip_while(|a| a == "install").collect()
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn pip2_version_not_pinned() {
        rule_catches("DL3013", "RUN pip2 install MySQL_python");
        on_build_rule_catches("DL3013", "RUN pip2 install MySQL_python");
    }

    #[test]
    fn pip3_version_not_pinned() {
        rule_catches("DL3013", "RUN pip3 install MySQL_python");
        on_build_rule_catches("DL3013", "RUN pip2 install MySQL_python");
    }

    #[test]
    fn pip3_version_pinned() {
        rule_catches_not("DL3013", "RUN pip3 install MySQL_python==1.2.2");
        on_build_rule_catches_not("DL3013", "RUN pip3 install MySQL_python==1.2.2");
    }

    #[test]
    fn pip3_install_from_local_package() {
        rule_catches_not("DL3013", "RUN pip3 install mypkg.whl");
        rule_catches_not("DL3013", "RUN pip3 install mypkg.tar.gz");
        on_build_rule_catches_not("DL3013", "RUN pip3 install mypkg.whl");
        on_build_rule_catches_not("DL3013", "RUN pip3 install mypkg.tar.gz");
    }

    #[test]
    fn pip_install_requirements() {
        rule_catches_not("DL3013", "RUN pip install -r requirements.txt");
        on_build_rule_catches_not("DL3013", "RUN pip install -r requirements.txt");
    }

    #[test]
    fn pip_install_requirements_with_long_flag() {
        rule_catches_not("DL3013", "RUN pip install --requirement requirements.txt");
        on_build_rule_catches_not("DL3013", "RUN pip install --requirement requirements.txt");
    }

    #[test]
    fn pip_install_use_setup_py() {
        rule_catches_not("DL3013", "RUN pip install .");
        on_build_rule_catches_not("DL3013", "RUN pip install .");
    }

    #[test]
    fn pip_version_not_pinned() {
        rule_catches("DL3013", "RUN pip install MySQL_python");
        on_build_rule_catches("DL3013", "RUN pip install MySQL_python");
    }

    #[test]
    fn pip_version_pinned() {
        rule_catches_not("DL3013", "RUN pip install MySQL_python==1.2.2");
        on_build_rule_catches_not("DL3013", "RUN pip install MySQL_python==1.2.2");
    }

    #[test]
    fn pip_version_pinned_with_operator() {
        rule_catches_not("DL3013", "RUN pip install MySQL_python~=1.2.2");
        on_build_rule_catches_not("DL3013", "RUN pip install MySQL_python~=1.2.2");
    }

    #[test]
    fn pip_version_pinned_with_operator_2() {
        rule_catches_not("DL3013", "RUN pip install MySQL_python===1.2.2");
        on_build_rule_catches_not("DL3013", "RUN pip install MySQL_python===1.2.2");
    }

    #[test]
    fn pip_version_pinned_with_flag_ignore_installed() {
        rule_catches_not("DL3013", "RUN pip install --ignore-installed MySQL_python==1.2.2");
        on_build_rule_catches_not("DL3013", "RUN pip install --ignore-installed MySQL_python==1.2.2");
    }

    #[test]
    fn pip_version_pinned_with_flag_build() {
        rule_catches_not("DL3013", "RUN pip3 install --build /opt/yamllint yamllint==1.20.0");
        on_build_rule_catches_not("DL3013", "RUN pip3 install --build /opt/yamllint yamllint==1.20.0");
    }

    #[test]
    fn pip_version_pinned_with_flag_prefix() {
        rule_catches_not("DL3013", "RUN pip3 install --prefix /opt/yamllint yamllint==1.20.0");
        on_build_rule_catches_not("DL3013", "RUN pip3 install --prefix /opt/yamllint yamllint==1.20.0");
    }

    #[test]
    fn pip_version_pinned_with_flag_root() {
        rule_catches_not("DL3013", "RUN pip3 install --root /opt/yamllint yamllint==1.20.0");
        on_build_rule_catches_not("DL3013", "RUN pip3 install --root /opt/yamllint yamllint==1.20.0");
    }

    #[test]
    fn pip_version_pinned_with_flag_target() {
        rule_catches_not("DL3013", "RUN pip3 install --target /opt/yamllint yamllint==1.20.0");
        on_build_rule_catches_not("DL3013", "RUN pip3 install --target /opt/yamllint yamllint==1.20.0");
    }

    #[test]
    fn pip_version_pinned_with_flag_trusted_host() {
        rule_catches_not("DL3013", "RUN pip3 install --trusted-host host example==1.2.2");
        on_build_rule_catches_not("DL3013", "RUN pip3 install --trusted-host host example==1.2.2");
    }

    #[test]
    fn pip_version_pinned_with_python_m() {
        rule_catches_not("DL3013", "RUN python -m pip install example==1.2.2");
        on_build_rule_catches_not("DL3013", "RUN python -m pip install example==1.2.2");
    }

    #[test]
    fn pip_version_not_pinned_with_python_m() {
        rule_catches("DL3013", "RUN python -m pip install example");
        on_build_rule_catches("DL3013", "RUN python -m pip install --index-url url example");
    }

    #[test]
    fn pip_install_git() {
        rule_catches_not("DL3013", "RUN pip install git+https://github.com/rtfd/r-ext.git@0.6-alpha#egg=r-ext");
        on_build_rule_catches_not("DL3013", "RUN pip install git+https://github.com/rtfd/r-ext.git@0.6-alpha#egg=r-ext");
        rule_catches_not("DL3013", "RUN pip install git+ssh://github.com/rtfd/r-ext.git@0.6-alpha#egg=r-ext");
        on_build_rule_catches_not("DL3013", "RUN pip install git+ssh://github.com/rtfd/r-ext.git@0.6-alpha#egg=r-ext");
    }

    #[test]
    fn pip_install_unversioned_git() {
        rule_catches("DL3013", "RUN pip install git+https://github.com/rtfd/read-ext.git#egg=read-ext");
        on_build_rule_catches("DL3013", "RUN pip install git+https://github.com/rtfd/read-ext.git#egg=read-ext");
        rule_catches("DL3013", "RUN pip install git+ssh://github.com/rtfd/read-ext.git#egg=read-ext");
        on_build_rule_catches("DL3013", "RUN pip install git+ssh://github.com/rtfd/read-ext.git#egg=read-ext");
    }

    #[test]
    fn pip_install_local_dir() {
        rule_catches_not("DL3013", "RUN pip install foo/bar");
        on_build_rule_catches_not("DL3013", "RUN pip install foo/bar");
    }

    #[test]
    fn pip_install_https_url_package() {
        rule_catches_not("DL3013", "RUN pip install https://foo.bar/baz.zip");
        on_build_rule_catches_not("DL3013", "RUN pip install https://foo.bar/baz.zip");
    }

    #[test]
    fn pip_install_upper_bound() {
        rule_catches_not("DL3013", "RUN pip install 'alabaster>=0.7'");
        on_build_rule_catches_not("DL3013", "RUN pip install 'alabaster>=0.7'");
    }

    #[test]
    fn pip_install_lower_bound() {
        rule_catches_not("DL3013", "RUN pip install 'alabaster<0.7'");
        on_build_rule_catches_not("DL3013", "RUN pip install 'alabaster<0.7'");
    }

    #[test]
    fn pip_install_excluded_version() {
        rule_catches_not("DL3013", "RUN pip install 'alabaster!=0.7'");
        on_build_rule_catches_not("DL3013", "RUN pip install 'alabaster!=0.7'");
    }

    #[test]
    fn pip_install_user_directory() {
        rule_catches_not("DL3013", "RUN pip install MySQL_python==1.2.2 --user");
        on_build_rule_catches_not("DL3013", "RUN pip install MySQL_python==1.2.2 --user");
    }

    #[test]
    fn pip_install_no_pip_version_check() {
        rule_catches_not("DL3013", "RUN pip install MySQL_python==1.2.2 --disable-pip-version-check");
        on_build_rule_catches_not("DL3013", "RUN pip install MySQL_python==1.2.2 --disable-pip-version-check");
    }

    #[test]
    fn pip_install_index_url() {
        rule_catches_not("DL3013", "RUN pip install --index-url https://eg.com/foo foobar==1.0.0");
        on_build_rule_catches_not("DL3013", "RUN pip install --index-url https://eg.com/foo foobar==1.0.0");
    }

    #[test]
    fn pip_install_index_url_with_i_flag() {
        rule_catches_not("DL3013", "RUN pip install --index-url https://eg.com/foo foobar==1.0.0");
        on_build_rule_catches_not("DL3013", "RUN pip install --index-url https://eg.com/foo foobar==1.0.0");
    }

    #[test]
    fn pip_install_index_url_with_extra_index_url() {
        rule_catches_not("DL3013", "RUN pip install --index-url https://eg.com/foo --extra-index-url https://ex-eg.io/foo foobar==1.0.0");
        on_build_rule_catches_not("DL3013", "RUN pip install --index-url https://eg.com/foo --extra-index-url https://ex-eg.io/foo foobar==1.0.0");
    }

    #[test]
    fn pip_install_no_cache_dir() {
        rule_catches_not("DL3013", "RUN pip install MySQL_python==1.2.2 --no-cache-dir");
        on_build_rule_catches_not("DL3013", "RUN pip install MySQL_python==1.2.2 --no-cache-dir");
    }

    #[test]
    fn pip_install_constraints_file_long_version_argument() {
        rule_catches_not("DL3013", "RUN pip install pykafka --constraint http://foo.bar.baz");
        on_build_rule_catches_not("DL3013", "RUN pip install pykafka --constraint http://foo.bar.baz");
    }

    #[test]
    fn pip_install_constraints_file_short_version_argument() {
        rule_catches_not("DL3013", "RUN pip install pykafka -c http://foo.bar.baz");
        on_build_rule_catches_not("DL3013", "RUN pip install pykafka -c http://foo.bar.baz");
    }

    #[test]
    fn pip_install_index_url_with_extra_index_url_with_basic_auth() {
        rule_catches_not("DL3013", "RUN pip install --index-url https://user:pass@eg.com/foo --extra-index-url https://user:pass@ex-eg.io/foo foobar==1.0.0");
        on_build_rule_catches_not("DL3013", "RUN pip install --index-url https://user:pass@eg.com/foo --extra-index-url https://user:pass@ex-eg.io/foo foobar==1.0.0");
    }

    #[test]
    fn pipenv_is_not_pip() {
        rule_catches_not("DL3013", "RUN pipenv install black");
        on_build_rule_catches_not("DL3013", "RUN pipenv install black");
    }

    #[test]
    fn pipx_version_not_pinned() {
        rule_catches("DL3013", "RUN pipx install black");
        on_build_rule_catches("DL3013", "RUN pipx install black");
    }

    #[test]
    fn pipx_install_python_argument_is_not_a_package() {
        rule_catches_not("DL3013", "RUN pipx install --python \"$(which python)\" \"poetry==1.8.5\"");
        on_build_rule_catches_not("DL3013", "RUN pipx install --python \"$(which python)\" \"poetry==1.8.5\"");
    }

    #[test]
    fn pip_install_root_user_action_argument_is_not_a_package() {
        rule_catches_not("DL3013", "RUN pip install --no-cache-dir --root-user-action ignore poetry==1.8.5");
        on_build_rule_catches_not("DL3013", "RUN pip install --no-cache-dir --root-user-action ignore poetry==1.8.5");
    }

    #[test]
    fn pip_install_flag_value_still_checks_the_package() {
        rule_catches("DL3013", "RUN pip install --root-user-action=ignore mypkg");
        on_build_rule_catches("DL3013", "RUN pip install --root-user-action=ignore mypkg");
    }
}
