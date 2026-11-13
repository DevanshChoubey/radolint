// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Syntax.hs (+ Syntax/Port.hs,
//! Syntax/PortRange.hs, Syntax/Protocol.hs).

use std::collections::BTreeSet;

/// Haskell `Data.List.NonEmpty.NonEmpty a`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NonEmpty<T> {
    pub head: T,
    pub tail: Vec<T>,
}

impl<T> NonEmpty<T> {
    pub fn new(head: T, tail: Vec<T>) -> Self {
        NonEmpty { head, tail }
    }
    /// `NonEmpty.nonEmpty`
    pub fn from_vec(mut v: Vec<T>) -> Option<Self> {
        if v.is_empty() {
            None
        } else {
            let head = v.remove(0);
            Some(NonEmpty { head, tail: v })
        }
    }
    /// `NonEmpty.toList`
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut v = vec![self.head.clone()];
        v.extend(self.tail.iter().cloned());
        v
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.head).chain(self.tail.iter())
    }
    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
    /// `NonEmpty.last`
    pub fn last(&self) -> &T {
        self.tail.last().unwrap_or(&self.head)
    }
    /// `NonEmpty.init`
    pub fn init(&self) -> Vec<&T> {
        let all: Vec<&T> = self.iter().collect();
        all[..all.len() - 1].to_vec()
    }
}

/// Haskell `Data.Time.Clock.DiffTime`: a number of picoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct DiffTime(pub i128);

impl DiffTime {
    /// `secondsToDiffTime`
    pub fn from_seconds(s: i128) -> Self {
        DiffTime(s * 1_000_000_000_000)
    }
}

// ---------------------------------------------------------------- protocol / ports

/// Haskell `Language.Docker.Syntax.Protocol.Protocol`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Protocol {
    TCP,
    UDP,
}

/// Haskell `Language.Docker.Syntax.Port.Port`: a number (plus protocol) or a variable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Port {
    Port(i64, Protocol),
    PortStr(String),
}

/// Haskell `Language.Docker.Syntax.PortRange.PortRange`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PortRange(pub Port, pub Port);

/// Haskell `Language.Docker.Syntax.PortSpec`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PortSpec {
    PortSpec(Port),
    PortRangeSpec(PortRange),
}

/// Haskell `Language.Docker.Syntax.Ports` (newtype, field `unPorts`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ports {
    pub un_ports: Vec<PortSpec>,
}

// ---------------------------------------------------------------- images

/// Haskell `Language.Docker.Syntax.Registry` (newtype, field `unRegistry`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Registry {
    pub un_registry: String,
}

/// Haskell `Language.Docker.Syntax.Image`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Image {
    pub registry_name: Option<Registry>,
    pub image_name: String,
}

/// Haskell `Language.Docker.Syntax.Tag` (field `unTag`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tag {
    pub un_tag: String,
}

/// Haskell `Language.Docker.Syntax.Digest` (field `unDigest`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Digest {
    pub un_digest: String,
}

/// Haskell `Language.Docker.Syntax.Directory`.
pub type Directory = String;
/// Haskell `Language.Docker.Syntax.Platform`.
pub type Platform = String;

/// Haskell `Language.Docker.Syntax.ImageAlias` (field `unImageAlias`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImageAlias {
    pub un_image_alias: String,
}

/// Haskell `Language.Docker.Syntax.BaseImage`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BaseImage {
    pub image: Image,
    pub tag: Option<Tag>,
    pub digest: Option<Digest>,
    pub alias: Option<ImageAlias>,
    pub platform: Option<Platform>,
}

/// Haskell `Language.Docker.Syntax.SourcePath` (field `unSourcePath`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourcePath {
    pub un_source_path: String,
}

/// Haskell `Language.Docker.Syntax.TargetPath` (field `unTargetPath`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TargetPath {
    pub un_target_path: String,
}

/// Haskell `Language.Docker.Syntax.Relabel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Relabel {
    RelabelShared,
    RelabelPrivate,
}

/// Haskell `Language.Docker.Syntax.Checksum`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Checksum {
    Checksum(String),
    NoChecksum,
}

/// Haskell `Language.Docker.Syntax.Chown`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Chown {
    Chown(String),
    NoChown,
}

/// Haskell `Language.Docker.Syntax.Chmod`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Chmod {
    Chmod(String),
    NoChmod,
}

/// Haskell `Language.Docker.Syntax.Link`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Link {
    Link(bool),
    NoLink,
}

/// Haskell `Language.Docker.Syntax.KeepGitDir`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeepGitDir {
    KeepGitDir(bool),
    NoKeepGitDir,
}

/// Haskell `Language.Docker.Syntax.Parents`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Parents {
    Parents(bool),
    NoParents,
}

/// Haskell `Language.Docker.Syntax.Unpack`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Unpack {
    Unpack(bool),
    NoUnpack,
}

/// Haskell `Language.Docker.Syntax.CopySource`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CopySource {
    CopySource(String),
    NoSource,
}

/// Haskell `Language.Docker.Syntax.Duration` (field `durationTime`; Num/Fractional in Haskell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Duration {
    pub duration_time: DiffTime,
}

/// Haskell `Language.Docker.Syntax.Retries` (field `times`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Retries {
    pub times: i64,
}

/// Haskell `Language.Docker.Syntax.Exclude` (field `unExclude`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Exclude {
    pub un_exclude: String,
}

/// Haskell `Language.Docker.Syntax.CopyArgs`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CopyArgs {
    pub source_paths: NonEmpty<SourcePath>,
    pub target_path: TargetPath,
}

/// Haskell `Language.Docker.Syntax.CopyFlags`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CopyFlags {
    pub chown_flag: Chown,
    pub chmod_flag: Chmod,
    pub link_flag: Link,
    pub parents_flag: Parents,
    pub source_flag: CopySource,
    pub exclude_flags: Vec<Exclude>,
}

/// Haskell `Language.Docker.Syntax.AddArgs`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AddArgs {
    pub source_paths: NonEmpty<SourcePath>,
    pub target_path: TargetPath,
}

/// Haskell `Language.Docker.Syntax.AddFlags`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AddFlags {
    pub checksum_flag: Checksum,
    pub chown_flag: Chown,
    pub chmod_flag: Chmod,
    pub link_flag: Link,
    pub keep_git_dir_flag: KeepGitDir,
    pub unpack_flag: Unpack,
    pub exclude_flags: Vec<Exclude>,
}

impl Default for CopyFlags {
    /// Haskell `def :: CopyFlags`
    fn default() -> Self {
        CopyFlags {
            chown_flag: Chown::NoChown,
            chmod_flag: Chmod::NoChmod,
            link_flag: Link::NoLink,
            parents_flag: Parents::NoParents,
            source_flag: CopySource::NoSource,
            exclude_flags: vec![],
        }
    }
}

impl Default for AddFlags {
    /// Haskell `def :: AddFlags`
    fn default() -> Self {
        AddFlags {
            checksum_flag: Checksum::NoChecksum,
            chown_flag: Chown::NoChown,
            chmod_flag: Chmod::NoChmod,
            link_flag: Link::NoLink,
            keep_git_dir_flag: KeepGitDir::NoKeepGitDir,
            unpack_flag: Unpack::NoUnpack,
            exclude_flags: vec![],
        }
    }
}

// ---------------------------------------------------------------- arguments / healthcheck

/// Haskell `Language.Docker.Syntax.Arguments` (derives Functor).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Arguments<Args> {
    ArgumentsText(Args),
    ArgumentsList(Args),
}

impl<A> Arguments<A> {
    /// `fmap`
    pub fn fmap<B>(self, f: &dyn Fn(A) -> B) -> Arguments<B> {
        match self {
            Arguments::ArgumentsText(a) => Arguments::ArgumentsText(f(a)),
            Arguments::ArgumentsList(a) => Arguments::ArgumentsList(f(a)),
        }
    }
}

/// Haskell `Language.Docker.Syntax.CheckArgs` (derives Functor).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CheckArgs<Args> {
    pub check_command: Arguments<Args>,
    pub interval: Option<Duration>,
    pub timeout: Option<Duration>,
    pub start_period: Option<Duration>,
    pub start_interval: Option<Duration>,
    pub retries: Option<Retries>,
}

impl<A> CheckArgs<A> {
    pub fn fmap<B>(self, f: &dyn Fn(A) -> B) -> CheckArgs<B> {
        CheckArgs {
            check_command: self.check_command.fmap(f),
            interval: self.interval,
            timeout: self.timeout,
            start_period: self.start_period,
            start_interval: self.start_interval,
            retries: self.retries,
        }
    }
}

/// Haskell `Language.Docker.Syntax.Check` (derives Functor).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Check<Args> {
    Check(CheckArgs<Args>),
    NoCheck,
}

impl<A> Check<A> {
    pub fn fmap<B>(self, f: &dyn Fn(A) -> B) -> Check<B> {
        match self {
            Check::Check(c) => Check::Check(c.fmap(f)),
            Check::NoCheck => Check::NoCheck,
        }
    }
}

/// Haskell `Language.Docker.Syntax.Pairs`.
pub type Pairs = Vec<(String, String)>;

// ---------------------------------------------------------------- RUN flags

/// Haskell `Language.Docker.Syntax.RunMount`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RunMount {
    BindMount(BindOpts),
    CacheMount(CacheOpts),
    TmpfsMount(TmpOpts),
    SecretMount(SecretOpts),
    SshMount(SecretOpts),
}

/// Haskell `Language.Docker.Syntax.BindOpts`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BindOpts {
    pub b_target: TargetPath,
    pub b_source: Option<SourcePath>,
    pub b_from_image: Option<String>,
    pub b_read_only: Option<bool>,
    pub b_relabel: Option<Relabel>,
}

impl Default for BindOpts {
    fn default() -> Self {
        BindOpts {
            b_target: TargetPath::from(""),
            b_source: None,
            b_from_image: None,
            b_read_only: None,
            b_relabel: None,
        }
    }
}

/// Haskell `Language.Docker.Syntax.CacheOpts`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CacheOpts {
    pub c_target: TargetPath,
    pub c_sharing: Option<CacheSharing>,
    pub c_cache_id: Option<String>,
    pub c_read_only: Option<bool>,
    pub c_from_image: Option<String>,
    pub c_source: Option<SourcePath>,
    pub c_mode: Option<String>,
    pub c_uid: Option<String>,
    pub c_gid: Option<String>,
}

impl Default for CacheOpts {
    fn default() -> Self {
        CacheOpts {
            c_target: TargetPath::from(""),
            c_sharing: None,
            c_cache_id: None,
            c_read_only: None,
            c_from_image: None,
            c_source: None,
            c_mode: None,
            c_uid: None,
            c_gid: None,
        }
    }
}

/// Haskell `Language.Docker.Syntax.TmpOpts`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TmpOpts {
    pub t_target: TargetPath,
    pub t_size: Option<String>,
}

impl Default for TmpOpts {
    fn default() -> Self {
        TmpOpts {
            t_target: TargetPath::from(""),
            t_size: None,
        }
    }
}

/// Haskell `Language.Docker.Syntax.SecretOpts`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct SecretOpts {
    pub s_target: Option<TargetPath>,
    pub s_cache_id: Option<String>,
    pub s_is_required: Option<bool>,
    pub s_source: Option<SourcePath>,
    pub s_env: Option<String>,
    pub s_mode: Option<String>,
    pub s_uid: Option<String>,
    pub s_gid: Option<String>,
}

/// Haskell `Language.Docker.Syntax.CacheSharing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CacheSharing {
    Shared,
    Private,
    Locked,
}

/// Haskell `Language.Docker.Syntax.RunSecurity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RunSecurity {
    Insecure,
    Sandbox,
}

/// Haskell `Language.Docker.Syntax.RunNetwork`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RunNetwork {
    NetworkNone,
    NetworkHost,
    NetworkDefault,
}

/// Haskell `Language.Docker.Syntax.RunFlags` (`mount` is a `Set RunMount`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct RunFlags {
    pub mount: BTreeSet<RunMount>,
    pub security: Option<RunSecurity>,
    pub network: Option<RunNetwork>,
}

/// Haskell `Language.Docker.Syntax.RunArgs` (derives Functor).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RunArgs<Args>(pub Arguments<Args>, pub RunFlags);

impl<A> RunArgs<A> {
    pub fn fmap<B>(self, f: &dyn Fn(A) -> B) -> RunArgs<B> {
        RunArgs(self.0.fmap(f), self.1)
    }
}

// ---------------------------------------------------------------- pragmas / instructions

/// Haskell `Language.Docker.Syntax.EscapeChar` (field `escape`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EscapeChar {
    pub escape: char,
}

/// Haskell `Language.Docker.Syntax.SyntaxImage` (field `syntax`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SyntaxImage {
    pub syntax: Image,
}

/// Haskell `Language.Docker.Syntax.PragmaDirective`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PragmaDirective {
    Escape(EscapeChar),
    Syntax(SyntaxImage),
}

/// Haskell `Language.Docker.Syntax.Instruction` (all commands available in Dockerfiles; derives Functor).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Instruction<Args> {
    From(BaseImage),
    Add(AddArgs, AddFlags),
    User(String),
    Label(Pairs),
    Stopsignal(String),
    Copy(CopyArgs, CopyFlags),
    Run(RunArgs<Args>),
    Cmd(Arguments<Args>),
    Shell(Arguments<Args>),
    Workdir(Directory),
    Expose(Ports),
    Volume(String),
    Entrypoint(Arguments<Args>),
    Maintainer(String),
    Env(Pairs),
    Arg(String, Option<String>),
    Healthcheck(Check<Args>),
    Pragma(PragmaDirective),
    Comment(String),
    OnBuild(Box<Instruction<Args>>),
}

impl<A> Instruction<A> {
    /// `fmap`
    pub fn fmap<B>(self, f: &dyn Fn(A) -> B) -> Instruction<B> {
        use Instruction::*;
        match self {
            From(b) => From(b),
            Add(a, fl) => Add(a, fl),
            User(t) => User(t),
            Label(p) => Label(p),
            Stopsignal(t) => Stopsignal(t),
            Copy(a, fl) => Copy(a, fl),
            Run(r) => Run(r.fmap(f)),
            Cmd(a) => Cmd(a.fmap(f)),
            Shell(a) => Shell(a.fmap(f)),
            Workdir(d) => Workdir(d),
            Expose(p) => Expose(p),
            Volume(t) => Volume(t),
            Entrypoint(a) => Entrypoint(a.fmap(f)),
            Maintainer(t) => Maintainer(t),
            Env(p) => Env(p),
            Arg(a, b) => Arg(a, b),
            Healthcheck(c) => Healthcheck(c.fmap(f)),
            Pragma(p) => Pragma(p),
            Comment(t) => Comment(t),
            OnBuild(i) => OnBuild(Box::new((*i).fmap(f))),
        }
    }
}

/// Haskell `Language.Docker.Syntax.Filename`.
pub type Filename = String;
/// Haskell `Language.Docker.Syntax.Linenumber`.
pub type Linenumber = i64;

/// Haskell `Language.Docker.Syntax.InstructionPos`: an `Instruction` with location
/// information (derives Functor).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstructionPos<Args> {
    pub instruction: Instruction<Args>,
    pub sourcename: Filename,
    pub line_number: Linenumber,
}

impl<A> InstructionPos<A> {
    /// `fmap`
    pub fn fmap<B>(self, f: &dyn Fn(A) -> B) -> InstructionPos<B> {
        InstructionPos {
            instruction: self.instruction.fmap(f),
            sourcename: self.sourcename,
            line_number: self.line_number,
        }
    }
}

/// Haskell `Language.Docker.Syntax.Dockerfile`: type of the Dockerfile AST.
pub type Dockerfile = Vec<InstructionPos<String>>;

/// Haskell `Language.Docker.Syntax.defaultEsc`.
pub const DEFAULT_ESC: char = '\\';

// ---------------------------------------------------------------- IsString instances

macro_rules! newtype_from_str {
    ($t:ident, $f:ident) => {
        impl From<&str> for $t {
            fn from(s: &str) -> Self {
                $t { $f: s.to_string() }
            }
        }
        impl From<String> for $t {
            fn from(s: String) -> Self {
                $t { $f: s }
            }
        }
    };
}
newtype_from_str!(Registry, un_registry);
newtype_from_str!(Tag, un_tag);
newtype_from_str!(Digest, un_digest);
newtype_from_str!(ImageAlias, un_image_alias);
newtype_from_str!(SourcePath, un_source_path);
newtype_from_str!(TargetPath, un_target_path);
newtype_from_str!(Exclude, un_exclude);

impl From<&str> for Image {
    /// Haskell `IsString Image` (`fromString`).
    fn from(img: &str) -> Self {
        if img.contains('/') {
            // Haskell `endBy "/"`: split on '/', dropping a trailing empty piece.
            let mut parts: Vec<&str> = img.split('/').collect();
            if parts.last() == Some(&"") {
                parts.pop();
            }
            if parts.first().is_some_and(|p| p.contains('.')) {
                Image {
                    registry_name: Some(Registry::from(parts[0])),
                    image_name: parts[1..].join("/"),
                }
            } else {
                Image {
                    registry_name: None,
                    image_name: img.to_string(),
                }
            }
        } else {
            Image {
                registry_name: None,
                image_name: img.to_string(),
            }
        }
    }
}

impl From<&str> for Checksum {
    fn from(s: &str) -> Self {
        if s.is_empty() {
            Checksum::NoChecksum
        } else {
            Checksum::Checksum(s.to_string())
        }
    }
}
impl From<&str> for Chown {
    fn from(s: &str) -> Self {
        if s.is_empty() {
            Chown::NoChown
        } else {
            Chown::Chown(s.to_string())
        }
    }
}
impl From<&str> for Chmod {
    fn from(s: &str) -> Self {
        if s.is_empty() {
            Chmod::NoChmod
        } else {
            Chmod::Chmod(s.to_string())
        }
    }
}
impl From<&str> for CopySource {
    fn from(s: &str) -> Self {
        if s.is_empty() {
            CopySource::NoSource
        } else {
            CopySource::CopySource(s.to_string())
        }
    }
}

impl From<&str> for Arguments<String> {
    /// Haskell `IsString (Arguments Text)`.
    fn from(s: &str) -> Self {
        Arguments::ArgumentsText(s.to_string())
    }
}

impl From<&str> for RunArgs<String> {
    /// Haskell `IsString (RunArgs Text)`.
    fn from(s: &str) -> Self {
        RunArgs(Arguments::ArgumentsText(s.to_string()), RunFlags::default())
    }
}

impl Arguments<String> {
    /// Haskell `IsList (Arguments Text)` `fromList` (`ArgumentsList . Text.unwords`).
    pub fn from_list(l: &[&str]) -> Self {
        Arguments::ArgumentsList(l.join(" "))
    }
    /// Haskell `IsList (Arguments Text)` `toList` (`Text.words`).
    pub fn to_list(&self) -> Vec<String> {
        match self {
            Arguments::ArgumentsText(s) | Arguments::ArgumentsList(s) => {
                s.split_whitespace().map(String::from).collect()
            }
        }
    }
}

impl Duration {
    /// Haskell `Num Duration` `fromInteger` (whole seconds).
    pub fn from_seconds(s: i128) -> Self {
        Duration {
            duration_time: DiffTime::from_seconds(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_from_string() {
        assert_eq!(
            Image::from("foo.com/bar/baz"),
            Image {
                registry_name: Some(Registry::from("foo.com")),
                image_name: "bar/baz".into()
            }
        );
        assert_eq!(
            Image::from("a/b"),
            Image {
                registry_name: None,
                image_name: "a/b".into()
            }
        );
    }
}
