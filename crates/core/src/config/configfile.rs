// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Config/Configfile.hs.

use crate::config::configuration::PartialConfiguration;

/// Haskell `getConfigFromFile`: explicit path, or the first existing of
/// `./.hadolint.{yaml,yml}`, `XDG_CONFIG/hadolint.{yaml,yml}`, `~/.hadolint/{hadolint,config}.{yaml,yml}`,
/// `~/.hadolint.{yaml,yml}`; prints the used file to stderr when verbose.
pub fn get_config_from_file(
    maybe_explicit_path: Option<&str>,
    verbose: bool,
) -> std::io::Result<Result<PartialConfiguration, String>> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Configfile.hs getConfigFromFile
    let _ = (maybe_explicit_path, verbose);
    Ok(Ok(PartialConfiguration::mempty()))
}

/// Haskell `readConfig`.
pub fn read_config(path: &str) -> std::io::Result<Result<PartialConfiguration, String>> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Configfile.hs readConfig
    let _ = path;
    Ok(Ok(PartialConfiguration::mempty()))
}

/// Haskell `getFilePathDescription`.
pub fn get_file_path_description(path: Option<&str>) -> String {
    match path {
        None => "No configuration was specified. Using default configuration".to_string(),
        Some(p) => format!("Configuration file used: {p}"),
    }
}

/// Haskell `formatError`.
pub fn format_error(err: &str, config: &str) -> String {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Configfile.hs formatError
    let _ = (err, config);
    String::new()
}

/// Haskell `getConfig`.
pub fn get_config(maybe_config: Option<&str>) -> std::io::Result<Option<String>> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Configfile.hs getConfig
    Ok(maybe_config.map(String::from))
}

/// Haskell `findConfig`.
pub fn find_config() -> std::io::Result<Option<String>> {
    // TODO(port): upstream/hadolint/src/Hadolint/Config/Configfile.hs findConfig
    Ok(None)
}
