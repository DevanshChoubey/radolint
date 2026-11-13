// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Run.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use std::collections::BTreeSet;
use dlint_syntax::*;
use crate::combinator::{PResult, Parser};
use crate::prelude::DockerfileError;

/// Haskell `data RunFlag` (Run.hs, module-private upstream).
#[derive(Debug, Clone)]
pub enum RunFlag {
    RunFlagMount(RunMount),
    RunFlagSecurity(RunSecurity),
    RunFlagNetwork(RunNetwork),
}

/// Haskell `data RunMountArg` (Run.hs, module-private upstream).
#[derive(Debug, Clone)]
pub enum RunMountArg {
    MountArgEnv(String),
    MountArgFromImage(String),
    MountArgId(String),
    MountArgMode(String),
    MountArgReadOnly(bool),
    MountArgRequired(bool),
    MountArgSharing(CacheSharing),
    MountArgSize(String),
    MountArgSource(SourcePath),
    MountArgTarget(TargetPath),
    MountArgType(MountType),
    MountArgUid(String),
    MountArgGid(String),
    MountArgRelabel(Relabel),
}

/// Haskell `data MountType` (Run.hs, module-private upstream).
#[derive(Debug, Clone)]
pub enum MountType {
    Bind,
    Cache,
    Tmpfs,
    Secret,
    Ssh,
}

/// Haskell `parseRun` (Run.hs).
pub fn parse_run(p: &mut Parser) -> PResult<Instruction<String>> {
    todo!("port Run.hs parseRun")
}

/// Haskell `runArguments` (Run.hs).
pub fn run_arguments(p: &mut Parser) -> PResult<RunArgs<String>> {
    todo!("port Run.hs runArguments")
}

/// Haskell `runFlags` (Run.hs).
pub fn run_flags(p: &mut Parser) -> PResult<RunFlags> {
    todo!("port Run.hs runFlags")
}

/// Haskell `runFlag` (Run.hs).
pub fn run_flag(p: &mut Parser) -> PResult<RunFlag> {
    todo!("port Run.hs runFlag")
}

/// Haskell `runFlagSecurity` (Run.hs).
pub fn run_flag_security(p: &mut Parser) -> PResult<RunSecurity> {
    todo!("port Run.hs runFlagSecurity")
}

/// Haskell `runFlagNetwork` (Run.hs).
pub fn run_flag_network(p: &mut Parser) -> PResult<RunNetwork> {
    todo!("port Run.hs runFlagNetwork")
}

/// Haskell `runFlagMount` (Run.hs).
pub fn run_flag_mount(p: &mut Parser) -> PResult<RunMount> {
    todo!("port Run.hs runFlagMount")
}

/// Haskell `parseTypeFromArgs` (Run.hs).
pub fn parse_type_from_args(p: &mut Parser, args: &[RunMountArg]) -> PResult<MountType> {
    todo!("port Run.hs parseTypeFromArgs")
}

/// Haskell `isMountArgType` (Run.hs).
pub fn is_mount_arg_type(arg: &RunMountArg) -> bool {
    todo!("port Run.hs isMountArgType")
}

/// Haskell `bindMount` (Run.hs).
pub fn bind_mount(p: &mut Parser, args: &[RunMountArg]) -> PResult<BindOpts> {
    todo!("port Run.hs bindMount")
}

/// Haskell `cacheMount` (Run.hs).
pub fn cache_mount(p: &mut Parser, args: &[RunMountArg]) -> PResult<CacheOpts> {
    todo!("port Run.hs cacheMount")
}

/// Haskell `tmpfsMount` (Run.hs).
pub fn tmpfs_mount(p: &mut Parser, args: &[RunMountArg]) -> PResult<TmpOpts> {
    todo!("port Run.hs tmpfsMount")
}

/// Haskell `secretMount` (Run.hs).
pub fn secret_mount(p: &mut Parser, args: &[RunMountArg]) -> PResult<SecretOpts> {
    todo!("port Run.hs secretMount")
}

/// Haskell `validArgs` (Run.hs).
pub fn valid_args(type_name: &str, allowed: &BTreeSet<String>, required: &BTreeSet<String>, args: &[RunMountArg]) -> Result<Vec<RunMountArg>, DockerfileError> {
    todo!("port Run.hs validArgs")
}

/// Haskell `mountArgs` (Run.hs).
pub fn mount_args(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgs")
}

/// Haskell `stringArg` (Run.hs).
pub fn string_arg(p: &mut Parser) -> PResult<String> {
    todo!("port Run.hs stringArg")
}

/// Haskell `key` (Run.hs).
pub fn key<T>(p: &mut Parser, name: &str, parser: impl Fn(&mut Parser) -> PResult<T>) -> PResult<T> {
    todo!("port Run.hs key")
}

/// Haskell `tryKeyValue'` (Run.hs).
pub fn try_key_value_(p: &mut Parser, k: &str, v: &str) -> PResult<String> {
    todo!("port Run.hs tryKeyValue'")
}

/// Haskell `cacheSharing` (Run.hs).
pub fn cache_sharing(p: &mut Parser) -> PResult<CacheSharing> {
    todo!("port Run.hs cacheSharing")
}

/// Haskell `mountArgEnv` (Run.hs).
pub fn mount_arg_env(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgEnv")
}

/// Haskell `mountArgFromImage` (Run.hs).
pub fn mount_arg_from_image(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgFromImage")
}

/// Haskell `mountArgGid` (Run.hs).
pub fn mount_arg_gid(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgGid")
}

/// Haskell `mountArgId` (Run.hs).
pub fn mount_arg_id(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgId")
}

/// Haskell `mountArgMode` (Run.hs).
pub fn mount_arg_mode(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgMode")
}

/// Haskell `mountArgReadOnly` (Run.hs).
pub fn mount_arg_read_only(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgReadOnly")
}

/// Haskell `mountArgRequired` (Run.hs).
pub fn mount_arg_required(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgRequired")
}

/// Haskell `mountArgSharing` (Run.hs).
pub fn mount_arg_sharing(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgSharing")
}

/// Haskell `mountArgSize` (Run.hs).
pub fn mount_arg_size(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgSize")
}

/// Haskell `mountArgSource` (Run.hs).
pub fn mount_arg_source(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgSource")
}

/// Haskell `mountArgTarget` (Run.hs).
pub fn mount_arg_target(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgTarget")
}

/// Haskell `mountArgType` (Run.hs).
pub fn mount_arg_type(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgType")
}

/// Haskell `mountArgUid` (Run.hs).
pub fn mount_arg_uid(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgUid")
}

/// Haskell `mountArgRelabel` (Run.hs).
pub fn mount_arg_relabel(p: &mut Parser) -> PResult<RunMountArg> {
    todo!("port Run.hs mountArgRelabel")
}

/// Haskell `mountType` (Run.hs).
pub fn mount_type(p: &mut Parser) -> PResult<MountType> {
    todo!("port Run.hs mountType")
}

/// Haskell `relabel` (Run.hs).
pub fn relabel(p: &mut Parser) -> PResult<Relabel> {
    todo!("port Run.hs relabel")
}

/// Haskell `toArgName` (Run.hs).
pub fn to_arg_name(arg: &RunMountArg) -> String {
    todo!("port Run.hs toArgName")
}

