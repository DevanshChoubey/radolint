// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3064.hs.

use crate::rule::{combine, onbuild, simple_rule, DLSeverity, Rule};
use dlint_syntax::*;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    combine(dl3064(), onbuild(dl3064()))
}

/// Haskell `dl3064`.
pub fn dl3064() -> Box<dyn Rule> {
    let code = "DL3064";
    let severity = DLSeverity::DLWarningC;
    let message = "Potentially sensitive data should not be used in the `ARG` or `ENV` commands";
    let check = |instr: &Instruction<_>| match instr {
        Instruction::Arg(name, _) => {
            !known_sensitive_names().contains(&name.to_uppercase().as_str())
                && none(|s: &&str| name.to_lowercase().contains(*s), suspicious_substrings())
        }
        Instruction::Env(pairs) => {
            none(|s: &(String, String)| known_sensitive_names().contains(&s.0.to_uppercase().as_str()), pairs)
                && none(
                    |s: &&str| pairs.iter().any(|p| p.0.to_lowercase().contains(*s)),
                    suspicious_substrings(),
                )
        }
        _ => true,
    };
    simple_rule(code, severity, message, check)
}

/// Haskell `none`.
fn none<T>(p: impl Fn(&T) -> bool, xs: &[T]) -> bool {
    !xs.iter().any(|x| p(x))
}

/// Haskell `knownSensitiveNames`.
fn known_sensitive_names() -> &'static [&'static str] {
    &[
    "ACCESS_TOKEN",
    "APPLICATION_KEY",
    "APP_SECRET",
    "AUTH_TOKEN",
    "AWS_ACCESS_KEY_ID",
    "AWS_SECRET_ACCESS_KEY",
    "BITTREX_API_KEY",
    "BITTREX_API_SECRET",
    "CF_PASSWORD",
    "CF_USERNAME",
    "CIRCLE_TOKEN",
    "CI_DEPLOY_PASSWORD",
    "CI_DEPLOY_USER",
    "DOCKERHUB_PASSWORD",
    "DOCKER_EMAIL",
    "DOCKER_PASSWORD",
    "DOCKER_USERNAME",
    "FACEBOOK_ACCESS_TOKEN",
    "FACEBOOK_APP_ID",
    "FACEBOOK_APP_SECRET",
    "FIREBASE_API_TOKEN",
    "FIREBASE_TOKEN",
    "FOSSA_API_KEY",
    "GH_ENTERPRISE_TOKEN",
    "GH_TOKEN",
    "GITHUB_ENTERPRISE_TOKEN",
    "GITHUB_TOKEN",
    "HEROKU_API_KEY",
    "HEROKU_API_USER",
    "NPM_AUTH_TOKEN",
    "NPM_TOKEN",
    "OKTA_AUTHN_GROUPID",
    "OKTA_CLIENT_ORGURL",
    "OKTA_CLIENT_TOKEN",
    "OKTA_OAUTH2_CLIENTID",
    "OKTA_OAUTH2_CLIENTSECRET",
    "OPENAI_API_KEY",
    "OS_PASSWORD",
    "OS_USERNAME",
    "POSTGRES_PASSWORD",
    "SLACK_TOKEN",
    "STRIPE_API_KEY",
    "STRIPE_DEVICE_NAME",
    "TRAVIS_OS_NAME",
    "TRAVIS_SECURE_ENV_VARS",
    "TRAVIS_SUDO",
    "VAULT_CLIENT_KEY",
    "VAULT_TOKEN",
    ]
}

/// Haskell `suspiciousSubstrings`.
fn suspicious_substrings() -> &'static [&'static str] {
    &["api_key", "client_key", "password", "private", "secret", "token", "username"]
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    fn both_not(d: &str) {
        rule_catches_not("DL3064", d);
        on_build_rule_catches_not("DL3064", d);
    }

    fn both(d: &str) {
        rule_catches("DL3064", d);
        on_build_rule_catches("DL3064", d);
    }

    #[test]
    fn ok_no_arg_no_env() {
        both_not("FROM debian:bullseye\nRUN foobar\n");
    }

    #[test]
    fn ok_arg_no_sensitive_data_no_env() {
        both_not("ARG foobar\nFROM debian:bullseye\nRUN foobar\n");
    }

    #[test]
    fn ok_no_arg_env_no_sensitive_data() {
        both_not("FROM debian:bullseye\nENV foobar=barfoo\nRUN foobar\n");
    }

    #[test]
    fn not_ok_arg_with_sensitive_data() {
        both("ARG AWS_ACCESS_KEY_ID\nFROM debian:bullseye\nRUN foobar\n");
    }

    #[test]
    fn not_ok_arg_with_sensitive_data_different_casing() {
        both("ARG openai_api_key\nFROM debian:bullseye\nRUN foobar\n");
    }

    #[test]
    fn not_ok_env_with_sensitive_data() {
        both("FROM debian:bullseye\nENV AWS_ACCESS_KEY_ID=abcdefghijklmnopqrstuvwxyz\nRUN foobar\n");
    }

    #[test]
    fn not_ok_env_with_sensitive_data_different_casing() {
        both("FROM debian:bullseye\nENV my_password=abcdefghijklmnopqrstuvwxyz\nRUN foobar\n");
    }
}
