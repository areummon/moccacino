use std::collections::BTreeSet;

use super::{finite, words};
use crate::regex::{self, RegexAst};
use crate::state_machine::{Machine, StateMachine};

fn endpoints(node: &RegexAst, text: &[char], start: usize) -> BTreeSet<usize> {
    let single = |p: usize| BTreeSet::from([p]);
    match node {
        RegexAst::Empty => BTreeSet::new(),
        RegexAst::Epsilon => single(start),
        RegexAst::Char(c) => {
            if start < text.len() && text[start] == *c {
                single(start + 1)
            } else {
                BTreeSet::new()
            }
        },
        RegexAst::Concat(left, right) => endpoints(left, text, start)
            .iter()
            .flat_map(|&p| endpoints(right, text, p))
            .collect(),
        RegexAst::Union(left, right) => {
            let mut set = endpoints(left, text, start);
            set.extend(endpoints(right, text, start));
            set
        },
        RegexAst::Star(inner) => {
            let mut result = single(start);
            let mut frontier = endpoints(inner, text, start);
            result.extend(frontier.iter().copied());
            loop {
                frontier = frontier
                    .iter()
                    .flat_map(|&p| endpoints(inner, text, p))
                    .filter(|p| !result.contains(p))
                    .collect();
                if frontier.is_empty() {
                    break;
                }
                result.extend(frontier.iter().copied());
            }
            result
        },
        RegexAst::Plus(inner) => {
            let once = endpoints(inner, text, start);
            let star = RegexAst::Star(inner.clone());
            let mut result = BTreeSet::new();
            for &p in once.iter() {
                result.extend(endpoints(&star, text, p));
            }
            result
        },
        RegexAst::Quest(inner) => {
            let mut set = single(start);
            set.extend(endpoints(inner, text, start));
            set
        },
    }
}

fn naive_accepts(ast: &RegexAst, text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    endpoints(ast, &chars, 0).contains(&chars.len())
}

#[test]
fn regex_parse_valid_test() {
    for pattern in [
        "", "a", "ε", "ab", "a*", "(a)", "(a|b)*abb", "a|b|c", "a+", "a?",
        "\\*", "\\\\", "\\(a\\)", "(a|)+", "|a", "()", "a**", "a*?", "abc",
        "((a))", "ε*",
    ] {
        assert!(regex::parse(pattern).is_ok(), "pattern {pattern:?} should parse");
    }
}

#[test]
fn regex_parse_errors_test() {
    for pattern in ["(", ")", "*a", "+", "?", "a**b)c", "a\\", "(ab"] {
        let error = regex::parse(pattern).expect_err("pattern should fail");
        assert!(
            error.position <= pattern.chars().count(),
            "error position out of range for {pattern:?}"
        );
    }
    let error: regex::ParseError = regex::parse("(").unwrap_err();
    assert_eq!(error.position, 1);
    assert_eq!(error.to_string(), "expected ')' to close the group (at position 1)");
    assert_eq!(regex::parse("*a").unwrap_err().position, 0);
    assert_eq!(regex::parse("a\\").unwrap_err().position, 2);
}

#[test]
fn regex_parse_precedence_test() {
    let ast = regex::parse("ab|cd").unwrap();
    assert!(naive_accepts(&ast, "ab"));
    assert!(naive_accepts(&ast, "cd"));
    assert!(!naive_accepts(&ast, "ad"));
    assert!(!naive_accepts(&ast, "cb"));
    assert!(!naive_accepts(&ast, "abcd"));

    let ast = regex::parse("ab*").unwrap();
    assert!(naive_accepts(&ast, "a"));
    assert!(naive_accepts(&ast, "abbb"));
    assert!(!naive_accepts(&ast, "abab"));

    let ast = regex::parse("a|bc").unwrap();
    assert!(naive_accepts(&ast, "bc"));
    assert!(!naive_accepts(&ast, "ac"));

    let ast = regex::parse("\\*|a\\+").unwrap();
    assert!(naive_accepts(&ast, "*"));
    assert!(naive_accepts(&ast, "a+"));
    assert!(!naive_accepts(&ast, "aa"));
}

#[test]
fn regex_compile_agrees_with_naive_matcher_test() {
    let cases: Vec<(&str, &[char])> = vec![
        ("(a|b)*abb", &['a', 'b']),
        ("a*b*", &['a', 'b']),
        ("(ab)+", &['a', 'b']),
        ("(0|1)*01", &['0', '1']),
        ("a?b*a?", &['a', 'b']),
        ("(a|ε)b", &['a', 'b']),
        ("(((a)))", &['a']),
        ("a**", &['a']),
        ("ε", &['a', 'b']),
        ("", &['a', 'b']),
    ];
    for (pattern, alphabet) in cases {
        let ast = regex::parse(pattern).unwrap();
        let automata = regex::compile_str(pattern).unwrap();
        assert!(
            Machine::validate(&automata).is_ok(),
            "compiled automaton for {pattern:?} must be valid"
        );
        for input in words(alphabet, 6) {
            let expected = naive_accepts(&ast, &input);
            let actual = automata.accepts(&input);
            assert_eq!(
                actual, expected,
                "mismatch on pattern {pattern:?} with input {input:?}"
            );
        }
    }
}

#[test]
fn regex_pipeline_minimize_test() {
    let automata = regex::compile_str("(a|b)*abb").unwrap();
    assert!(!automata.is_deterministic());
    let deterministic_automata = automata.to_dfa();
    assert!(deterministic_automata.is_deterministic());
    let minimized = deterministic_automata.minimize();
    assert_eq!(minimized.get_states_by_id_ref().len(), 4);

    for (input, expected) in [
        ("abb", true), ("aabb", true), ("aaabbbabb", true), ("abababb", true),
        ("", false), ("ab", false), ("abba", false), ("babbb", false),
    ] {
        assert_eq!(
            minimized.accepts(input),
            expected,
            "mismatch on minimized pipeline with input {input:?}"
        );
    }

    let automata = regex::compile_str("(ab)*").unwrap();
    let minimized = automata.to_dfa().minimize();
    assert_eq!(minimized.get_states_by_id_ref().len(), 2);
    for (input, expected) in [("", true), ("ab", true), ("abab", true), ("aba", false), ("b", false)] {
        assert_eq!(minimized.accepts(input), expected, "input {input:?}");
    }
}

#[test]
fn regex_empty_language_and_epsilon_test() {
    let automata = regex::compile_str("ε").unwrap();
    assert!(automata.accepts(""));
    assert!(!automata.accepts("a"));

    let automata = regex::compile(&RegexAst::Empty);
    assert!(!automata.accepts(""));
    assert!(!automata.accepts("anything"));

    let automata = regex::compile_str("").unwrap();
    assert!(automata.accepts(""));
}

#[test]
fn regex_display_precedence_test() {
    let cases: Vec<(RegexAst, &str)> = vec![
        (RegexAst::Char('a'), "a"),
        (RegexAst::Epsilon, "ε"),
        (RegexAst::Char('*'), "\\*"),
        (RegexAst::Char('|'), "\\|"),
        (RegexAst::Char('\\'), "\\\\"),
        (
            RegexAst::Union(
                Box::new(RegexAst::Concat(
                    Box::new(RegexAst::Char('a')),
                    Box::new(RegexAst::Char('b')),
                )),
                Box::new(RegexAst::Char('c')),
            ),
            "ab|c",
        ),
        (
            RegexAst::Concat(
                Box::new(RegexAst::Union(
                    Box::new(RegexAst::Char('a')),
                    Box::new(RegexAst::Char('b')),
                )),
                Box::new(RegexAst::Char('c')),
            ),
            "(a|b)c",
        ),
        (
            RegexAst::Star(Box::new(RegexAst::Concat(
                Box::new(RegexAst::Char('a')),
                Box::new(RegexAst::Char('b')),
            ))),
            "(ab)*",
        ),
        (
            RegexAst::Star(Box::new(RegexAst::Star(Box::new(RegexAst::Char('a'))))),
            "(a*)*",
        ),
        (RegexAst::Star(Box::new(RegexAst::Epsilon)), "ε*"),
        (
            RegexAst::Plus(Box::new(RegexAst::Char('a'))),
            "a+",
        ),
        (
            RegexAst::Union(
                Box::new(RegexAst::Char('a')),
                Box::new(RegexAst::Union(
                    Box::new(RegexAst::Char('b')),
                    Box::new(RegexAst::Char('c')),
                )),
            ),
            "a|b|c",
        ),
    ];
    for (ast, expected) in cases {
        assert_eq!(format!("{ast}"), expected, "rendering {:?}", ast);
    }
}

fn build_abb_nfa() -> crate::finite_automata::FiniteAutomata {
    finite(11, &[10], &[
        (0, 1, "ε"), (0, 7, "ε"), (1, 2, "a"), (2, 3, "ε"), (3, 4, "b"), (4, 5, "ε"),
        (5, 1, "ε"), (5, 6, "ε"), (6, 7, "ε"), (7, 8, "a"), (8, 9, "b"), (9, 10, "b"),
    ])
}

#[test]
fn fa_to_regex_language_agreement_test() {
    use crate::finite_automata::FiniteAutomata;

    let odd = finite(3, &[1, 2], &[(0, 1, "a"), (0, 0, "b"), (1, 0, "a"), (1, 1, "b"), (2, 2, "a")]);
    let abb = build_abb_nfa();
    let reject_all = finite(1, &[], &[(0, 0, "a")]);
    let accept_all = finite(1, &[0], &[(0, 0, "a")]);

    let fixtures: Vec<&FiniteAutomata> = vec![&odd, &abb, &reject_all, &accept_all];
    for machine in &fixtures {
        let ast = machine.to_regex();
        let rendered = format!("{ast}");
        let compiled = regex::compile_str(&rendered)
            .unwrap_or_else(|error| panic!("eliminated regex {rendered:?} failed to parse: {error}"));
        for input in words(&['a', 'b'], 6) {
            assert_eq!(
                compiled.accepts(&input),
                machine.accepts(&input),
                "state elimination changed the language on {input:?} (regex {rendered:?})"
            );
        }
    }

    assert_eq!(reject_all.to_regex().to_string(), "\u{2205}");
}

#[test]
fn fa_to_regex_empty_machine_test() {
    use crate::finite_automata::FiniteAutomata;
    let machine = FiniteAutomata::new();
    assert!(matches!(machine.to_regex(), RegexAst::Empty));
}

#[test]
fn regex_literal_epsilon_is_not_compiled_test() {
    let error = regex::parse("a\\ε").expect_err("a literal ε is rejected while parsing");
    assert_eq!(error.position, 1);
    assert_eq!(regex::compile_str("a\\ε").unwrap_err(), error);
    assert!(regex::compile_str("aε").is_ok());
}

#[test]
fn fa_to_regex_drops_epsilon_padding_test() {
    let single_b = finite(2, &[1], &[(0, 1, "b")]);
    assert_eq!(single_b.to_regex().to_string(), "b");

    let ends_with_b = finite(2, &[1], &[(0, 0, "a"), (0, 0, "b"), (0, 1, "b")]);
    assert_eq!(ends_with_b.to_regex().to_string(), "(a|b)*b");
}

#[test]
fn fa_to_regex_trivial_languages_test() {
    let no_finals = finite(2, &[], &[(0, 1, "a"), (1, 0, "b")]);
    assert_eq!(no_finals.to_regex().to_string(), "\u{2205}");

    let only_epsilon = finite(1, &[0], &[]);
    assert_eq!(only_epsilon.to_regex().to_string(), "ε");
}

#[test]
fn fa_to_regex_simplified_language_agreement_test() {
    let fixtures = [
        finite(2, &[1], &[(0, 1, "b")]),
        finite(2, &[0], &[(0, 1, "a"), (0, 0, "b"), (1, 0, "a"), (1, 1, "b")]),
        finite(2, &[1], &[(0, 0, "a"), (0, 0, "b"), (0, 1, "b")]),
        finite(3, &[2], &[(0, 1, "ε"), (1, 1, "a"), (1, 2, "ε"), (2, 2, "b"), (0, 2, "ab")]),
        finite(3, &[0, 2], &[(0, 1, "a"), (1, 0, "ε"), (1, 2, "b"), (2, 2, "ε"), (2, 0, "a")]),
        finite(1, &[0], &[(0, 0, "a"), (0, 0, "ε")]),
    ];
    for machine in &fixtures {
        let rendered = machine.to_regex().to_string();
        let compiled = regex::compile_str(&rendered)
            .unwrap_or_else(|error| panic!("regex {rendered:?} failed to parse: {error}"));
        for input in words(&['a', 'b'], 6) {
            assert_eq!(
                compiled.accepts(&input),
                machine.accepts(&input),
                "simplified regex {rendered:?} disagrees with its automaton on {input:?}"
            );
        }
    }
}

#[test]
fn regex_smart_constructors_preserve_language_test() {
    let raw_star = |inner: &RegexAst| RegexAst::Star(Box::new(inner.clone()));
    let operands = [
        RegexAst::Empty,
        RegexAst::Epsilon,
        RegexAst::Char('a'),
        RegexAst::Char('b'),
        raw_star(&RegexAst::Char('a')),
        raw_star(&raw_star(&RegexAst::Char('b'))),
        raw_star(&RegexAst::Empty),
        raw_star(&RegexAst::Epsilon),
        RegexAst::Union(Box::new(RegexAst::Epsilon), Box::new(RegexAst::Char('a'))),
    ];
    let inputs = words(&['a', 'b'], 4);
    let agree = |simplified: &RegexAst, raw: &RegexAst| {
        for word in &inputs {
            assert_eq!(
                naive_accepts(simplified, word),
                naive_accepts(raw, word),
                "{simplified:?} and {raw:?} disagree on {word:?}"
            );
        }
    };
    for left in &operands {
        agree(&RegexAst::star(left.clone()), &raw_star(left));
        for right in &operands {
            let raw_concat = RegexAst::Concat(Box::new(left.clone()), Box::new(right.clone()));
            let raw_union = RegexAst::Union(Box::new(left.clone()), Box::new(right.clone()));
            agree(&RegexAst::concat(left.clone(), right.clone()), &raw_concat);
            agree(&RegexAst::union(left.clone(), right.clone()), &raw_union);
        }
    }
}
