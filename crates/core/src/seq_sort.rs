// SPDX-License-Identifier: GPL-3.0-only
//! Faithful emulation of `Data.Sequence.unstableSort` (containers-0.7, the version linked into the
//! oracle binary) for a `Seq` built by `Seq.filter`.
//!
//! Upstream `Formatter.Format.toResult` sorts failures with `Seq.unstableSort`, whose order for
//! elements that compare equal (same line) depends on the finger-tree shape and the pairing-heap
//! merge order. `Seq.filter p = foldl' (\xs x -> if p x then xs `snoc'` x else xs) empty`, so the
//! tree shape is fixed by the element count alone and is rebuilt here by repeated snocs.
//!
//! Source references (containers v0.7): `Data.Sequence.Internal` (`snocTree'`, `foldDigit`,
//! `foldNode`) and `Data.Sequence.Internal.Sorting` (`unstableSortBy`, `buildQ`,
//! `foldToMaybeTree`, `mergeQ`, `popMinQ`).

use crate::rule::CheckFailure;

/// A finger-tree element: a leaf (`Elem`, holding an index into the input) or a `Node2`/`Node3`
/// (2 or 3 children one level down).
enum Item {
    Elem(usize),
    Node(Vec<Item>),
}

/// Digits `One`..`Four` are `Vec`s of length 1..4.
enum Tree {
    Empty,
    Single(Item),
    Deep {
        pr: Vec<Item>,
        m: Box<Tree>,
        sf: Vec<Item>,
    },
}

/// `snocTree'`.
fn snoc(t: Tree, x: Item) -> Tree {
    match t {
        Tree::Empty => Tree::Single(x),
        Tree::Single(a) => Tree::Deep {
            pr: vec![a],
            m: Box::new(Tree::Empty),
            sf: vec![x],
        },
        Tree::Deep { pr, m, mut sf } => {
            if sf.len() == 4 {
                // Deep s pr m (Four a b c d) `snoc` e = Deep pr (m `snoc` node3 a b c) (Two d e)
                let e = x;
                let d = sf.pop().unwrap();
                let abc = Item::Node(sf); // a, b, c
                Tree::Deep {
                    pr,
                    m: Box::new(snoc(*m, abc)),
                    sf: vec![d, e],
                }
            } else {
                sf.push(x);
                Tree::Deep { pr, m, sf }
            }
        }
    }
}

/// Pairing heap `Queue e = Q !e (QList e)`; `ts` holds the sub-heaps in REVERSE list order
/// (head of the Haskell list is the last element), so `QCons` is a `push`.
struct Q {
    x: usize,
    ts: Vec<Q>,
}

/// `mergeQ cmp`: `cmp x1 x2 == GT` => `Q x2 (q1 : ts2)`, otherwise `Q x1 (q2 : ts1)`.
fn merge(keys: &[i64], q1: Q, q2: Q) -> Q {
    if keys[q1.x] > keys[q2.x] {
        let mut ts2 = q2.ts;
        let x2 = q2.x;
        ts2.push(q1);
        Q { x: x2, ts: ts2 }
    } else {
        let mut ts1 = q1.ts;
        let x1 = q1.x;
        ts1.push(q2);
        Q { x: x1, ts: ts1 }
    }
}

/// Left fold of `<+>` over a non-empty list (`f a <+> f b <+> f c`, infixl).
fn fold_all(keys: &[i64], qs: impl IntoIterator<Item = Q>) -> Q {
    let mut it = qs.into_iter();
    let first = it.next().expect("non-empty digit/node");
    it.fold(first, |acc, q| merge(keys, acc, q))
}

/// `foldDigit (<+>) f` / `foldNode (<+>) f` composed through the levels.
fn fold_item(keys: &[i64], it: &Item) -> Q {
    match it {
        Item::Elem(i) => Q { x: *i, ts: vec![] },
        Item::Node(cs) => fold_all(keys, cs.iter().map(|c| fold_item(keys, c))),
    }
}

/// `foldToMaybeTree (<+>) f`.
fn fold_tree(keys: &[i64], t: &Tree) -> Option<Q> {
    match t {
        Tree::Empty => None,
        Tree::Single(x) => Some(fold_item(keys, x)),
        Tree::Deep { pr, m, sf } => {
            let pr_q = fold_all(keys, pr.iter().map(|c| fold_item(keys, c)));
            let sf_q = fold_all(keys, sf.iter().map(|c| fold_item(keys, c)));
            let base = merge(keys, pr_q, sf_q);
            Some(match fold_tree(keys, m) {
                None => base,
                Some(mq) => merge(keys, base, mq),
            })
        }
    }
}

/// `mergeQs` of `popMinQ`: pairs from the left, `(t1 <+> t2) <+> mergeQs ts`.
fn merge_pairs(keys: &[i64], mut ts: Vec<Q>) -> Q {
    // `ts` is in list order here (head first).
    let mut items: std::collections::VecDeque<Q> = std::mem::take(&mut ts).into();
    fn go(keys: &[i64], items: &mut std::collections::VecDeque<Q>) -> Q {
        let t1 = items
            .pop_front()
            .expect("popMinQ: tried to pop from empty queue");
        match items.pop_front() {
            None => t1,
            Some(t2) => {
                let h = merge(keys, t1, t2);
                if items.is_empty() {
                    h
                } else {
                    let r = go(keys, items);
                    merge(keys, h, r)
                }
            }
        }
    }
    go(keys, &mut items)
}

/// `Seq.unstableSort (foldl' snoc' empty v)` keyed on `CheckFailure.line` (the `Ord` instance).
pub fn unstable_sort_by_line(v: Vec<CheckFailure>) -> Vec<CheckFailure> {
    let n = v.len();
    if n < 2 {
        return v;
    }
    let keys: Vec<i64> = v.iter().map(|f| f.line).collect();
    let mut tree = Tree::Empty;
    for i in 0..n {
        tree = snoc(tree, Item::Elem(i));
    }
    let mut q = fold_tree(&keys, &tree).expect("non-empty");
    let mut slots: Vec<Option<CheckFailure>> = v.into_iter().map(Some).collect();
    let mut out = Vec::with_capacity(n);
    for k in 0..n {
        out.push(slots[q.x].take().expect("each index popped once"));
        if k + 1 < n {
            // list order: head is last element of `ts`
            let mut ts = q.ts;
            ts.reverse();
            q = merge_pairs(&keys, ts);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::configuration::Configuration;
    use crate::lint::analyze;
    use crate::rule::{DLSeverity, RuleCode};
    use radolint_parser::parse_text;

    fn cf(code: &str, line: i64) -> CheckFailure {
        CheckFailure {
            code: RuleCode::from(code),
            severity: DLSeverity::DLInfoC,
            message: String::new(),
            line,
        }
    }

    /// Lint `dockerfile`, sort via the Seq emulation and compare `line:code` against the order the
    /// oracle (hadolint v2.15.1 binary) printed.
    fn check(dockerfile: &str, expected: &str) {
        let d = parse_text(dockerfile).expect("parse");
        let got = unstable_sort_by_line(analyze(&Configuration::default(), &d));
        let got: Vec<String> = got
            .iter()
            .map(|f| format!("{}:{}", f.line, f.code.un_rule_code))
            .collect();
        assert_eq!(got.join(" "), expected);
    }

    #[test]
    fn small_inputs() {
        assert!(unstable_sort_by_line(vec![]).is_empty());
        let v = unstable_sort_by_line(vec![cf("B", 2), cf("A", 1)]);
        assert_eq!(v[0].code.un_rule_code, "A");
    }

    #[test]
    fn is_a_permutation_sorted_by_line() {
        for n in 0..200usize {
            let v: Vec<CheckFailure> = (0..n)
                .map(|i| cf(&i.to_string(), ((i * 7) % 5) as i64))
                .collect();
            let s = unstable_sort_by_line(v);
            assert_eq!(s.len(), n);
            assert!(s.windows(2).all(|w| w[0].line <= w[1].line));
            let mut codes: Vec<usize> = s
                .iter()
                .map(|f| f.code.un_rule_code.parse().unwrap())
                .collect();
            codes.sort();
            assert_eq!(codes, (0..n).collect::<Vec<_>>());
        }
    }

    #[test]
    fn oracle_3_findings() {
        check(
            "FROM ubuntu:latest\nADD a b\nWORKDIR /a/b\nFROM ubuntu\n",
            "1:DL3007 2:DL3020 4:DL3006",
        );
    }

    #[test]
    fn oracle_5_findings() {
        check(
            "FROM debian:9\nONBUILD RUN zypper install p && mkdir x && su root\nRUN set -e; wget http://x/y\n",
            "2:DL3034 2:DL3036 2:SC2117 2:DL3037 3:DL3047",
        );
    }

    #[test]
    fn oracle_9_findings() {
        check(
            "FROM debian:9\nRUN pip3 install --upgrade pip && sudo pip install y && zypper install p\nRUN apt-get install a b c\n",
            "2:DL3004 2:DL3013 2:DL3037 2:DL3034 2:DL3042 2:DL3036 3:DL3014 3:DL3015 3:DL3008",
        );
    }

    #[test]
    fn oracle_13_findings() {
        check(
            "FROM debian:9\nRUN su root && apk update && sudo apt-get install -y q && apt-get dist-upgrade && yarn add left-pad && curl http://x | sh && su root && apk add git && mkdir x\nONBUILD RUN npm i z && cat file | wc && sudo pip install y && chmod 777 /x && apt-get dist-upgrade\nARG X\nCOPY a b\nONBUILD RUN wget x; curl y && tar xf a.tar\nONBUILD RUN apk add git && gem install rails\nONBUILD RUN echo $a && su root && cd /tmp && tar xf a.tar\n",
            "2:DL3004 2:DL3018 2:DL4006 2:DL3019 2:SC2117 3:DL3004 5:DL3045 6:DL4001 6:DL3047 7:DL3018 7:DL3019 7:DL3028 8:SC2117",
        );
    }

    #[test]
    fn oracle_17_findings() {
        check(
            "FROM ubuntu\nRUN set -e; cat file | wc && cd /tmp && su root && rm -rf / && apk update && apt-get dist-upgrade && apt install curl && sudo pip install y\nONBUILD RUN gem install rails && chmod 777 /x && cat file | wc && cat file | wc && apt-get install a b c && dnf install q && cd /tmp\nRUN mkdir x && sudo pip install y && wget x; curl y && useradd -m u\n",
            "1:DL3006 2:DL3003 2:DL3004 2:DL4006 2:DL3027 2:SC2114 3:DL3040 3:SC2164 3:DL3038 3:DL3041 3:DL3028 3:DL3008 3:DL3015 3:DL3014 4:DL4001 4:DL3047 4:DL3004",
        );
    }

    #[test]
    fn oracle_22_findings() {
        check(
            "FROM ubuntu:latest\nRUN apt-get dist-upgrade && apt-get dist-upgrade && wget x; curl y && zypper install p && cat file | wc\nRUN pip install foo && yarn add left-pad && yarn add left-pad && cd /x && mkdir x && zypper install p && rm -rf / && useradd -m u\nRUN rm -rf /\nEXPOSE 80 80\nRUN pip3 install --upgrade pip && gem install rails && su root\nRUN set -e; apt-get dist-upgrade && cd /tmp && cd /x\nRUN wget http://x/y\n",
            "1:DL3007 2:DL3034 2:DL3047 2:DL3037 2:DL4001 2:DL3036 2:DL4006 3:DL3042 3:DL3013 3:DL3036 3:DL3003 3:DL3037 3:SC2114 3:DL3034 4:SC2114 6:DL3013 6:SC2117 6:DL3028 6:DL3042 7:DL3003 8:DL3047 8:DL4001",
        );
    }

    #[test]
    fn oracle_27_findings() {
        check(
            "RUN apk update && gem install rails && chmod 777 /x && apt-get dist-upgrade\nRUN set -e; echo $a && yum install -y bar && pip install foo && zypper install p && cd /tmp && gem install rails && yarn add left-pad\nRUN set -e; yum install -y bar && apt-get dist-upgrade && dnf install q && pip3 install --upgrade pip && wget x; curl y && cat file | wc && apk add git\nLABEL a=b\n",
            "1:DL3061 1:DL3028 2:DL3013 2:DL3034 2:DL3037 2:DL3028 2:DL3061 2:DL3036 2:DL3042 2:DL3032 2:DL3003 2:DL3033 2:SC2086 3:DL3033 3:DL4006 3:DL3032 3:DL3047 3:DL3019 3:DL3018 3:DL3038 3:DL3013 3:DL3042 3:DL3061 3:DL3041 3:DL4001 3:DL3040 4:DL3061",
        );
    }

    #[test]
    fn oracle_31_findings() {
        check(
            "FROM debian:9\nRUN apt-get install a b c && gem install rails && tar xf a.tar && zypper install p && dnf install q && wget http://x/y && yum install -y bar && rm -rf /\nONBUILD RUN yarn add left-pad && useradd -m u && zypper install p && npm i z && pip3 install --upgrade pip\nRUN chmod 777 /x && wget http://x/y && apt-get install a b c && wget x; curl y\nFROM ubuntu\nRUN set -e; yum install -y bar && apt-get dist-upgrade && curl http://x | sh && apk add git && wget http://x/y\nHEALTHCHECK CMD a\n",
            "2:SC2114 2:DL3008 2:DL3014 2:DL3041 2:DL3028 2:DL3033 2:DL3015 2:DL3047 2:DL3037 2:DL3034 2:DL3036 2:DL3038 3:DL3013 3:DL3037 3:DL3042 3:DL3036 3:DL3034 4:DL3047 4:DL3015 4:DL4001 4:DL3008 4:DL3014 5:DL3006 6:DL3019 6:DL4001 6:DL3032 6:DL4006 6:DL3018 6:DL3047 6:DL3033 7:DL3025",
        );
    }

    #[test]
    fn oracle_36_findings() {
        check(
            "FROM debian:9\nONBUILD RUN apt-get install a b c && dnf install q && ls | grep x && dnf install q\nRUN cd /tmp && apt-get install a b c && yarn add left-pad\nONBUILD RUN tar xf a.tar && sudo apt-get install -y q && rm -rf / && chmod 777 /x\nRUN set -e; apt-get dist-upgrade && ls | grep x && apt-get update && apt-get install a b c && apk update && sudo pip install y\nRUN set -e; wget http://x/y && gem install rails && yum install -y bar && apk add git && echo $a && dnf install q\nRUN set -e; pip install foo && apk add git && ls | grep x\n",
            "2:SC2010 2:DL3038 2:DL3008 2:DL3041 2:DL3015 2:DL3040 2:DL3014 3:DL3003 3:DL3008 3:DL3014 3:DL3015 4:SC2114 4:DL3004 5:DL3015 5:DL3004 5:DL3009 5:SC2010 5:DL3014 5:DL4006 5:DL3008 6:DL3041 6:DL3032 6:DL3040 6:DL3018 6:SC2086 6:DL3038 6:DL3047 6:DL3019 6:DL3033 6:DL3028 7:DL3018 7:DL3019 7:SC2010 7:DL3013 7:DL3042 7:DL4006",
        );
    }

    #[test]
    fn oracle_39_findings() {
        check(
            "RUN cd /x && yum install -y bar && useradd -m u && wget http://x/y && mkdir x && sudo pip install y\nFROM ubuntu\nRUN curl http://x | sh && mkdir x && su root && sudo pip install y && ls | grep x && apk add git && mkdir x && yum install -y bar && apt-get install a b c\nFROM debian AS x\nRUN mkdir x && pip install foo && zypper install p && pip install foo && useradd -m u && apt-get install a b c && cat file | wc && cd /tmp\nRUN wget x; curl y && mkdir x && sudo pip install y && apk add git && mkdir x && apt-get update && yum install -y bar && ls | grep x && cat file | wc\n",
            "1:DL3003 1:DL3004 1:DL3032 1:DL3061 1:DL3033 1:DL3047 2:DL3006 3:DL3033 3:DL3015 3:DL3004 3:DL3008 3:SC2010 3:DL3032 3:DL4006 3:DL3014 3:DL3019 3:DL3018 4:DL3006 5:DL3015 5:DL3008 5:DL3003 5:DL3014 5:DL4006 5:DL3042 5:SC2164 5:DL3013 5:DL3034 5:DL3037 5:DL3036 6:DL3047 6:DL4006 6:DL3033 6:SC2010 6:DL3004 6:DL4001 6:DL3009 6:DL3019 6:DL3032 6:DL3018",
        );
    }

    #[test]
    fn oracle_41_findings() {
        check(
            "ENV a b\nRUN set -e; zypper install p && wget http://x/y && sudo pip install y && sudo apt-get install -y q && apt-get install a b c && yum install -y bar && apk update && yarn add left-pad && pip3 install --upgrade pip\nRUN apt-get dist-upgrade && useradd -m u && rm -rf / && tar xf a.tar && zypper install p && pip3 install --upgrade pip && apk update && npm i z && echo $a\nRUN npm i z && wget x; curl y && dnf install q && echo $a\nRUN wget x; curl y && dnf install q\nRUN set -e; ls | grep x && tar xf a.tar\nONBUILD RUN zypper install p && apt-get dist-upgrade && gem install rails\n",
            "1:DL3061 2:DL3004 2:DL3014 2:DL3008 2:DL3042 2:DL3013 2:DL3036 2:DL3061 2:DL3015 2:DL3037 2:DL3032 2:DL3047 2:DL3034 2:DL3033 3:DL3037 3:DL3034 3:DL3042 3:DL3061 3:DL3036 3:SC2114 3:DL3013 4:DL3041 4:DL4001 4:DL3038 4:DL3061 4:DL3047 4:DL3040 5:DL3061 5:DL3040 5:DL4001 5:DL3038 5:DL3041 5:DL3047 6:DL3061 6:DL4006 6:SC2010 7:DL3036 7:DL3061 7:DL3028 7:DL3037 7:DL3034",
        );
    }
}
