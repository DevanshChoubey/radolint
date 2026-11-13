// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Run.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use crate::arguments::arguments;
use crate::combinator::{PResult, Parser};
use crate::prelude::{
    DockerfileError, custom_error, double_quoted_string, required_whitespace, reserved, some_unless,
};
use dlint_syntax::*;
use std::collections::BTreeSet;

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
    reserved(p, "RUN")?;
    Ok(Instruction::Run(run_arguments(p)?))
}

/// Haskell `runArguments` (Run.hs).
pub fn run_arguments(p: &mut Parser) -> PResult<RunArgs<String>> {
    let present_flags = p.alt(
        |p| {
            let f = run_flags(p)?;
            required_whitespace(p)?;
            Ok(f)
        },
        |_p| {
            Ok(RunFlags {
                mount: BTreeSet::new(),
                security: None,
                network: None,
            })
        },
    )?;
    let args = arguments(p)?;
    Ok(RunArgs(args, present_flags))
}

/// Haskell `runFlags` (Run.hs).
pub fn run_flags(p: &mut Parser) -> PResult<RunFlags> {
    fn flag_separator(p: &mut Parser) -> PResult<String> {
        p.alt(
            |p| {
                p.try_(|p| {
                    required_whitespace(p)?;
                    p.look_ahead(|p| p.string("--"))
                })
            },
            |p| p.fail("expected flag"),
        )
    }
    let flags = p.sep_by(run_flag, flag_separator)?;
    let mut acc = RunFlags {
        mount: BTreeSet::new(),
        security: None,
        network: None,
    };
    // foldr toRunFlags emptyFlags flags
    for f in flags.into_iter().rev() {
        match f {
            RunFlag::RunFlagMount(m) => {
                acc.mount.insert(m);
            }
            RunFlag::RunFlagNetwork(n) => acc.network = Some(n),
            RunFlag::RunFlagSecurity(s) => acc.security = Some(s),
        }
    }
    Ok(acc)
}

/// Haskell `runFlag` (Run.hs).
pub fn run_flag(p: &mut Parser) -> PResult<RunFlag> {
    p.choice(&[
        &|p: &mut Parser| Ok(RunFlag::RunFlagMount(run_flag_mount(p)?)),
        &|p: &mut Parser| Ok(RunFlag::RunFlagSecurity(run_flag_security(p)?)),
        &|p: &mut Parser| Ok(RunFlag::RunFlagNetwork(run_flag_network(p)?)),
    ])
}

/// Haskell `runFlagSecurity` (Run.hs).
pub fn run_flag_security(p: &mut Parser) -> PResult<RunSecurity> {
    p.string("--security=")?;
    p.choice(&[
        &|p: &mut Parser| p.string("insecure").map(|_| RunSecurity::Insecure),
        &|p: &mut Parser| p.string("sandbox").map(|_| RunSecurity::Sandbox),
    ])
}

/// Haskell `runFlagNetwork` (Run.hs).
pub fn run_flag_network(p: &mut Parser) -> PResult<RunNetwork> {
    p.string("--network=")?;
    p.choice(&[
        &|p: &mut Parser| p.string("none").map(|_| RunNetwork::NetworkNone),
        &|p: &mut Parser| p.string("host").map(|_| RunNetwork::NetworkHost),
        &|p: &mut Parser| p.string("default").map(|_| RunNetwork::NetworkDefault),
    ])
}

/// Haskell `runFlagMount` (Run.hs).
pub fn run_flag_mount(p: &mut Parser) -> PResult<RunMount> {
    p.string("--mount=")?;
    let args = p.sep_by1(mount_args, |p| p.string(","))?;
    let mt = parse_type_from_args(p, &args)?;
    let rest: Vec<RunMountArg> = args
        .iter()
        .filter(|a| !is_mount_arg_type(a))
        .cloned()
        .collect();
    match mt {
        MountType::Bind => Ok(RunMount::BindMount(bind_mount(p, &rest)?)),
        MountType::Cache => Ok(RunMount::CacheMount(cache_mount(p, &rest)?)),
        MountType::Tmpfs => Ok(RunMount::TmpfsMount(tmpfs_mount(p, &rest)?)),
        MountType::Secret => Ok(RunMount::SecretMount(secret_mount(p, &rest)?)),
        MountType::Ssh => Ok(RunMount::SshMount(secret_mount(p, &rest)?)),
    }
}

/// Haskell `parseTypeFromArgs` (Run.hs).
pub fn parse_type_from_args(p: &mut Parser, args: &[RunMountArg]) -> PResult<MountType> {
    // `notFollowedBy eof` is a trivially succeeding parser that isn't supposed to
    // consume any input here (see the Haskell comment).
    let types: Vec<&RunMountArg> = args.iter().filter(|a| is_mount_arg_type(a)).collect();
    match types.as_slice() {
        [] => {
            p.not_followed_by(|p| p.eof())?;
            Ok(MountType::Bind)
        }
        [RunMountArg::MountArgType(t)] => {
            p.not_followed_by(|p| p.eof())?;
            Ok(t.clone())
        }
        _ => p.fail("--mount with multiple `type` arguments"),
    }
}

/// Haskell `isMountArgType` (Run.hs).
pub fn is_mount_arg_type(arg: &RunMountArg) -> bool {
    matches!(arg, RunMountArg::MountArgType(_))
}

fn names(xs: &[&str]) -> BTreeSet<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

fn unhandled(invalid: &RunMountArg) -> ! {
    panic!("unhandled {:?} please report this bug", invalid)
}

/// Haskell `bindMount` (Run.hs).
pub fn bind_mount(p: &mut Parser, args: &[RunMountArg]) -> PResult<BindOpts> {
    let allowed = names(&["target", "source", "from", "ro", "relabel"]);
    let required = names(&["target"]);
    fn bind_opts(a: &RunMountArg, mut bo: BindOpts) -> BindOpts {
        match a {
            RunMountArg::MountArgTarget(path) => bo.b_target = path.clone(),
            RunMountArg::MountArgSource(path) => bo.b_source = Some(path.clone()),
            RunMountArg::MountArgFromImage(img) => bo.b_from_image = Some(img.clone()),
            RunMountArg::MountArgReadOnly(ro) => bo.b_read_only = Some(*ro),
            RunMountArg::MountArgRelabel(re) => bo.b_relabel = Some(*re),
            invalid => unhandled(invalid),
        }
        bo
    }
    match valid_args("bind", &allowed, &required, args) {
        Err(e) => custom_error(p, e),
        Ok(as_) => Ok(as_
            .iter()
            .rev()
            .fold(BindOpts::default(), |acc, a| bind_opts(a, acc))),
    }
}

/// Haskell `cacheMount` (Run.hs).
pub fn cache_mount(p: &mut Parser, args: &[RunMountArg]) -> PResult<CacheOpts> {
    let allowed = names(&[
        "target", "sharing", "id", "ro", "from", "source", "mode", "uid", "gid",
    ]);
    let required = names(&["target"]);
    fn cache_opts(a: &RunMountArg, mut co: CacheOpts) -> CacheOpts {
        match a {
            RunMountArg::MountArgTarget(path) => co.c_target = path.clone(),
            RunMountArg::MountArgSharing(sh) => co.c_sharing = Some(*sh),
            RunMountArg::MountArgId(i) => co.c_cache_id = Some(i.clone()),
            RunMountArg::MountArgReadOnly(ro) => co.c_read_only = Some(*ro),
            RunMountArg::MountArgFromImage(img) => co.c_from_image = Some(img.clone()),
            RunMountArg::MountArgSource(path) => co.c_source = Some(path.clone()),
            RunMountArg::MountArgMode(m) => co.c_mode = Some(m.clone()),
            RunMountArg::MountArgUid(u) => co.c_uid = Some(u.clone()),
            RunMountArg::MountArgGid(g) => co.c_gid = Some(g.clone()),
            invalid => unhandled(invalid),
        }
        co
    }
    match valid_args("cache", &allowed, &required, args) {
        Err(e) => custom_error(p, e),
        Ok(as_) => Ok(as_
            .iter()
            .rev()
            .fold(CacheOpts::default(), |acc, a| cache_opts(a, acc))),
    }
}

/// Haskell `tmpfsMount` (Run.hs).
pub fn tmpfs_mount(p: &mut Parser, args: &[RunMountArg]) -> PResult<TmpOpts> {
    let allowed = names(&["target", "size"]);
    let required = names(&["target"]);
    fn tmp_opts(a: &RunMountArg, mut t: TmpOpts) -> TmpOpts {
        match a {
            RunMountArg::MountArgTarget(path) => t.t_target = path.clone(),
            RunMountArg::MountArgSize(size) => t.t_size = Some(size.clone()),
            invalid => unhandled(invalid),
        }
        t
    }
    match valid_args("tmpfs", &allowed, &required, args) {
        Err(e) => custom_error(p, e),
        Ok(as_) => Ok(as_
            .iter()
            .rev()
            .fold(TmpOpts::default(), |acc, a| tmp_opts(a, acc))),
    }
}

/// Haskell `secretMount` (Run.hs).
pub fn secret_mount(p: &mut Parser, args: &[RunMountArg]) -> PResult<SecretOpts> {
    let allowed = names(&[
        "target", "id", "required", "source", "mode", "uid", "gid", "env",
    ]);
    let required = BTreeSet::new();
    fn secret_opts(a: &RunMountArg, mut co: SecretOpts) -> SecretOpts {
        match a {
            RunMountArg::MountArgTarget(path) => co.s_target = Some(path.clone()),
            RunMountArg::MountArgId(i) => co.s_cache_id = Some(i.clone()),
            RunMountArg::MountArgRequired(r) => co.s_is_required = Some(*r),
            RunMountArg::MountArgSource(path) => co.s_source = Some(path.clone()),
            RunMountArg::MountArgEnv(e) => co.s_env = Some(e.clone()),
            RunMountArg::MountArgMode(m) => co.s_mode = Some(m.clone()),
            RunMountArg::MountArgUid(u) => co.s_uid = Some(u.clone()),
            RunMountArg::MountArgGid(g) => co.s_gid = Some(g.clone()),
            invalid => unhandled(invalid),
        }
        co
    }
    match valid_args("secret", &allowed, &required, args) {
        Err(e) => custom_error(p, e),
        Ok(as_) => Ok(as_
            .iter()
            .rev()
            .fold(SecretOpts::default(), |acc, a| secret_opts(a, acc))),
    }
}

/// Haskell `validArgs` (Run.hs).
pub fn valid_args(
    type_name: &str,
    allowed: &BTreeSet<String>,
    required: &BTreeSet<String>,
    args: &[RunMountArg],
) -> Result<Vec<RunMountArg>, DockerfileError> {
    type Acc = (Result<Vec<RunMountArg>, DockerfileError>, BTreeSet<String>);
    let check_valid_arg = |a: &RunMountArg, acc: Acc| -> Acc {
        let (res, mut seen) = acc;
        let as_ = match res {
            Err(e) => return (Err(e), seen),
            Ok(as_) => as_,
        };
        let name = to_arg_name(a);
        match (allowed.contains(&name), seen.contains(&name)) {
            (false, _) => (
                Err(DockerfileError::UnexpectedArgument(
                    name,
                    type_name.to_string(),
                )),
                seen,
            ),
            (_, true) => (Err(DockerfileError::DuplicateArgument(name)), seen),
            (true, false) => {
                seen.insert(name);
                let mut v = vec![a.clone()];
                v.extend(as_);
                (Ok(v), seen)
            }
        }
    };
    // foldr checkValidArg (Right [], Set.empty) args
    let (result, seen) = args
        .iter()
        .rev()
        .fold((Ok(vec![]), BTreeSet::new()), |acc, a| {
            check_valid_arg(a, acc)
        });
    let missing: Vec<String> = required.difference(&seen).cloned().collect();
    if missing.is_empty() {
        result
    } else {
        Err(DockerfileError::MissingArgument(missing))
    }
}

/// Haskell `mountArgs` (Run.hs).
pub fn mount_args(p: &mut Parser) -> PResult<RunMountArg> {
    p.choice(&[
        &mount_arg_env,
        &mount_arg_from_image,
        &mount_arg_gid,
        &mount_arg_id,
        &mount_arg_mode,
        &mount_arg_read_only,
        &mount_arg_relabel,
        &mount_arg_required,
        &mount_arg_sharing,
        &mount_arg_size,
        &mount_arg_source,
        &mount_arg_target,
        &mount_arg_type,
        &mount_arg_uid,
    ])
}

/// Haskell `stringArg` (Run.hs).
pub fn string_arg(p: &mut Parser) -> PResult<String> {
    p.choice(&[&double_quoted_string, &|p: &mut Parser| {
        some_unless(p, "a string", &|c| c == ',')
    }])
}

/// Haskell `key` (Run.hs).
pub fn key<T>(
    p: &mut Parser,
    name: &str,
    parser: impl Fn(&mut Parser) -> PResult<T>,
) -> PResult<T> {
    p.string(&format!("{name}="))?;
    parser(p)
}

/// Haskell `tryKeyValue'` (Run.hs).
pub fn try_key_value_(p: &mut Parser, k: &str, v: &str) -> PResult<String> {
    p.try_(|p| {
        p.string_ci(&format!("{k}="))?;
        p.string_ci(v)
    })
}

/// Haskell `cacheSharing` (Run.hs).
pub fn cache_sharing(p: &mut Parser) -> PResult<CacheSharing> {
    p.choice(&[
        &|p: &mut Parser| p.string("private").map(|_| CacheSharing::Private),
        &|p: &mut Parser| p.string("shared").map(|_| CacheSharing::Shared),
        &|p: &mut Parser| p.string("locked").map(|_| CacheSharing::Locked),
    ])
}

/// Haskell `mountArgEnv` (Run.hs).
pub fn mount_arg_env(p: &mut Parser) -> PResult<RunMountArg> {
    Ok(RunMountArg::MountArgEnv(key(p, "env", string_arg)?))
}

/// Haskell `mountArgFromImage` (Run.hs).
pub fn mount_arg_from_image(p: &mut Parser) -> PResult<RunMountArg> {
    Ok(RunMountArg::MountArgFromImage(key(p, "from", string_arg)?))
}

/// Haskell `mountArgGid` (Run.hs).
pub fn mount_arg_gid(p: &mut Parser) -> PResult<RunMountArg> {
    Ok(RunMountArg::MountArgGid(key(p, "gid", string_arg)?))
}

/// Haskell `mountArgId` (Run.hs).
pub fn mount_arg_id(p: &mut Parser) -> PResult<RunMountArg> {
    Ok(RunMountArg::MountArgId(key(p, "id", string_arg)?))
}

/// Haskell `mountArgMode` (Run.hs).
pub fn mount_arg_mode(p: &mut Parser) -> PResult<RunMountArg> {
    Ok(RunMountArg::MountArgMode(key(p, "mode", string_arg)?))
}

/// Haskell `mountArgReadOnly` (Run.hs).
pub fn mount_arg_read_only(p: &mut Parser) -> PResult<RunMountArg> {
    let choice_ro_explicit = |p: &mut Parser| -> PResult<bool> {
        p.choice(&[
            &|p: &mut Parser| try_key_value_(p, "ro", "true"),
            &|p: &mut Parser| try_key_value_(p, "rw", "false"),
            &|p: &mut Parser| try_key_value_(p, "readonly", "true"),
            &|p: &mut Parser| try_key_value_(p, "readwrite", "false"),
        ])?;
        Ok(true)
    };
    let choice_rw_explicit = |p: &mut Parser| -> PResult<bool> {
        p.choice(&[
            &|p: &mut Parser| try_key_value_(p, "rw", "true"),
            &|p: &mut Parser| try_key_value_(p, "ro", "false"),
            &|p: &mut Parser| try_key_value_(p, "readwrite", "true"),
            &|p: &mut Parser| try_key_value_(p, "readonly", "false"),
        ])?;
        Ok(false)
    };
    let choice_ro = |p: &mut Parser| -> PResult<bool> {
        p.choice(&[&|p: &mut Parser| p.string_ci("ro"), &|p: &mut Parser| {
            p.string_ci("readonly")
        }])?;
        Ok(true)
    };
    let choice_rw = |p: &mut Parser| -> PResult<bool> {
        p.choice(&[&|p: &mut Parser| p.string_ci("rw"), &|p: &mut Parser| {
            p.string_ci("readwrite")
        }])?;
        Ok(false)
    };
    // these two must come last and be separate
    let b = p.choice(&[
        &choice_ro_explicit,
        &choice_rw_explicit,
        &choice_ro,
        &choice_rw,
    ])?;
    Ok(RunMountArg::MountArgReadOnly(b))
}

/// Haskell `mountArgRequired` (Run.hs).
pub fn mount_arg_required(p: &mut Parser) -> PResult<RunMountArg> {
    let b = p.choice(&[
        &|p: &mut Parser| {
            p.choice(&[
                &|p: &mut Parser| p.string("required=true"),
                &|p: &mut Parser| p.string("required=True"),
            ])?;
            Ok(true)
        },
        &|p: &mut Parser| {
            p.choice(&[
                &|p: &mut Parser| p.string("required=false"),
                &|p: &mut Parser| p.string("required=False"),
            ])?;
            Ok(false)
        },
        // This must come last in the list!
        &|p: &mut Parser| {
            p.string("required")?;
            Ok(true)
        },
    ])?;
    Ok(RunMountArg::MountArgRequired(b))
}

/// Haskell `mountArgSharing` (Run.hs).
pub fn mount_arg_sharing(p: &mut Parser) -> PResult<RunMountArg> {
    Ok(RunMountArg::MountArgSharing(key(
        p,
        "sharing",
        cache_sharing,
    )?))
}

/// Haskell `mountArgSize` (Run.hs).
pub fn mount_arg_size(p: &mut Parser) -> PResult<RunMountArg> {
    Ok(RunMountArg::MountArgSize(key(p, "size", string_arg)?))
}

/// Haskell `mountArgSource` (Run.hs).
pub fn mount_arg_source(p: &mut Parser) -> PResult<RunMountArg> {
    p.label("source=", |p| {
        p.choice(&[&|p: &mut Parser| p.string("source="), &|p: &mut Parser| {
            p.string("src=")
        }])
    })?;
    Ok(RunMountArg::MountArgSource(SourcePath {
        un_source_path: string_arg(p)?,
    }))
}

/// Haskell `mountArgTarget` (Run.hs).
pub fn mount_arg_target(p: &mut Parser) -> PResult<RunMountArg> {
    p.label("target=", |p| {
        p.choice(&[
            &|p: &mut Parser| p.string("target="),
            &|p: &mut Parser| p.string("dst="),
            &|p: &mut Parser| p.string("destination="),
        ])
    })?;
    Ok(RunMountArg::MountArgTarget(TargetPath {
        un_target_path: string_arg(p)?,
    }))
}

/// Haskell `mountArgType` (Run.hs).
pub fn mount_arg_type(p: &mut Parser) -> PResult<RunMountArg> {
    Ok(RunMountArg::MountArgType(key(p, "type", mount_type)?))
}

/// Haskell `mountArgUid` (Run.hs).
pub fn mount_arg_uid(p: &mut Parser) -> PResult<RunMountArg> {
    Ok(RunMountArg::MountArgUid(key(p, "uid", string_arg)?))
}

/// Haskell `mountArgRelabel` (Run.hs).
pub fn mount_arg_relabel(p: &mut Parser) -> PResult<RunMountArg> {
    let r = p.choice(&[
        &|p: &mut Parser| key(p, "relabel", relabel),
        &|p: &mut Parser| p.string("z").map(|_| Relabel::RelabelShared),
        &|p: &mut Parser| p.string("Z").map(|_| Relabel::RelabelPrivate),
    ])?;
    Ok(RunMountArg::MountArgRelabel(r))
}

/// Haskell `mountType` (Run.hs).
pub fn mount_type(p: &mut Parser) -> PResult<MountType> {
    p.choice(&[
        &|p: &mut Parser| p.string("bind").map(|_| MountType::Bind),
        &|p: &mut Parser| p.string("cache").map(|_| MountType::Cache),
        &|p: &mut Parser| p.string("tmpfs").map(|_| MountType::Tmpfs),
        &|p: &mut Parser| p.string("secret").map(|_| MountType::Secret),
        &|p: &mut Parser| p.string("ssh").map(|_| MountType::Ssh),
    ])
}

/// Haskell `relabel` (Run.hs).
pub fn relabel(p: &mut Parser) -> PResult<Relabel> {
    p.choice(&[
        &|p: &mut Parser| p.string("shared").map(|_| Relabel::RelabelShared),
        &|p: &mut Parser| p.string("private").map(|_| Relabel::RelabelPrivate),
    ])
}

/// Haskell `toArgName` (Run.hs).
pub fn to_arg_name(arg: &RunMountArg) -> String {
    use RunMountArg::*;
    match arg {
        MountArgEnv(_) => "env",
        MountArgFromImage(_) => "from",
        MountArgGid(_) => "gid",
        MountArgId(_) => "id",
        MountArgMode(_) => "mode",
        MountArgReadOnly(_) => "ro",
        MountArgRequired(_) => "required",
        MountArgSharing(_) => "sharing",
        MountArgSize(_) => "size",
        MountArgSource(_) => "source",
        MountArgTarget(_) => "target",
        MountArgType(_) => "type",
        MountArgUid(_) => "uid",
        MountArgRelabel(_) => "relabel",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_text;

    fn tp(s: &str) -> TargetPath {
        TargetPath {
            un_target_path: s.to_string(),
        }
    }

    fn sp(s: &str) -> SourcePath {
        SourcePath {
            un_source_path: s.to_string(),
        }
    }

    fn mounts(ms: Vec<RunMount>) -> RunFlags {
        RunFlags {
            mount: ms.into_iter().collect(),
            ..Default::default()
        }
    }

    fn run(text: &str, flags: RunFlags) -> Instruction<String> {
        Instruction::Run(RunArgs(Arguments::ArgumentsText(text.to_string()), flags))
    }

    /// Haskell `assertAst` (TestHelper.hs).
    fn assert_ast(s: &str, ast: Vec<Instruction<String>>) {
        match parse_text(s) {
            Err(e) => panic!("{}", e.bundle_pretty()),
            Ok(d) => {
                let got: Vec<Instruction<String>> = d.into_iter().map(|i| i.instruction).collect();
                assert_eq!(ast, got, "ASTs are not equal");
            }
        }
    }

    #[test]
    fn escaped_with_space_before() {
        assert_ast(
            "RUN yum install -y \\\nimagemagick \\\nmysql\n",
            vec![run("yum install -y imagemagick mysql", RunFlags::default())],
        );
    }

    #[test]
    fn escaped_linebreak_indented() {
        assert_ast(
            "RUN foo ; \\\n    bar\n",
            vec![run("foo ;  bar", RunFlags::default())],
        );
    }

    #[test]
    fn does_not_choke_on_unmatched_brackets() {
        assert_ast("RUN [foo\n", vec![run("[foo", RunFlags::default())]);
    }

    #[test]
    fn distinguishes_between_text_and_a_list() {
        assert_ast(
            "RUN echo foo\nRUN [\"echo\", \"foo\"]\n",
            vec![
                Instruction::Run(RunArgs(
                    Arguments::ArgumentsText("echo foo".to_string()),
                    RunFlags::default(),
                )),
                Instruction::Run(RunArgs(
                    Arguments::ArgumentsList("echo foo".to_string()),
                    RunFlags::default(),
                )),
            ],
        );
    }

    #[test]
    fn accepts_spaces_inside_the_brackets() {
        assert_ast(
            "RUN [  \"echo\", \"foo\"  ]\n",
            vec![Instruction::Run(RunArgs(
                Arguments::ArgumentsList("echo foo".to_string()),
                RunFlags::default(),
            ))],
        );
    }

    #[test]
    fn mount_type_bind_and_target() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_default_to_bind() {
        assert_ast(
            "RUN --mount=target=/foo echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_all_modifiers() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,from=ubuntu,ro,relabel=shared echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_from_image: Some("ubuntu".to_string()),
                    b_read_only: Some(true),
                    b_relabel: Some(Relabel::RelabelShared),
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_cache_with_target() {
        assert_ast(
            "RUN --mount=type=cache,target=/foo echo foo\nRUN --mount=type=cache,target=/bar echo foo\nRUN --mount=type=cache,target=/baz echo foo\n",
            vec![
                run(
                    "echo foo",
                    mounts(vec![RunMount::CacheMount(CacheOpts {
                        c_target: tp("/foo"),
                        ..Default::default()
                    })]),
                ),
                run(
                    "echo foo",
                    mounts(vec![RunMount::CacheMount(CacheOpts {
                        c_target: tp("/bar"),
                        ..Default::default()
                    })]),
                ),
                run(
                    "echo foo",
                    mounts(vec![RunMount::CacheMount(CacheOpts {
                        c_target: tp("/baz"),
                        ..Default::default()
                    })]),
                ),
            ],
        );
    }

    #[test]
    fn mount_type_cache_with_id_from_variable_arg() {
        assert_ast(
            "ARG debian_version=12\nFROM debian:bookworm-slim\nRUN --mount=id=debian:${debian_version},type=cache,target=/foo foo bar\n",
            vec![
                Instruction::Arg("debian_version".to_string(), Some("12".to_string())),
                Instruction::From(BaseImage {
                    image: Image {
                        registry_name: None,
                        image_name: "debian".to_string(),
                    },
                    tag: Some(Tag {
                        un_tag: "bookworm-slim".to_string(),
                    }),
                    digest: None,
                    alias: None,
                    platform: None,
                }),
                run(
                    "foo bar",
                    mounts(vec![RunMount::CacheMount(CacheOpts {
                        c_target: tp("/foo"),
                        c_cache_id: Some("debian:${debian_version}".to_string()),
                        ..Default::default()
                    })]),
                ),
            ],
        );
    }

    #[test]
    fn mount_type_cache_dst_foo() {
        assert_ast(
            "RUN --mount=type=cache,dst=/foo echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::CacheMount(CacheOpts {
                    c_target: tp("/foo"),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_dst_foo_type_cache() {
        assert_ast(
            "RUN --mount=dst=/foo,type=cache echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::CacheMount(CacheOpts {
                    c_target: tp("/foo"),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_cache_with_all_modifiers() {
        assert_ast(
            "RUN --mount=type=cache,target=/foo,sharing=private,id=a,ro,from=ubuntu,source=/bar,mode=0700,uid=0,gid=0 echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::CacheMount(CacheOpts {
                    c_target: tp("/foo"),
                    c_sharing: Some(CacheSharing::Private),
                    c_cache_id: Some("a".to_string()),
                    c_read_only: Some(true),
                    c_from_image: Some("ubuntu".to_string()),
                    c_source: Some(sp("/bar")),
                    c_mode: Some("0700".to_string()),
                    c_uid: Some("0".to_string()),
                    c_gid: Some("0".to_string()),
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_cache_with_all_modifiers_different_order() {
        assert_ast(
            "RUN --mount=readonly,sharing=private,id=a,type=cache,from=ubuntu,source=/bar,destination=/foo,mode=0700,uid=0,gid=0 echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::CacheMount(CacheOpts {
                    c_target: tp("/foo"),
                    c_sharing: Some(CacheSharing::Private),
                    c_cache_id: Some("a".to_string()),
                    c_read_only: Some(true),
                    c_from_image: Some("ubuntu".to_string()),
                    c_source: Some(sp("/bar")),
                    c_mode: Some("0700".to_string()),
                    c_uid: Some("0".to_string()),
                    c_gid: Some("0".to_string()),
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_tmpfs() {
        assert_ast(
            "RUN --mount=type=tmpfs,target=/foo echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::TmpfsMount(TmpOpts {
                    t_target: tp("/foo"),
                    t_size: None,
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_tmpfs_size_12g() {
        assert_ast(
            "RUN --mount=type=tmpfs,target=/foo,size=12G foo bar\n",
            vec![run(
                "foo bar",
                mounts(vec![RunMount::TmpfsMount(TmpOpts {
                    t_target: tp("/foo"),
                    t_size: Some("12G".to_string()),
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_ssh() {
        assert_ast(
            "RUN --mount=type=ssh echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::SshMount(SecretOpts {
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_ssh_required_false() {
        assert_ast(
            "RUN --mount=type=ssh,required=false echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::SshMount(SecretOpts {
                    s_is_required: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_ssh_required_false_2() {
        assert_ast(
            "RUN --mount=type=ssh,required=False echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::SshMount(SecretOpts {
                    s_is_required: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_secret_required_true() {
        assert_ast(
            "RUN --mount=type=secret,required=true echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::SecretMount(SecretOpts {
                    s_is_required: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_secret_required_true_2() {
        assert_ast(
            "RUN --mount=type=secret,required=True echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::SecretMount(SecretOpts {
                    s_is_required: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_ssh_all_modifiers() {
        assert_ast(
            "RUN --mount=type=ssh,target=/foo,id=a,required,source=/bar,mode=0700,uid=0,gid=0 echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::SshMount(SecretOpts {
                    s_target: Some(tp("/foo")),
                    s_cache_id: Some("a".to_string()),
                    s_is_required: Some(true),
                    s_source: Some(sp("/bar")),
                    s_mode: Some("0700".to_string()),
                    s_uid: Some("0".to_string()),
                    s_gid: Some("0".to_string()),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_ssh_all_modifiers_required_explicit() {
        assert_ast(
            "RUN --mount=type=ssh,target=/foo,id=a,required=true,source=/bar,mode=0700,uid=0,gid=0 echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::SshMount(SecretOpts {
                    s_target: Some(tp("/foo")),
                    s_cache_id: Some("a".to_string()),
                    s_is_required: Some(true),
                    s_source: Some(sp("/bar")),
                    s_mode: Some("0700".to_string()),
                    s_uid: Some("0".to_string()),
                    s_gid: Some("0".to_string()),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_secret_all_modifiers() {
        assert_ast(
            "RUN --mount=type=secret,target=/foo,env=baz,id=a,required,source=/bar,mode=0700,uid=0,gid=0 echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::SecretMount(SecretOpts {
                    s_target: Some(tp("/foo")),
                    s_cache_id: Some("a".to_string()),
                    s_is_required: Some(true),
                    s_source: Some(sp("/bar")),
                    s_mode: Some("0700".to_string()),
                    s_uid: Some("0".to_string()),
                    s_gid: Some("0".to_string()),
                    s_env: Some("baz".to_string()),
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_secret_all_modifiers_required_explicit() {
        assert_ast(
            "RUN --mount=type=secret,target=/foo,env=baz,id=a,required=true,source=/bar,mode=0700,uid=0,gid=0 echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::SecretMount(SecretOpts {
                    s_target: Some(tp("/foo")),
                    s_cache_id: Some("a".to_string()),
                    s_is_required: Some(true),
                    s_source: Some(sp("/bar")),
                    s_mode: Some("0700".to_string()),
                    s_uid: Some("0".to_string()),
                    s_gid: Some("0".to_string()),
                    s_env: Some("baz".to_string()),
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_cache_uid_gid_var() {
        assert_ast(
            "RUN --mount=type=cache,target=/foo,uid=$VAR_UID,gid=$VAR_GID echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::CacheMount(CacheOpts {
                    c_target: tp("/foo"),
                    c_uid: Some("$VAR_UID".to_string()),
                    c_gid: Some("$VAR_GID".to_string()),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn multiple_mount_type_cache_flags() {
        assert_ast(
            "RUN --mount=type=cache,target=/foo \\\n    --mount=type=cache,target=/bar \\\n    echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![
                    RunMount::CacheMount(CacheOpts {
                        c_target: tp("/foo"),
                        ..Default::default()
                    }),
                    RunMount::CacheMount(CacheOpts {
                        c_target: tp("/bar"),
                        ..Default::default()
                    }),
                ]),
            )],
        );
    }

    #[test]
    fn multiple_different_mount_flags() {
        assert_ast(
            "RUN --mount=type=cache,target=/foo \\\n    --mount=type=secret,target=/bar \\\n    echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![
                    RunMount::CacheMount(CacheOpts {
                        c_target: tp("/foo"),
                        ..Default::default()
                    }),
                    RunMount::SecretMount(SecretOpts {
                        s_target: Some(tp("/bar")),
                        ..Default::default()
                    }),
                ]),
            )],
        );
    }

    #[test]
    fn network_none() {
        assert_ast(
            "RUN --network=none echo foo\n",
            vec![run(
                "echo foo",
                RunFlags {
                    network: Some(RunNetwork::NetworkNone),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn network_host() {
        assert_ast(
            "RUN --network=host echo foo\n",
            vec![run(
                "echo foo",
                RunFlags {
                    network: Some(RunNetwork::NetworkHost),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn network_default() {
        assert_ast(
            "RUN --network=default echo foo\n",
            vec![run(
                "echo foo",
                RunFlags {
                    network: Some(RunNetwork::NetworkDefault),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn security_insecure() {
        assert_ast(
            "RUN --security=insecure echo foo\n",
            vec![run(
                "echo foo",
                RunFlags {
                    security: Some(RunSecurity::Insecure),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn security_sandbox() {
        assert_ast(
            "RUN --security=sandbox echo foo\n",
            vec![run(
                "echo foo",
                RunFlags {
                    security: Some(RunSecurity::Sandbox),
                    ..Default::default()
                },
            )],
        );
    }

    #[test]
    fn allows_all_flags() {
        assert_ast(
            "RUN --mount=target=/foo --network=none --security=sandbox echo foo\n",
            vec![run(
                "echo foo",
                RunFlags {
                    security: Some(RunSecurity::Sandbox),
                    network: Some(RunNetwork::NetworkNone),
                    mount: mounts(vec![RunMount::BindMount(BindOpts {
                        b_target: tp("/foo"),
                        ..Default::default()
                    })])
                    .mount,
                },
            )],
        );
    }

    #[test]
    fn mount_type_bind_ro() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,ro echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_ro_2() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,RO echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readonly() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,readonly echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readonly_2() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,Readonly echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_ro_true() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,ro=true echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_ro_true_2() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,RO=TRUE echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readonly_true() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,readonly=true echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readonly_true_2() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,Readonly=True echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_ro_true_3() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,ro=True echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readonly_true_3() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,readonly=True echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_rw_false() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,rw=false echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readwrite_false() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,readwrite=false echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_rw_false_2() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,rw=False echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readwrite_false_2() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,readwrite=False echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(true),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_rw() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,rw echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readwrite() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,readwrite echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_rw_true() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,rw=true echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readwrite_true() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,readwrite=true echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_rw_true_2() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,rw=True echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readwrite_true_2() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,readwrite=True echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_ro_false() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,ro=false echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readonly_false() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,readonly=false echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_ro_false_2() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,ro=False echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn mount_type_bind_readonly_false_2() {
        assert_ast(
            "RUN --mount=type=bind,target=/foo,source=/bar,readonly=False echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/foo"),
                    b_source: Some(sp("/bar")),
                    b_read_only: Some(false),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn foo_heredoc() {
        assert_ast(
            "RUN <<EOF\nfoo\nEOF\n",
            vec![run("foo", RunFlags::default())],
        );
    }

    #[test]
    fn foo_heredoc_without_newline_at_end() {
        assert_ast("RUN <<EOF\nfoo\nEOF", vec![run("foo", RunFlags::default())]);
    }

    #[test]
    fn foo_heredoc_marker() {
        assert_ast(
            "RUN <<FOO\nfoo\nFOO\n",
            vec![run("foo", RunFlags::default())],
        );
    }

    #[test]
    fn foo_heredoc_quoted_marker() {
        assert_ast(
            "RUN <<\"FOO\"\nfoo\nFOO\n",
            vec![run("foo", RunFlags::default())],
        );
    }

    #[test]
    fn foo_heredoc_single_quoted_marker() {
        assert_ast(
            "RUN <<'FOO'\nfoo\nFOO\n",
            vec![run("foo", RunFlags::default())],
        );
    }

    #[test]
    fn foo_heredoc_dash() {
        assert_ast(
            "RUN <<-FOO\nfoo\nFOO\n",
            vec![run("foo", RunFlags::default())],
        );
    }

    #[test]
    fn foo_heredoc_quoted_dash() {
        assert_ast(
            "RUN <<-\"FOO\"\nfoo\nFOO\n",
            vec![run("foo", RunFlags::default())],
        );
    }

    #[test]
    fn foo_heredoc_single_quoted_dash() {
        assert_ast(
            "RUN <<-'FOO'\nfoo\nFOO\n",
            vec![run("foo", RunFlags::default())],
        );
    }

    #[test]
    fn multiline_foo_heredoc() {
        assert_ast(
            "RUN <<EOF\nfoo\nbar\n\ndodo\nEOF\n",
            vec![run("foo\nbar\n\ndodo", RunFlags::default())],
        );
    }

    #[test]
    fn heredoc_with_shebang_commend() {
        assert_ast(
            "RUN <<EOF\n#/usr/bin/env python\n\n#print foo\nprint(\"foo\")\nEOF\n",
            vec![run(
                "#/usr/bin/env python\n\n#print foo\nprint(\"foo\")",
                RunFlags::default(),
            )],
        );
    }

    #[test]
    fn empty_heredoc() {
        assert_ast("RUN <<EOF\nEOF\n", vec![run("", RunFlags::default())]);
    }

    #[test]
    fn empty_heredoc_not_followed_by_newline() {
        assert_ast("RUN <<EOF\nEOF", vec![run("", RunFlags::default())]);
    }

    #[test]
    fn evil_heredoc() {
        assert_ast(
            "RUN <<EOF foo\nbar EOF\nEOF\n",
            vec![run("foo\nbar EOF", RunFlags::default())],
        );
    }

    #[test]
    fn heredoc_with_redirection_to_file() {
        assert_ast(
            "RUN <<EOF > /file\nfoo\nEOF\n",
            vec![run("foo", RunFlags::default())],
        );
    }

    #[test]
    fn heredoc_to_program_stdin() {
        assert_ast(
            "RUN python <<EOF\nprint(\"foo\")\nEOF\n",
            vec![run("python", RunFlags::default())],
        );
    }

    #[test]
    fn heredoc_to_program_stdin_with_redirect_to_file_with() {
        assert_ast(
            "RUN python <<EOF > /file\nprint(\"foo\")\nEOF\n",
            vec![run("python", RunFlags::default())],
        );
    }

    #[test]
    fn heredoc_to_program_stdin_with_redirect_to_file_with_2() {
        assert_ast(
            "RUN python <<EOF >> /file\nprint(\"foo\")\nEOF\n",
            vec![run("python", RunFlags::default())],
        );
    }

    #[test]
    fn heredoc_pipe_to_program() {
        assert_ast(
            "RUN cat <<EOF | sh\necho foo\nEOF\n",
            vec![run("cat", RunFlags::default())],
        );
    }

    #[test]
    fn heredoc_with_line_continuation_in_the_heredoc() {
        assert_ast(
            "RUN <<EOF\napt-get update\napt-get install foo bar \\\n  buzz bar\nEOF\n",
            vec![run(
                "apt-get update\napt-get install foo bar \\\n  buzz bar",
                RunFlags::default(),
            )],
        );
    }

    #[test]
    fn heredoc_correct_termination() {
        assert_ast(
            "RUN <<EOF\necho $EOF\nEOF\n",
            vec![run("echo $EOF", RunFlags::default())],
        );
    }

    #[test]
    fn heredoc_correct_termination_without_newline() {
        assert_ast(
            "RUN <<EOF\necho $EOF\nEOF",
            vec![run("echo $EOF", RunFlags::default())],
        );
    }

    #[test]
    fn heredoc_no_overzealous_parsing() {
        assert_ast(
            "RUN foo bar\nFROM something\nCOPY <<EOF /foobar.sh\n#!/bin/bash\necho foobar\nEOF\n",
            vec![
                run("foo bar", RunFlags::default()),
                Instruction::From(BaseImage {
                    image: Image {
                        registry_name: None,
                        image_name: "something".to_string(),
                    },
                    tag: None,
                    digest: None,
                    alias: None,
                    platform: None,
                }),
                Instruction::Copy(
                    CopyArgs {
                        source_paths: NonEmpty::new(sp("EOF"), vec![]),
                        target_path: tp("/foobar.sh"),
                    },
                    CopyFlags::default(),
                ),
            ],
        );
    }

    #[test]
    fn heredoc_in_command_chain_with_escaped_newlines_after_redirect() {
        assert_ast(
            "RUN ls && cat <<EOF >> go.mod && \\\n    tac go.mod\nreplace (\n    github.com/user/repo => ../dir\n)\nEOF\n",
            vec![run("ls && cat", RunFlags::default())],
        );
    }

    #[test]
    fn heredoc_in_command_chain_with_escaped_newlines_before_heredoc() {
        assert_ast(
            "RUN ls && \\\n    cat <<EOF >> go.mod && tac go.mod\nreplace (\n    github.com/user/repo => ../dir\n)\nEOF\n",
            vec![run("ls &&  cat", RunFlags::default())],
        );
    }

    #[test]
    fn heredoc_in_command_chain_with_escaped_newlines_before_and_after_heredoc() {
        assert_ast(
            "RUN ls && \\\n    cat <<EOF >> go.mod && \\\n    tac go.mod\nreplace (\n    github.com/user/repo => ../dir\n)\nEOF\n",
            vec![run("ls &&  cat", RunFlags::default())],
        );
    }

    #[test]
    fn fail_because_of_multiple_types_in_mount() {
        assert!(parse_text("RUN --mount=type=cache,type=tmpfs,target=/foo foo bar").is_err());
    }

    #[test]
    fn run_with_relabel_shared() {
        assert_ast(
            "RUN --mount=type=bind,target=/bar,relabel=shared echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/bar"),
                    b_relabel: Some(Relabel::RelabelShared),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn run_with_relabel_private() {
        assert_ast(
            "RUN --mount=type=bind,target=/bar,relabel=private echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/bar"),
                    b_relabel: Some(Relabel::RelabelPrivate),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn run_with_short_selinux_relabel_z() {
        assert_ast(
            "RUN --mount=type=bind,target=/bar,z echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/bar"),
                    b_relabel: Some(Relabel::RelabelShared),
                    ..Default::default()
                })]),
            )],
        );
    }

    #[test]
    fn run_with_short_selinux_relabel_z_2() {
        assert_ast(
            "RUN --mount=type=bind,target=/bar,Z echo foo\n",
            vec![run(
                "echo foo",
                mounts(vec![RunMount::BindMount(BindOpts {
                    b_target: tp("/bar"),
                    b_relabel: Some(Relabel::RelabelPrivate),
                    ..Default::default()
                })]),
            )],
        );
    }
}
