// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/hadolint/src/Hadolint/Config.hs (re-exports).

pub mod commandline;
pub mod configfile;
pub mod configuration;
pub mod environment;

pub use commandline::{parse_commandline, CommandlineConfig, CommandlineOutcome};
pub use configfile::get_config_from_file;
pub use configuration::*;
pub use environment::get_config_from_environment;
