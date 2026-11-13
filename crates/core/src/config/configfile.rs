// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Config/Configfile.hs.

use crate::config::configuration::{PartialConfiguration, partial_from_yaml};
use std::path::PathBuf;

/// Haskell `getConfigFromFile`: explicit path, or the first existing of
/// `./.hadolint.{yaml,yml}`, `XDG_CONFIG/hadolint.{yaml,yml}`, `~/.hadolint/{hadolint,config}.{yaml,yml}`,
/// `~/.hadolint.{yaml,yml}`; prints the used file to stderr when verbose.
pub fn get_config_from_file(
    maybe_explicit_path: Option<&str>,
    verbose: bool,
) -> std::io::Result<Result<PartialConfiguration, String>> {
    let maybe_path = get_config(maybe_explicit_path)?;
    if verbose {
        // hPrint stderr on a String: shows the string (quoted, escaped).
        eprintln!("{:?}", get_file_path_description(maybe_path.as_deref()));
    }
    match maybe_path {
        None => Ok(Ok(PartialConfiguration::mempty())),
        Some(path) => read_config(&path),
    }
}

/// Haskell `readConfig`.
pub fn read_config(path: &str) -> std::io::Result<Result<PartialConfiguration, String>> {
    let bytes = std::fs::read(path)?;
    Ok(match decode1_strict(&bytes) {
        Err(err) => Err(format_error(&err, path)),
        Ok(config) => Ok(config),
    })
}

/// `Yaml.decode1Strict` for PartialConfiguration.
fn decode1_strict(bytes: &[u8]) -> Result<PartialConfiguration, String> {
    let text = match std::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(e) => return Err(e.to_string()),
    };
    // DIVERGENCE: HsYAML reports "empty YAML stream" for a document without content;
    // serde_yaml_ng yields null, so detect it here.
    if text.lines().all(|l| {
        let t = l.trim();
        t.is_empty() || t.starts_with('#')
    }) {
        return Err("empty YAML stream".to_string());
    }
    partial_from_yaml(text)
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
    let lines = [
        format!("Error parsing your config file in  '{config}':"),
        "It should contain one of the keys 'override', 'ignored'".to_string(),
        "or 'trustedRegistries'. For example:\n".to_string(),
        "ignored:".to_string(),
        "\t- DL3000".to_string(),
        "\t- SC1099\n\n".to_string(),
        "The key 'override' should contain only lists with names 'error',".to_string(),
        "'warning', 'info' or 'style', which each name rules to override the".to_string(),
        "severity on. For example:\n".to_string(),
        "override:".to_string(),
        "\terror:".to_string(),
        "\t\t- DL3008\n\n".to_string(),
        "The key 'trustedRegistries' should contain the names of the allowed".to_string(),
        "docker registries:\n".to_string(),
        "trustedRegistries:".to_string(),
        "\t- docker.io".to_string(),
        "\t- my-company.com".to_string(),
        String::new(),
        err.to_string(),
    ];
    let mut out = String::new();
    for l in lines {
        out.push_str(&l);
        out.push('\n');
    }
    out
}

/// Haskell `getConfig`.
pub fn get_config(maybe_config: Option<&str>) -> std::io::Result<Option<String>> {
    match maybe_config {
        None => find_config(),
        _ => Ok(maybe_config.map(String::from)),
    }
}

fn home_dir() -> PathBuf {
    // getUserDocumentsDirectory / getHomeDirectory on POSIX: $HOME.
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

fn xdg_config_dir() -> PathBuf {
    match std::env::var("XDG_CONFIG_HOME") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => home_dir().join(".config"),
    }
}

/// Haskell `findConfig`.
pub fn find_config() -> std::io::Result<Option<String>> {
    let hidden_configs = [".hadolint.yaml", ".hadolint.yml"];
    let visible_configs = ["hadolint.yaml", "hadolint.yml"];
    let more_configs = ["config.yaml", "config.yml"];
    let cwd = std::env::current_dir()?;
    let files_in_cwd: Vec<PathBuf> = hidden_configs.iter().map(|f| cwd.join(f)).collect();
    let files_in_xdg_config: Vec<PathBuf> = visible_configs
        .iter()
        .map(|f| xdg_config_dir().join(f))
        .collect();
    let app_data = home_dir().join(".hadolint");
    let files_in_app_data: Vec<PathBuf> = visible_configs
        .iter()
        .chain(more_configs.iter())
        .map(|f| app_data.join(f))
        .collect();
    let files_in_home: Vec<PathBuf> = hidden_configs.iter().map(|f| home_dir().join(f)).collect();
    Ok(files_in_cwd
        .into_iter()
        .chain(files_in_xdg_config)
        .chain(files_in_app_data)
        .chain(files_in_home)
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::format::OutputFormat;
    use crate::rule::{DLSeverity, LabelType};

    fn parse_yaml(ls: &[&str]) -> Result<PartialConfiguration, String> {
        let mut s = String::new();
        for l in ls {
            s.push_str(l);
            s.push('\n');
        }
        decode1_strict(s.as_bytes())
    }

    fn mempty() -> PartialConfiguration {
        PartialConfiguration::mempty()
    }

    fn codes(v: &[&str]) -> Vec<crate::rule::RuleCode> {
        v.iter().map(|s| crate::rule::RuleCode::from(*s)).collect()
    }

    #[test]
    fn parse_empty_config_file() {
        assert_eq!(parse_yaml(&[]), Err("empty YAML stream".to_string()));
    }

    #[test]
    fn parse_no_fail_true() {
        let mut m = mempty();
        m.partial_no_fail = Some(true);
        assert_eq!(parse_yaml(&["no-fail: true"]), Ok(m));
    }

    #[test]
    fn parse_no_fail_false() {
        let mut m = mempty();
        m.partial_no_fail = Some(false);
        assert_eq!(parse_yaml(&["no-fail: false"]), Ok(m));
    }

    #[test]
    fn parse_no_color_true() {
        let mut m = mempty();
        m.partial_no_color = Some(true);
        assert_eq!(parse_yaml(&["no-color: true"]), Ok(m));
    }

    #[test]
    fn parse_no_color_false() {
        let mut m = mempty();
        m.partial_no_color = Some(false);
        assert_eq!(parse_yaml(&["no-color: false"]), Ok(m));
    }

    #[test]
    fn parse_verbose_true() {
        let mut m = mempty();
        m.partial_verbose = Some(true);
        assert_eq!(parse_yaml(&["verbose: true"]), Ok(m));
    }

    #[test]
    fn parse_verbose_false() {
        let mut m = mempty();
        m.partial_verbose = Some(false);
        assert_eq!(parse_yaml(&["verbose: false"]), Ok(m));
    }

    #[test]
    fn parse_format_json() {
        let mut m = mempty();
        m.partial_formats = vec![OutputFormat::Json];
        assert_eq!(parse_yaml(&["format: json"]), Ok(m));
    }

    #[test]
    fn parse_format_sarif() {
        let mut m = mempty();
        m.partial_formats = vec![OutputFormat::Sarif];
        assert_eq!(parse_yaml(&["format: sarif"]), Ok(m));
    }

    #[test]
    fn parse_multiple_formats() {
        let mut m = mempty();
        m.partial_formats = vec![OutputFormat::Sarif, OutputFormat::Json];
        assert_eq!(parse_yaml(&["formats:", "  - sarif", "  - json"]), Ok(m));
    }

    fn override_yaml(k: &str) -> Vec<String> {
        vec![
            "override:".to_string(),
            format!("  {k}:"),
            "    - DL3020".to_string(),
            "    - DL3040".to_string(),
            "    - SC1020".to_string(),
        ]
    }

    fn parse_override(k: &str) -> Result<PartialConfiguration, String> {
        let y = override_yaml(k);
        let r: Vec<&str> = y.iter().map(|s| s.as_str()).collect();
        parse_yaml(&r)
    }

    #[test]
    fn parse_override_error_rules() {
        let mut m = mempty();
        m.partial_error_rules = codes(&["DL3020", "DL3040", "SC1020"]);
        assert_eq!(parse_override("error"), Ok(m));
    }

    #[test]
    fn parse_override_warning_rules() {
        let mut m = mempty();
        m.partial_warning_rules = codes(&["DL3020", "DL3040", "SC1020"]);
        assert_eq!(parse_override("warning"), Ok(m));
    }

    #[test]
    fn parse_override_info_rules() {
        let mut m = mempty();
        m.partial_info_rules = codes(&["DL3020", "DL3040", "SC1020"]);
        assert_eq!(parse_override("info"), Ok(m));
    }

    #[test]
    fn parse_override_style_rules() {
        let mut m = mempty();
        m.partial_style_rules = codes(&["DL3020", "DL3040", "SC1020"]);
        assert_eq!(parse_override("style"), Ok(m));
    }

    #[test]
    fn parse_ignored_rules() {
        let mut m = mempty();
        m.partial_ignore_rules = codes(&["DL3020", "DL3040", "SC1020"]);
        assert_eq!(
            parse_yaml(&["ignored:", "  - DL3020", "  - DL3040", "  - SC1020"]),
            Ok(m)
        );
    }

    #[test]
    fn parse_trusted_registries() {
        let mut m = mempty();
        m.partial_allowed_registries = ["foobar.com", "barfoo.com"]
            .iter()
            .map(|s| radolint_syntax::Registry::from(*s))
            .collect();
        assert_eq!(
            parse_yaml(&["trustedRegistries:", "  - foobar.com", "  - barfoo.com"]),
            Ok(m)
        );
    }

    #[test]
    fn parse_label_schema() {
        let mut m = mempty();
        m.partial_label_schema
            .insert("foo".to_string(), LabelType::Email);
        m.partial_label_schema
            .insert("bar".to_string(), LabelType::RawText);
        assert_eq!(
            parse_yaml(&["label-schema:", "  foo: email", "  bar: text"]),
            Ok(m)
        );
    }

    #[test]
    fn parse_strict_labels_true() {
        let mut m = mempty();
        m.partial_strict_labels = Some(true);
        assert_eq!(parse_yaml(&["strict-labels: true"]), Ok(m));
    }

    #[test]
    fn parse_strict_labels_false() {
        let mut m = mempty();
        m.partial_strict_labels = Some(false);
        assert_eq!(parse_yaml(&["strict-labels: false"]), Ok(m));
    }

    #[test]
    fn parse_disable_ignore_pragma_true() {
        let mut m = mempty();
        m.partial_disable_ignore_pragma = Some(true);
        assert_eq!(parse_yaml(&["disable-ignore-pragma: true"]), Ok(m));
    }

    #[test]
    fn parse_disable_ignore_pragma_false() {
        let mut m = mempty();
        m.partial_disable_ignore_pragma = Some(false);
        assert_eq!(parse_yaml(&["disable-ignore-pragma: false"]), Ok(m));
    }

    #[test]
    fn parse_failure_threshold_warning() {
        let mut m = mempty();
        m.partial_failure_threshold = Some(DLSeverity::DLWarningC);
        assert_eq!(parse_yaml(&["failure-threshold: warning"]), Ok(m));
    }

    #[test]
    fn parse_failure_threshold_style() {
        let mut m = mempty();
        m.partial_failure_threshold = Some(DLSeverity::DLStyleC);
        assert_eq!(parse_yaml(&["failure-threshold: style"]), Ok(m));
    }
}
