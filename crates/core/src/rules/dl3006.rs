// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Rule/DL3006.hs.

use crate::rule::{add_fail, custom_rule, empty_state, modify, pipe, CheckFailure, DLSeverity, Rule};
use dlint_syntax::*;
use std::collections::BTreeSet;

/// Haskell `rule`.
pub fn rule() -> Box<dyn Rule> {
    let code = "DL3006";
    let severity = DLSeverity::DLWarningC;
    let message = "Always tag the version of an image explicitly";

    custom_rule(
        move |line, st, instr| match instr {
            Instruction::From(from) => {
                let failure = CheckFailure {
                    code: code.into(),
                    severity,
                    message: message.to_string(),
                    line,
                };
                let was_alias = |i: &str| st.state.contains(i);
                let untagged_ok = match &from.image {
                    Image { image_name: i, .. } if from.tag.is_none() && from.digest.is_none() && i != "scratch" => {
                        // When the image being used is a previously defined FROM alias,
                        // then we can safely ignore that the image is not tagged. Otherwise
                        // we marked it as a failure
                        Some(i.starts_with('$') || was_alias(i))
                    }
                    _ => None,
                };
                let new_state = pipe(st, |s| modify(|s| insert_from_alias(from, s), s));
                match untagged_ok {
                    Some(false) => add_fail(failure, new_state),
                    _ => new_state,
                }
            }
            _ => st,
        },
        empty_state(BTreeSet::new()),
    )
}

/// Haskell `insertFromAlias`.
fn insert_from_alias(from: &BaseImage, mut st: BTreeSet<String>) -> BTreeSet<String> {
    if let Some(a) = &from.alias {
        st.insert(a.un_image_alias.clone());
    }
    st
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn no_untagged() {
        rule_catches("DL3006", "FROM debian");
    }
    #[test]
    fn no_untagged_with_name() {
        rule_catches("DL3006", "FROM debian AS builder");
    }
    #[test]
    fn untagged_digest_is_not_an_error() {
        rule_catches_not("DL3006", "FROM ruby@sha256:f1dbca0f5dbc9");
    }
    #[test]
    fn untagged_digest_is_not_an_error_2() {
        rule_catches_not("DL3006", "FROM ruby:2");
    }
    #[test]
    fn using_args_is_not_an_error() {
        rule_catches_not("DL3006", "FROM ${VALUE}");
    }
    #[test]
    fn local_aliases_are_ok_to_be_untagged() {
        let d = [
            "FROM golang:1.9.3-alpine3.7 AS build",
            "RUN foo",
            "FROM build as unit-test",
            "RUN bar",
            "FROM alpine:3.7",
            "RUN baz",
        ]
        .join("\n")
            + "\n";
        rule_catches_not("DL3006", &d);
        on_build_rule_catches_not("DL3006", &d);
    }
    #[test]
    fn other_untagged_cases_are_not_ok() {
        let d = [
            "FROM golang:1.9.3-alpine3.7 AS build",
            "RUN foo",
            "FROM node as unit-test",
            "RUN bar",
            "FROM alpine:3.7",
            "RUN baz",
        ]
        .join("\n")
            + "\n";
        rule_catches("DL3006", &d);
        on_build_rule_catches("DL3006", &d);
    }
    #[test]
    fn scratch() {
        rule_catches_not("DL3006", "FROM scratch");
    }
}
