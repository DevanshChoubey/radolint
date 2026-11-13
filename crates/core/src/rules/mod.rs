// SPDX-License-Identifier: GPL-3.0-only
//! One module per rule, plus the registry [`failures`] (Process.hs `failures`).

pub mod dl1001;
pub mod dl3000;
pub mod dl3001;
pub mod dl3002;
pub mod dl3003;
pub mod dl3004;
pub mod dl3006;
pub mod dl3007;
pub mod dl3008;
pub mod dl3009;
pub mod dl3010;
pub mod dl3011;
pub mod dl3012;
pub mod dl3013;
pub mod dl3014;
pub mod dl3015;
pub mod dl3016;
pub mod dl3018;
pub mod dl3019;
pub mod dl3020;
pub mod dl3021;
pub mod dl3022;
pub mod dl3023;
pub mod dl3024;
pub mod dl3025;
pub mod dl3026;
pub mod dl3027;
pub mod dl3028;
pub mod dl3029;
pub mod dl3030;
pub mod dl3032;
pub mod dl3033;
pub mod dl3034;
pub mod dl3035;
pub mod dl3036;
pub mod dl3037;
pub mod dl3038;
pub mod dl3040;
pub mod dl3041;
pub mod dl3042;
pub mod dl3043;
pub mod dl3044;
pub mod dl3045;
pub mod dl3046;
pub mod dl3047;
pub mod dl3048;
pub mod dl3049;
pub mod dl3050;
pub mod dl3051;
pub mod dl3052;
pub mod dl3053;
pub mod dl3054;
pub mod dl3055;
pub mod dl3056;
pub mod dl3057;
pub mod dl3058;
pub mod dl3059;
pub mod dl3060;
pub mod dl3061;
pub mod dl3062;
pub mod dl3063;
pub mod dl3064;
pub mod dl3065;
pub mod dl3066;
pub mod dl3067;
pub mod dl4000;
pub mod dl4001;
pub mod dl4003;
pub mod dl4004;
pub mod dl4005;
pub mod dl4006;
pub mod shellcheck;

use crate::config::configuration::Configuration;
use crate::rule::{Rule, mconcat};

/// Haskell `Process.failures`: all rules combined in upstream order.
pub fn failures(config: &Configuration) -> Box<dyn Rule> {
    let allowed_registries = &config.allowed_registries;
    let label_schema = &config.label_schema;
    let strict_labels = &config.strict_labels;
    mconcat(vec![
        dl1001::rule(),
        dl3000::rule(),
        dl3001::rule(),
        dl3002::rule(),
        dl3003::rule(),
        dl3004::rule(),
        dl3006::rule(),
        dl3007::rule(),
        dl3008::rule(),
        dl3009::rule(),
        dl3010::rule(),
        dl3011::rule(),
        dl3012::rule(),
        dl3013::rule(),
        dl3014::rule(),
        dl3015::rule(),
        dl3016::rule(),
        dl3018::rule(),
        dl3019::rule(),
        dl3020::rule(),
        dl3021::rule(),
        dl3022::rule(),
        dl3023::rule(),
        dl3024::rule(),
        dl3025::rule(),
        dl3026::rule(allowed_registries),
        dl3027::rule(),
        dl3028::rule(),
        dl3029::rule(),
        dl3030::rule(),
        dl3032::rule(),
        dl3033::rule(),
        dl3034::rule(),
        dl3035::rule(),
        dl3036::rule(),
        dl3037::rule(),
        dl3038::rule(),
        dl3040::rule(),
        dl3041::rule(),
        dl3042::rule(),
        dl3043::rule(),
        dl3044::rule(),
        dl3045::rule(),
        dl3046::rule(),
        dl3047::rule(),
        dl3048::rule(),
        dl3049::rule(label_schema),
        dl3050::rule(label_schema, *strict_labels),
        dl3051::rule(label_schema),
        dl3052::rule(label_schema),
        dl3053::rule(label_schema),
        dl3054::rule(label_schema),
        dl3055::rule(label_schema),
        dl3056::rule(label_schema),
        dl3057::rule(),
        dl3058::rule(label_schema),
        dl3059::rule(),
        dl3060::rule(),
        dl3061::rule(),
        dl3062::rule(),
        dl3063::rule(),
        dl3064::rule(),
        dl3065::rule(),
        dl3066::rule(),
        dl3067::rule(),
        dl4000::rule(),
        dl4001::rule(),
        dl4003::rule(),
        dl4004::rule(),
        dl4005::rule(),
        dl4006::rule(),
        shellcheck::rule(),
    ])
}
