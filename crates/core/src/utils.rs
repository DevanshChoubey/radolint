// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Utils.hs.

use radolint_syntax::{RunFlags, RunMount};

/// Haskell `hasCacheOrTmpfsMountWith`.
pub fn has_cache_or_tmpfs_mount_with(frag: &str, flags: &RunFlags) -> bool {
    flags
        .mount
        .iter()
        .any(|m| is_cache_or_tmpfs_mount_with(frag, m))
}

fn is_cache_or_tmpfs_mount_with(frag: &str, m: &RunMount) -> bool {
    match m {
        RunMount::CacheMount(o) => o.c_target.un_target_path.contains(frag),
        RunMount::TmpfsMount(o) => o.t_target.un_target_path.contains(frag),
        _ => false,
    }
}

/// Haskell `hasCacheOrTmpfsMount`.
pub fn has_cache_or_tmpfs_mount(flags: &RunFlags) -> bool {
    flags.mount.iter().any(is_cache_or_tmpfs_mount)
}

fn is_cache_or_tmpfs_mount(m: &RunMount) -> bool {
    matches!(m, RunMount::CacheMount(_) | RunMount::TmpfsMount(_))
}
