// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Formatter/SonarQube.hs.

use super::format::FormatResult;
use std::io::Write;

/// Haskell `hWrite`.
pub fn h_write(handle: &mut dyn Write, results: &[FormatResult], file_path_in_report: Option<&str>) -> std::io::Result<()> {
    // TODO(port): upstream/hadolint/src/Hadolint/Formatter/SonarQube.hs
    let _ = (handle, results);
    let _ = file_path_in_report;
    Ok(())
}
