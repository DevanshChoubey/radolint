// SPDX-License-Identifier: GPL-3.0-only
//! Port of `Language.Docker.Syntax` (language-docker) and its `Port`, `PortRange`
//! and `Protocol` sub-modules: the Dockerfile AST.
//!
//! Naming: Haskell `Text` becomes `String`; constructors keep their names;
//! record fields are converted to snake_case. Haskell `DuplicateRecordFields`
//! (`sourcePaths`/`targetPath` in `CopyArgs` and `AddArgs`, `chownFlag` ...) are
//! simply separate fields of separate structs.
//!
//! `PrettyPrint.hs` is not ported: nothing under `upstream/hadolint/src` pretty
//! prints a Dockerfile, so the `Pretty` instances are omitted too.

mod syntax;
pub use syntax::*;
