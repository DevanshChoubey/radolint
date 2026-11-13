// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/Gnu.hs.

use super::format::FormatResult;
use std::io::Write;

/// Haskell `hWrite`.
pub fn h_write(handle: &mut dyn Write, results: &[FormatResult]) -> std::io::Result<()> {
    // TODO(port): upstream/hadolint/src/Hadolint/Formatter/Gnu.hs
    let _ = (handle, results);
    Ok(())
}
