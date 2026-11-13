// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Config/Commandline.hs (optparse-applicative parser),
//! hand-written so error texts can be matched to optparse-applicative's.

use crate::config::configuration::PartialConfiguration;
use crate::formatter::format::read_maybe_output_format;
use crate::rule::{read_either_label_type, read_either_severity, RuleCode};
use dlint_syntax::Registry;

/// Haskell `CommandlineConfig`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandlineConfig {
    pub show_version: bool,
    pub config_file: Option<String>,
    pub dockerfiles: Vec<String>,
    pub file_path_in_report_option: Option<String>,
    pub output: Vec<String>,
    pub configuration: PartialConfiguration,
}

/// Result of parsing argv: a config, `--help` (usage text, exit 0) or an error (message, exit 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandlineOutcome {
    Config(Box<CommandlineConfig>),
    Help(String),
    Error(String),
}

/// optparse-applicative usage/help text.
pub fn usage_text() -> String {
    // DIVERGENCE: hand-rendered approximation of optparse-applicative's `execParser` help output
    // (not recorded in the oracle corpus); header "hadolint - Dockerfile Linter written in Haskell".
    const HELP: &str = "\
hadolint - Dockerfile Linter written in Haskell

Usage: hadolint [-v|--version] [-c|--config FILENAME] [DOCKERFILE...]
                [--file-path-in-report FILEPATHINREPORT] [-o|--output OUTPUT]
                [--no-fail] [--no-color] [-V|--verbose] [-f|--format ARG]
                [--error RULECODE] [--warning RULECODE] [--info RULECODE]
                [--style RULECODE] [--ignore RULECODE]
                [--trusted-registry REGISTRY (e.g. docker.io)]
                [--require-label LABELSCHEMA (e.g. maintainer:text)]
                [--strict-labels] [--disable-ignore-pragma]
                [-t|--failure-threshold THRESHOLD]

  Lint Dockerfile for errors and best practices

Available options:
  -h,--help                Show this help text
  -v,--version             Show version
  -c,--config FILENAME     Path to the configuration file
  --file-path-in-report FILEPATHINREPORT
                           The file path referenced in the generated report.
                           This only applies for the 'checkstyle',
                           'codeclimate', 'sonarqube', 'junit' and
                           'gitlab_codeclimate' formats  and is useful when
                           running Hadolint with Docker to set  the correct file
                           path.
  -o,--output OUTPUT       Output destination file
  --no-fail                Don't exit with a failure status code when any rule
                           is violated
  --no-color               Don't colorize output
  -V,--verbose             Enables verbose logging of hadolint's output to
                           stderr
  -f,--format ARG          The output format for the results [tty | json |
                           checkstyle | codeclimate | gitlab_codeclimate | gnu |
                           codacy | sonarqube | sarif | junit] (default: tty)
  --error RULECODE         Make the rule `RULECODE` have the level `error`
  --warning RULECODE       Make the rule `RULECODE` have the level `warning`
  --info RULECODE          Make the rule `RULECODE` have the level `info`
  --style RULECODE         Make the rule `RULECODE` have the level `style`
  --ignore RULECODE        A rule to ignore. If present, the ignore list in the
                           config file is ignored
  --trusted-registry REGISTRY (e.g. docker.io)
                           A docker registry to allow to appear in FROM
                           instructions
  --require-label LABELSCHEMA (e.g. maintainer:text)
                           The option --require-label=label:format makes
                           Hadolint check that the label `label` conforms to
                           format requirement `format`
  --strict-labels          Do not permit labels other than specified in
                           `label-schema`
  --disable-ignore-pragma  Disable inline ignore pragmas  `# hadolint
                           ignore=DLxxxx`
  -t,--failure-threshold THRESHOLD
                           Exit with failure code only when rules with a
                           severity equal to or above THRESHOLD are violated.
                           Accepted values: [error | warning | info | style |
                           ignore | none] (default: info)
";
    HELP.to_string()
}

/// Haskell `parseCommandline` run by `execParser` over `args` (without the program name).
pub fn parse_commandline(args: &[String]) -> CommandlineOutcome {
    // DIVERGENCE: error texts are optparse-applicative-style messages without the usage block.
    let mut c = CommandlineConfig::default();
    let mut it = args.iter().peekable();
    let mut only_files = false;
    while let Some(a) = it.next() {
        if only_files || !a.starts_with('-') || a == "-" {
            c.dockerfiles.push(a.clone());
            continue;
        }
        if a == "--" {
            only_files = true;
            continue;
        }
        // split `--long=value`
        let (name, inline_val) = match a.split_once('=') {
            Some((n, v)) if n.starts_with("--") => (n.to_string(), Some(v.to_string())),
            _ => (a.clone(), None),
        };
        macro_rules! value {
            () => {
                match inline_val.clone().or_else(|| it.next().cloned()) {
                    Some(v) => v,
                    None => return CommandlineOutcome::Error(format!("The option `{name}' expects an argument.")),
                }
            };
        }
        let p = &mut c.configuration;
        match name.as_str() {
            "--version" | "-v" => c.show_version = true,
            "-h" | "--help" => return CommandlineOutcome::Help(usage_text()),
            "--config" | "-c" => c.config_file = Some(value!()),
            "--file-path-in-report" => c.file_path_in_report_option = Some(value!()),
            "--output" | "-o" => c.output.push(value!()),
            "--no-fail" => p.partial_no_fail = Some(true),
            "--no-color" => p.partial_no_color = Some(true),
            "--verbose" | "-V" => p.partial_verbose = Some(true),
            "--format" | "-f" => {
                let v = value!();
                match read_maybe_output_format(&v) {
                    Some(f) => p.partial_formats.push(f),
                    None => return CommandlineOutcome::Error(format!("option {name}: cannot parse value `{v}'")),
                }
            }
            "--error" => p.partial_error_rules.push(RuleCode::from(value!())),
            "--warning" => p.partial_warning_rules.push(RuleCode::from(value!())),
            "--info" => p.partial_info_rules.push(RuleCode::from(value!())),
            "--style" => p.partial_style_rules.push(RuleCode::from(value!())),
            "--ignore" => p.partial_ignore_rules.push(RuleCode::from(value!())),
            "--trusted-registry" => {
                p.partial_allowed_registries.insert(Registry::from(value!().as_str()));
            }
            "--require-label" => {
                let v = value!();
                match label_parser(&v) {
                    Ok((k, t)) => {
                        p.partial_label_schema.insert(k, t); // Map.fromList: later duplicates win
                    }
                    Err(e) => return CommandlineOutcome::Error(format!("option --require-label: {e}")),
                }
            }
            "--strict-labels" => p.partial_strict_labels = Some(true),
            "--disable-ignore-pragma" => p.partial_disable_ignore_pragma = Some(true),
            "--failure-threshold" | "-t" => {
                let v = value!();
                match read_either_severity(&v) {
                    Ok(s) => p.partial_failure_threshold = Some(s),
                    Err(e) => return CommandlineOutcome::Error(format!("option {name}: {e}")),
                }
            }
            _ => return CommandlineOutcome::Error(format!("Invalid option `{a}'")),
        }
    }
    CommandlineOutcome::Config(Box::new(c))
}

/// Haskell `labelParser` (`breakOn ":"`, then `readEitherLabelType . drop 1`).
pub fn label_parser(l: &str) -> Result<(String, crate::rule::LabelType), String> {
    let (ln, rest) = match l.find(':') {
        Some(i) => (&l[..i], &l[i + 1..]),
        None => (l, ""),
    };
    read_either_label_type(rest).map(|lt| (ln.to_string(), lt))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::format::OutputFormat;
    use crate::rule::{DLSeverity, LabelType};

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    fn check_commandline(args: &[&str], config: CommandlineConfig) {
        match parse_commandline(&s(args)) {
            CommandlineOutcome::Config(c) => assert_eq!(*c, config),
            _ => panic!("should have parsed commandline arguments"),
        }
    }

    fn check_commandline_fail(args: &[&str]) {
        match parse_commandline(&s(args)) {
            CommandlineOutcome::Error(_) => {}
            _ => panic!("should have failed parsing commandline arguments"),
        }
    }

    fn rc(x: &str) -> RuleCode {
        RuleCode::from(x.to_string())
    }

    fn with_cfg(f: impl FnOnce(&mut PartialConfiguration)) -> CommandlineConfig {
        let mut c = CommandlineConfig::default();
        f(&mut c.configuration);
        c
    }

    #[test]
    fn parse_with_no_arguments_given() {
        check_commandline(&[], CommandlineConfig::default());
    }

    #[test]
    fn parse_short_v() {
        check_commandline(&["-v"], CommandlineConfig { show_version: true, ..Default::default() });
    }

    #[test]
    fn parse_version() {
        check_commandline(&["--version"], CommandlineConfig { show_version: true, ..Default::default() });
    }

    #[test]
    fn parse_short_c() {
        check_commandline(
            &["-c", "hadolint.yaml"],
            CommandlineConfig { config_file: Some("hadolint.yaml".into()), ..Default::default() },
        );
    }

    #[test]
    fn parse_config() {
        check_commandline(
            &["--config", "hadolint.yaml"],
            CommandlineConfig { config_file: Some("hadolint.yaml".into()), ..Default::default() },
        );
    }

    #[test]
    fn parse_dockerfile() {
        check_commandline(&["Dockerfile"], CommandlineConfig { dockerfiles: s(&["Dockerfile"]), ..Default::default() });
    }

    #[test]
    fn parse_dockerfile1_dockerfile2() {
        check_commandline(
            &["Dockerfile1", "Dockerfile2"],
            CommandlineConfig { dockerfiles: s(&["Dockerfile1", "Dockerfile2"]), ..Default::default() },
        );
    }

    #[test]
    fn parse_file_path_in_report() {
        check_commandline(
            &["--file-path-in-report", "foobar/Dockerfile"],
            CommandlineConfig { file_path_in_report_option: Some("foobar/Dockerfile".into()), ..Default::default() },
        );
    }

    #[test]
    fn parse_single_output_destination_short_form() {
        check_commandline(&["-o", "report.log"], CommandlineConfig { output: s(&["report.log"]), ..Default::default() });
    }

    #[test]
    fn parse_single_output_destination_long_form() {
        check_commandline(&["--output", "report.log"], CommandlineConfig { output: s(&["report.log"]), ..Default::default() });
    }

    #[test]
    fn parse_multiple_output_destinations() {
        check_commandline(
            &["--output", "report.log", "-o", "other-report.log"],
            CommandlineConfig { output: s(&["report.log", "other-report.log"]), ..Default::default() },
        );
    }

    #[test]
    fn parse_no_fail() {
        check_commandline(&["--no-fail"], with_cfg(|p| p.partial_no_fail = Some(true)));
    }

    #[test]
    fn parse_no_color() {
        check_commandline(&["--no-color"], with_cfg(|p| p.partial_no_color = Some(true)));
    }

    #[test]
    fn parse_short_verbose() {
        check_commandline(&["-V"], with_cfg(|p| p.partial_verbose = Some(true)));
    }

    #[test]
    fn parse_verbose() {
        check_commandline(&["--verbose"], with_cfg(|p| p.partial_verbose = Some(true)));
    }

    #[test]
    fn parse_f_json() {
        check_commandline(&["-f", "json"], with_cfg(|p| p.partial_formats = vec![OutputFormat::Json]));
    }

    #[test]
    fn parse_format() {
        check_commandline(&["--format", "sarif"], with_cfg(|p| p.partial_formats = vec![OutputFormat::Sarif]));
    }

    #[test]
    fn parse_multiple_format() {
        check_commandline(
            &["-f", "junit", "--format", "sarif"],
            with_cfg(|p| p.partial_formats = vec![OutputFormat::JUnit, OutputFormat::Sarif]),
        );
    }

    #[test]
    fn parse_error() {
        check_commandline(&["--error", "DL3010"], with_cfg(|p| p.partial_error_rules = vec![rc("DL3010")]));
    }

    #[test]
    fn parse_error_twice() {
        check_commandline(
            &["--error", "DL3010", "--error=DL3020"],
            with_cfg(|p| p.partial_error_rules = vec![rc("DL3010"), rc("DL3020")]),
        );
    }

    #[test]
    fn parse_warning() {
        check_commandline(&["--warning", "DL3010"], with_cfg(|p| p.partial_warning_rules = vec![rc("DL3010")]));
    }

    #[test]
    fn parse_warning_twice() {
        check_commandline(
            &["--warning", "DL3010", "--warning=DL3020"],
            with_cfg(|p| p.partial_warning_rules = vec![rc("DL3010"), rc("DL3020")]),
        );
    }

    #[test]
    fn parse_info() {
        check_commandline(&["--info", "DL3010"], with_cfg(|p| p.partial_info_rules = vec![rc("DL3010")]));
    }

    #[test]
    fn parse_info_twice() {
        check_commandline(
            &["--info", "DL3010", "--info=DL3020"],
            with_cfg(|p| p.partial_info_rules = vec![rc("DL3010"), rc("DL3020")]),
        );
    }

    #[test]
    fn parse_style() {
        check_commandline(&["--style", "DL3010"], with_cfg(|p| p.partial_style_rules = vec![rc("DL3010")]));
    }

    #[test]
    fn parse_style_twice() {
        check_commandline(
            &["--style", "DL3010", "--style=DL3020"],
            with_cfg(|p| p.partial_style_rules = vec![rc("DL3010"), rc("DL3020")]),
        );
    }

    #[test]
    fn parse_ignore() {
        check_commandline(&["--ignore", "DL3010"], with_cfg(|p| p.partial_ignore_rules = vec![rc("DL3010")]));
    }

    #[test]
    fn parse_ignore_twice() {
        check_commandline(
            &["--ignore", "DL3010", "--ignore=DL3020"],
            with_cfg(|p| p.partial_ignore_rules = vec![rc("DL3010"), rc("DL3020")]),
        );
    }

    #[test]
    fn parse_trusted_registry() {
        check_commandline(
            &["--trusted-registry", "foobar.com"],
            with_cfg(|p| {
                p.partial_allowed_registries.insert(Registry::from("foobar.com"));
            }),
        );
    }

    #[test]
    fn parse_trusted_registry_twice() {
        check_commandline(
            &["--trusted-registry", "foobar.com", "--trusted-registry", "barfoo.io"],
            with_cfg(|p| {
                p.partial_allowed_registries.insert(Registry::from("foobar.com"));
                p.partial_allowed_registries.insert(Registry::from("barfoo.io"));
            }),
        );
    }

    #[test]
    fn parse_require_label() {
        check_commandline(
            &["--require-label", "foo:email"],
            with_cfg(|p| {
                p.partial_label_schema.insert("foo".into(), LabelType::Email);
            }),
        );
    }

    #[test]
    fn parse_require_label_twice() {
        check_commandline(
            &["--require-label", "foo:email", "--require-label", "bar:text"],
            with_cfg(|p| {
                p.partial_label_schema.insert("foo".into(), LabelType::Email);
                p.partial_label_schema.insert("bar".into(), LabelType::RawText);
            }),
        );
    }

    #[test]
    fn parse_strict_labels() {
        check_commandline(&["--strict-labels"], with_cfg(|p| p.partial_strict_labels = Some(true)));
    }

    #[test]
    fn parse_disable_ignore_pragma() {
        check_commandline(
            &["--disable-ignore-pragma"],
            with_cfg(|p| p.partial_disable_ignore_pragma = Some(true)),
        );
    }

    #[test]
    fn parse_t_warning() {
        check_commandline(
            &["-t", "warning"],
            with_cfg(|p| p.partial_failure_threshold = Some(DLSeverity::DLWarningC)),
        );
    }

    #[test]
    fn parse_failure_threshold_style() {
        check_commandline(
            &["--failure-threshold", "style"],
            with_cfg(|p| p.partial_failure_threshold = Some(DLSeverity::DLStyleC)),
        );
    }

    #[test]
    fn fail_parsing_blafoo() {
        check_commandline_fail(&["--blafoo"]);
    }

    #[test]
    fn fail_parsing_require_label_foo_bar() {
        check_commandline_fail(&["--require-label", "foo:bar"]);
    }
}
