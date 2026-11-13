// SPDX-License-Identifier: GPL-3.0-only
//! Port of upstream/language-docker/src/Language/Docker/Parser/From.hs.
//!
//! Parsers are `fn(&mut Parser) -> PResult<T>`; `?esc` is `p.esc`. See `combinator.rs`.

#![allow(unused_variables)]
use crate::combinator::{PResult, Parser};
use crate::prelude::{required_whitespace, reserved, some_unless};
use dlint_syntax::*;

/// Haskell `parseRegistry` (From.hs).
pub fn parse_registry(p: &mut Parser) -> PResult<Registry> {
    let domain = some_unless_expanded(p, "a domain name", &|c| c == '.')?;
    p.char('.')?;
    let tld = some_unless_expanded(p, "a TLD", &|c| c == '/')?;
    p.char('/')?;
    Ok(Registry {
        un_registry: format!("{domain}.{tld}"),
    })
}

/// Haskell `parsePlatform` (From.hs).
pub fn parse_platform(p: &mut Parser) -> PResult<Platform> {
    p.string("--platform=")?;
    let pl = some_unless(p, "the platform for the FROM image", &|c| c == ' ')?;
    required_whitespace(p)?;
    Ok(pl)
}

/// Haskell `someUnlessExpanded` (From.hs).
pub fn some_unless_expanded(
    p: &mut Parser,
    name: &str,
    predicate: &dyn Fn(char) -> bool,
) -> PResult<String> {
    let literal_run = |s: &mut Parser| some_unless(s, name, &|c| predicate(c) || c == '$');
    p.label(name, |s| {
        let v = s.some(|s| s.alt(variable_expansion, literal_run))?;
        Ok(v.concat())
    })
}

/// Haskell `variableExpansion` (From.hs).
pub fn variable_expansion(p: &mut Parser) -> PResult<String> {
    p.char('$')?;
    let brace = p.optional(|s| s.char('{'))?;
    match brace {
        None => Ok("$".to_string()),
        Some(_) => {
            let rest = braces(p, Vec::new(), 1)?;
            Ok(format!("${{{rest}"))
        }
    }
}

/// `braces` (local to `variableExpansion`); `acc` is kept in forward order.
fn braces(p: &mut Parser, mut acc: Vec<String>, mut depth: i64) -> PResult<String> {
    loop {
        let piece = p.take_while_p(None, |c| !matches!(c, '{' | '}' | ' ' | '\t' | '\n'))?;
        let next = p.optional(|s| s.alt(|s| s.char('{'), |s| s.char('}')))?;
        match next {
            Some('{') => {
                acc.push(piece);
                acc.push("{".to_string());
                depth += 1;
            }
            Some('}') => {
                acc.push(piece);
                acc.push("}".to_string());
                if depth <= 1 {
                    return Ok(acc.concat());
                }
                depth -= 1;
            }
            _ => {
                acc.push(piece);
                return Ok(acc.concat());
            }
        }
    }
}

/// Haskell `parseBaseImage` (From.hs).
pub fn parse_base_image(
    p: &mut Parser,
    tag_parser: &dyn Fn(&mut Parser, &str) -> PResult<Option<Tag>>,
) -> PResult<BaseImage> {
    let maybe_platform = p.alt(|s| s.try_(parse_platform).map(Some), |_| Ok(None))?;
    p.not_followed_by(|s| s.string("--"))?;
    let reg_name = p.alt(|s| s.try_(parse_registry).map(Some), |_| Ok(None))?;
    let name = some_unless_expanded(p, "the image name with a tag", &|c| c == '@' || c == ':')?;
    let maybe_tag = p.alt(|s| tag_parser(s, &name), |_| Ok(None))?;
    let maybe_digest = p.alt(|s| s.try_(parse_digest).map(Some), |_| Ok(None))?;
    let maybe_alias = p.alt(
        |s| {
            s.try_(|s| {
                required_whitespace(s)?;
                image_alias(s)
            })
            .map(Some)
        },
        |_| Ok(None),
    )?;
    Ok(BaseImage {
        image: Image {
            registry_name: reg_name,
            image_name: name,
        },
        tag: maybe_tag,
        digest: maybe_digest,
        alias: maybe_alias,
        platform: maybe_platform,
    })
}

/// Haskell `taggedImage` (From.hs).
pub fn tagged_image(p: &mut Parser) -> PResult<BaseImage> {
    parse_base_image(p, &|s, _| {
        s.char(':')?;
        let t = some_unless_expanded(s, "the image tag", &|c| c == '@' || c == ':')?;
        Ok(Some(Tag { un_tag: t }))
    })
}

/// Haskell `parseDigest` (From.hs).
pub fn parse_digest(p: &mut Parser) -> PResult<Digest> {
    p.char('@')?;
    let d = some_unless(p, "the image digest", &|c| c == '@')?;
    Ok(Digest { un_digest: d })
}

/// Haskell `untaggedImage` (From.hs).
pub fn untagged_image(p: &mut Parser) -> PResult<BaseImage> {
    parse_base_image(p, &|s, name| {
        let l = format!("no ':' or a valid image tag string (example: {name}:valid-tag)");
        s.label(&l, |s| s.try_(|s| s.not_followed_by(|s| s.string(":"))))?;
        Ok(None)
    })
}

/// Haskell `imageAlias` (From.hs).
pub fn image_alias(p: &mut Parser) -> PResult<ImageAlias> {
    p.label("'AS' followed by the image alias", |s| {
        s.try_(|s| reserved(s, "AS"))
    })?;
    let aka = some_unless(p, "the image alias", &|c| c == '\n')?;
    Ok(ImageAlias {
        un_image_alias: aka,
    })
}

/// Haskell `baseImage` (From.hs).
pub fn base_image(p: &mut Parser) -> PResult<BaseImage> {
    p.alt(|s| s.try_(tagged_image), untagged_image)
}

/// Haskell `parseFrom` (From.hs).
pub fn parse_from(p: &mut Parser) -> PResult<Instruction<String>> {
    reserved(p, "FROM")?;
    Ok(Instruction::From(base_image(p)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combinator::run_parser;

    // DIVERGENCE: the Haskell spec goes through `assertAst` (the full Dockerfile parser); these
    // tests drive `parse_from` directly so they do not depend on other stubbed parsers.
    fn ast(src: &str) -> Instruction<String> {
        match run_parser(src, "<string>", '\\', parse_from) {
            Ok(i) => i,
            Err(e) => panic!("{}", e.bundle_pretty()),
        }
    }

    fn img(reg: Option<&str>, name: &str) -> Image {
        Image {
            registry_name: reg.map(Registry::from),
            image_name: name.to_string(),
        }
    }
    fn untagged(image: Image) -> BaseImage {
        BaseImage {
            image,
            tag: None,
            digest: None,
            alias: None,
            platform: None,
        }
    }
    fn tagged(image: Image, t: &str) -> BaseImage {
        BaseImage {
            tag: Some(Tag::from(t)),
            ..untagged(image)
        }
    }
    fn n(name: &str) -> Image {
        img(None, name)
    }
    fn from(b: BaseImage) -> Instruction<String> {
        Instruction::From(b)
    }

    #[test]
    fn parse_untagged_image() {
        assert_eq!(ast("FROM busybox"), from(untagged(n("busybox"))));
    }
    #[test]
    fn parse_tagged_image() {
        assert_eq!(
            ast("FROM busybox:5.12-dev"),
            from(tagged(n("busybox"), "5.12-dev"))
        );
    }
    #[test]
    fn parse_digested_image() {
        let b = BaseImage {
            digest: Some(Digest::from("sha256:0ef2e08ed3fab")),
            ..untagged(n("ubuntu"))
        };
        assert_eq!(ast("FROM ubuntu@sha256:0ef2e08ed3fab"), from(b));
    }
    #[test]
    fn parse_digested_image_with_tag() {
        let b = BaseImage {
            digest: Some(Digest::from("sha256:0ef2e08ed3fab")),
            ..tagged(n("ubuntu"), "14.04")
        };
        assert_eq!(ast("FROM ubuntu:14.04@sha256:0ef2e08ed3fab"), from(b));
    }
    #[test]
    fn parse_image_with_spaces_at_the_end() {
        assert_eq!(
            ast("FROM dockerfile/mariadb "),
            from(untagged(n("dockerfile/mariadb")))
        );
    }
    #[test]
    fn aliased_untagged_image() {
        let b = BaseImage {
            alias: Some(ImageAlias::from("foo")),
            ..untagged(n("busybox"))
        };
        assert_eq!(ast("FROM busybox as foo"), from(b));
    }
    #[test]
    fn aliased_tagged_image() {
        let b = BaseImage {
            alias: Some(ImageAlias::from("foo-bar")),
            ..tagged(n("busybox"), "5.12-dev")
        };
        assert_eq!(ast("FROM busybox:5.12-dev AS foo-bar"), from(b));
    }
    #[test]
    fn aliased_digested_image() {
        let b = BaseImage {
            digest: Some(Digest::from("sha256:0ef2e08ed3fab")),
            alias: Some(ImageAlias::from("foo")),
            ..untagged(n("ubuntu"))
        };
        assert_eq!(ast("FROM ubuntu@sha256:0ef2e08ed3fab AS foo"), from(b));
    }
    #[test]
    fn untagged_image_with_platform() {
        let b = BaseImage {
            platform: Some("linux".into()),
            ..untagged(n("busybox"))
        };
        assert_eq!(ast("FROM --platform=linux busybox"), from(b));
    }
    #[test]
    fn tagged_image_with_platform() {
        let b = BaseImage {
            platform: Some("linux".into()),
            ..tagged(n("busybox"), "foo")
        };
        assert_eq!(ast("FROM --platform=linux busybox:foo"), from(b));
    }
    #[test]
    fn registry_without_port() {
        assert_eq!(
            ast("FROM foo.com/node"),
            from(untagged(img(Some("foo.com"), "node")))
        );
    }
    #[test]
    fn parse_with_port_and_tag() {
        assert_eq!(
            ast("FROM myregistry.com:5000/imagename:5.12-dev"),
            from(tagged(
                img(Some("myregistry.com:5000"), "imagename"),
                "5.12-dev"
            ))
        );
    }
    #[test]
    fn not_a_registry_if_no_tld() {
        assert_eq!(
            ast("FROM myfolder/imagename:5.12-dev"),
            from(tagged(img(None, "myfolder/imagename"), "5.12-dev"))
        );
    }
    #[test]
    fn keeps_colon_inside_an_expansion_out_of_the_tag() {
        assert_eq!(
            ast("FROM ${IMAGE_PREFIX:?}${DISTRO:?}:${VERSION_DISTRO:?}"),
            from(tagged(
                n("${IMAGE_PREFIX:?}${DISTRO:?}"),
                "${VERSION_DISTRO:?}"
            ))
        );
    }
    #[test]
    fn handles_minus_and_plus_operators() {
        assert_eq!(
            ast("FROM ${BASE:-alpine}:${TAG:+edge}"),
            from(tagged(n("${BASE:-alpine}"), "${TAG:+edge}"))
        );
    }
    #[test]
    fn handles_a_nested_expansion() {
        assert_eq!(
            ast("FROM ${BASE:-${DEFAULT}}"),
            from(untagged(n("${BASE:-${DEFAULT}}")))
        );
    }
    #[test]
    fn keeps_expansion_with_a_slash_in_the_registry() {
        assert_eq!(
            ast("FROM ${REGISTRY:-docker.io/library}/myimage:tag"),
            from(tagged(n("${REGISTRY:-docker.io/library}/myimage"), "tag"))
        );
    }
    #[test]
    fn an_unterminated_expansion_stays_on_its_line() {
        // DIVERGENCE: the trailing `RUN y` instruction is not parsed here (parse_from only).
        assert_eq!(ast("FROM ${BASE:x\nRUN y"), from(untagged(n("${BASE:x"))));
    }
    #[test]
    fn an_unterminated_expansion_keeps_a_following_alias() {
        let b = BaseImage {
            alias: Some(ImageAlias::from("build")),
            ..untagged(n("${BASE"))
        };
        assert_eq!(ast("FROM ${BASE AS build"), from(b));
    }
    #[test]
    fn nested_expansion_with_unterminated_brace() {
        assert_eq!(
            ast("FROM ${BASE:-${DEFAULT}"),
            from(untagged(n("${BASE:-${DEFAULT}")))
        );
    }
}
