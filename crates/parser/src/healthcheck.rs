// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/Healthcheck.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use dlint_syntax::*;
use crate::cmd::parse_cmd;
use crate::combinator::{PResult, Parser};
use crate::prelude::*;

/// Haskell `data CheckFlag` (Healthcheck.hs, module-private upstream).
#[derive(Debug, Clone)]
pub enum CheckFlag {
    FlagInterval(Duration),
    FlagTimeout(Duration),
    FlagStartPeriod(Duration),
    FlagStartInterval(Duration),
    FlagRetries(Retries),
    CFlagInvalid((String, String)),
}

/// Haskell `parseHealthcheck` (Healthcheck.hs).
pub fn parse_healthcheck(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "HEALTHCHECK")?;
    let c = p.alt(full_check, no_check)?;
    Ok(Instruction::Healthcheck(c))
}

fn no_check(p: &mut Parser) -> PResult<Check<String>> {
    p.string("NONE")?;
    Ok(Check::NoCheck)
}

fn all_flags(p: &mut Parser) -> PResult<Vec<CheckFlag>> {
    let flags = some_flags(p)?;
    p.label("another flag", required_whitespace)?;
    Ok(flags)
}

fn some_flags(p: &mut Parser) -> PResult<Vec<CheckFlag>> {
    let x = check_flag(p)?;
    let cont = p.alt(
        |p| {
            p.try_(|p| {
                required_whitespace(p)?;
                p.look_ahead(|p| p.string("--"))?;
                Ok(true)
            })
        },
        |_| Ok(false),
    )?;
    if cont {
        let mut xs = some_flags(p)?;
        xs.insert(0, x);
        Ok(xs)
    } else {
        Ok(vec![x])
    }
}

fn full_check(p: &mut Parser) -> PResult<Check<String>> {
    let flags = p.alt(all_flags, |_| Ok(vec![]))?;
    let mut intervals = vec![];
    let mut timeouts = vec![];
    let mut start_periods = vec![];
    let mut start_intervals = vec![];
    let mut retries_d = vec![];
    let mut invalid = vec![];
    for f in flags {
        match f {
            CheckFlag::FlagInterval(x) => intervals.push(x),
            CheckFlag::FlagTimeout(x) => timeouts.push(x),
            CheckFlag::FlagStartPeriod(x) => start_periods.push(x),
            CheckFlag::FlagStartInterval(x) => start_intervals.push(x),
            CheckFlag::FlagRetries(x) => retries_d.push(x),
            CheckFlag::CFlagInvalid(x) => invalid.push(x),
        }
    }
    // Let's do some validation on the flags
    if let Some((k, v)) = invalid.first() {
        return unexpected_flag(p, k, v);
    }
    if intervals.len() >= 2 {
        return custom_error(p, DockerfileError::DuplicateFlagError("--interval".to_string()));
    }
    if timeouts.len() >= 2 {
        return custom_error(p, DockerfileError::DuplicateFlagError("--timeout".to_string()));
    }
    if start_periods.len() >= 2 {
        return custom_error(p, DockerfileError::DuplicateFlagError("--start-period".to_string()));
    }
    if start_intervals.len() >= 2 {
        return custom_error(p, DockerfileError::DuplicateFlagError("--start-interval".to_string()));
    }
    if retries_d.len() >= 2 {
        return custom_error(p, DockerfileError::DuplicateFlagError("--retries".to_string()));
    }
    let check_command = match parse_cmd(p)? {
        Instruction::Cmd(c) => c,
        _ => return p.fail("Pattern match failure in do expression"),
    };
    Ok(Check::Check(CheckArgs {
        check_command,
        interval: intervals.first().copied(),
        timeout: timeouts.first().copied(),
        start_period: start_periods.first().copied(),
        start_interval: start_intervals.first().copied(),
        retries: retries_d.first().copied(),
    }))
}

/// Haskell `checkFlag` (Healthcheck.hs).
pub fn check_flag(p: &mut Parser) -> PResult<CheckFlag> {
    p.choice(&[
        &|p| p.label("--interval", |p| duration_flag(p, "--interval=").map(CheckFlag::FlagInterval)),
        &|p| p.label("--timeout", |p| duration_flag(p, "--timeout=").map(CheckFlag::FlagTimeout)),
        &|p| {
            p.label("--start-period", |p| {
                duration_flag(p, "--start-period=").map(CheckFlag::FlagStartPeriod)
            })
        },
        &|p| {
            p.label("--start-interval", |p| {
                duration_flag(p, "--start-interval=").map(CheckFlag::FlagStartInterval)
            })
        },
        &|p| p.label("--retries", |p| retries_flag(p).map(CheckFlag::FlagRetries)),
        &|p| p.label("no flags", |p| any_flag(p).map(CheckFlag::CFlagInvalid)),
    ])
}

/// Haskell `fromRational . realToFrac` from `Float` to `Pico` (floor to picoseconds, exact).
fn float_to_pico(f: f32) -> i128 {
    let bits = f.to_bits();
    let exp = ((bits >> 23) & 0xff) as i32;
    let frac = (bits & 0x7f_ffff) as i128;
    let (m, e) = if exp == 0 { (frac, -149) } else { (frac | 0x80_0000, exp - 150) };
    let scaled = m * 1_000_000_000_000i128;
    if e >= 0 {
        scaled << e
    } else if -e >= 127 {
        0
    } else {
        scaled >> (-e)
    }
}

/// Haskell `durationFlag` (Healthcheck.hs).
pub fn duration_flag(p: &mut Parser, flag_name: &str) -> PResult<Duration> {
    p.try_(|p| p.string(flag_name))?;
    let value = p.label("a natural or fractional number", |p| {
        p.alt(
            |p| p.try_(|p| fractional(p).map(|f| DiffTime(float_to_pico(f)))),
            |p| natural(p).map(|n| DiffTime::from_seconds(n as i128)),
        )
    })?;
    let unit = p.label("either 's', 'm' or 'h' as the unit", |p| {
        p.choice(&[&|p| p.char('s'), &|p| p.char('m'), &|p| p.char('h')])
    })?;
    match unit {
        's' => Ok(Duration { duration_time: value }),
        'm' => Ok(Duration { duration_time: DiffTime(value.0 * 60) }),
        'h' => Ok(Duration { duration_time: DiffTime(value.0 * 60 * 60) }),
        _ => p.fail("only 's', 'm' or 'h' are allowed as the duration"),
    }
}

/// Haskell `retriesFlag` (Healthcheck.hs).
pub fn retries_flag(p: &mut Parser) -> PResult<Retries> {
    p.try_(|p| p.string("--retries="))?;
    let n = p.label("the number of retries", |p| p.try_(natural))?;
    // DIVERGENCE: `fromIntegral :: Integer -> Int` wraps; `as i64` wraps likewise (u128 source).
    Ok(Retries { times: n as i64 })
}

/// Haskell `anyFlag` (Healthcheck.hs).
pub fn any_flag(p: &mut Parser) -> PResult<(String, String)> {
    p.string("--")?;
    let name = some_unless(p, "the flag value", &|c| c == '=')?;
    p.char('=')?;
    let val = any_unless(p, &|c| c == ' ')?;
    Ok((format!("--{name}"), val))
}

/// Haskell `unexpectedFlag` (Healthcheck.hs).
pub fn unexpected_flag<T>(p: &mut Parser, name: &str, value: &str) -> PResult<T> {
    if value.is_empty() {
        p.custom_failure(DockerfileError::NoValueFlagError(name.to_string()))
    } else {
        p.custom_failure(DockerfileError::InvalidFlagError(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_text;

    fn d(pico: i128) -> Option<Duration> {
        Some(Duration { duration_time: DiffTime(pico) })
    }
    fn secs(s: i128) -> Option<Duration> {
        Some(Duration::from_seconds(s))
    }
    fn check(cmd: &str, i: Option<Duration>, t: Option<Duration>, sp: Option<Duration>, si: Option<Duration>, r: Option<i64>) -> Instruction<String> {
        Instruction::Healthcheck(Check::Check(CheckArgs {
            check_command: Arguments::ArgumentsText(cmd.to_string()),
            interval: i,
            timeout: t,
            start_period: sp,
            start_interval: si,
            retries: r.map(|times| Retries { times }),
        }))
    }
    fn assert_ast(src: &str, expected: Vec<Instruction<String>>) {
        let got: Vec<_> = parse_text(src).expect("parse failed").into_iter().map(|i| i.instruction).collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn parse_healthcheck_with_interval() {
        assert_ast(
            "HEALTHCHECK --interval=5m \\\nCMD curl -f http://localhost/",
            vec![check("curl -f http://localhost/", secs(300), None, None, None, None)],
        );
    }
    #[test]
    fn parse_healthcheck_with_retries() {
        assert_ast(
            "HEALTHCHECK --retries=10 CMD curl -f http://localhost/",
            vec![check("curl -f http://localhost/", None, None, None, None, Some(10))],
        );
    }
    #[test]
    fn parse_healthcheck_with_timeout() {
        assert_ast(
            "HEALTHCHECK --timeout=10s CMD curl -f http://localhost/",
            vec![check("curl -f http://localhost/", None, secs(10), None, None, None)],
        );
    }
    #[test]
    fn parse_healthcheck_with_start_period() {
        assert_ast(
            "HEALTHCHECK --start-period=2m CMD curl -f http://localhost/",
            vec![check("curl -f http://localhost/", None, None, secs(120), None, None)],
        );
    }
    #[test]
    fn parse_healthcheck_with_start_interval() {
        assert_ast(
            "HEALTHCHECK --start-interval=4m CMD curl -f http://localhost/",
            vec![check("curl -f http://localhost/", None, None, None, secs(240), None)],
        );
    }
    #[test]
    fn parse_healthcheck_with_all_flags() {
        assert_ast(
            "HEALTHCHECK --start-period=2s --start-interval=10s --timeout=1m --retries=3 --interval=5s    CMD curl -f http://localhost/",
            vec![check("curl -f http://localhost/", secs(5), secs(60), secs(2), secs(10), Some(3))],
        );
    }
    #[test]
    fn parse_healthcheck_with_no_flags() {
        assert_ast(
            "HEALTHCHECK CMD curl -f http://localhost/",
            vec![check("curl -f http://localhost/", None, None, None, None, None)],
        );
    }
    #[test]
    fn fractional_arguments_to_flags() {
        let file = [
            "HEALTHCHECK \\",
            "  --interval=0.5s \\",
            "  --timeout=0.1s \\",
            "  --start-period=0.2s \\",
            "  --start-interval=0.5s \\",
            "  CMD curl -f http://localhost",
        ]
        .join("\n")
            + "\n";
        assert_ast(
            &file,
            vec![check(
                "curl -f http://localhost",
                d(500_000_000_000),
                d(100_000_001_490),
                d(200_000_002_980),
                d(500_000_000_000),
                None,
            )],
        );
    }
}
