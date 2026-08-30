use std::collections::HashSet;

use crate::grammar::{self, Grammar, ParseError};
use crate::state_machine::{Machine, StateMachine};

/* ---------- Fixtures ---------- */

const BALANCED_PARENS: &str = "S -> a S b\nS -> S S\nS -> ε";
const ANBN: &str = "S -> a S b | ε";
const UNIT_CHAINS: &str = "S -> T | c\nT -> U\nU -> a b a";
const USELESS_DECORATION: &str = "S -> T | c\nT -> U\nU -> a b a\nV -> V W\nW -> x";

/* Independent reference matcher: exhaustive breadth-first expansion that
 * always expands the first variable of every sentential form (complete for
 * CFG derivability: derivations can be reordered so expanding leftmost
 * variables still yields any derivable word). Iterative on purpose: S -> SS
 * style grammars recurse too deeply otherwise. */
fn naive_accepts(grammar: &Grammar, input: &str, max_steps: usize) -> bool {
    use std::collections::VecDeque;
    let mut queue: VecDeque<Vec<String>> = VecDeque::from([vec![
        grammar.start_symbol().to_string(),
    ]]);
    let mut visited: HashSet<Vec<String>> = HashSet::new();
    let mut steps_left = max_steps;

    while let Some(form) = queue.pop_front() {
        let is_terminal_form = form
            .iter()
            .all(|symbol| !grammar.nonterminals().contains(symbol));
        if is_terminal_form {
            if form.concat() == input {
                return true;
            }
            continue;
        }
        let position = match form
            .iter()
            .position(|symbol| grammar.nonterminals().contains(symbol))
        {
            Some(found) => found,
            None => continue,
        };
        let variable = form[position].clone();
        let bodies: Vec<Vec<String>> = grammar
            .productions_of(&variable)
            .cloned()
            .unwrap_or_default();
        for body in bodies {
            if steps_left == 0 {
                return false;
            }
            steps_left -= 1;
            let mut next = form.clone();
            next.splice(position..position + 1, body.iter().cloned());
            if !visited.insert(next.clone()) {
                continue;
            }
            if next.iter().all(|symbol| !grammar.nonterminals().contains(symbol)) {
                if next.concat() == input {
                    return true;
                }
                continue;
            }
            queue.push_back(next);
        }
    }
    false
}

/* All strings over `alphabet` up to `max_len`, including the empty one. */
fn alphabet_inputs(alphabet: &[char], max_len: usize) -> Vec<String> {
    let mut inputs = vec![String::new()];
    for _ in 0..max_len {
        let mut next = Vec::new();
        for input in &inputs {
            for c in alphabet {
                let mut longer = input.clone();
                longer.push(*c);
                next.push(longer);
            }
        }
        inputs.extend(next.drain(..));
    }
    inputs.retain(|input| input.chars().count() <= max_len);
    inputs.sort();
    inputs.dedup();
    inputs
}

/* One-shot reference enumeration: best-first expansion collecting every
 * terminal sentential form whose length stays within max_len. Amortized over
 * the whole battery instead of per-word, which keeps S->SS grammars fast. */
fn enumerate_accepted(grammar: &Grammar, max_len: usize, max_steps: usize) -> HashSet<String> {
    use std::cmp::Reverse;
    #[derive(Clone)]
    struct Node {
        form: Vec<String>,
    }
    let var_count = |form: &[String]| {
        form.iter()
            .filter(|symbol| grammar.nonterminals().contains(*symbol))
            .count()
    };
    let terminal_length_bound = |form: &[String]| -> usize {
        // Every variable eventually contributes at least one character or
        // nothing (nullable); terminals contribute their exact length.
        form.iter()
            .map(|symbol| {
                if grammar.nonterminals().contains(symbol) {
                    0
                } else {
                    symbol.chars().count()
                }
            })
            .sum()
    };
    let total_symbol_bound = |form: &[String]| -> usize { form.len() };

    let mut nodes: Vec<Node> = vec![Node {
        form: vec![grammar.start_symbol().to_string()],
    }];
    let mut seen: HashSet<Vec<String>> = HashSet::new();
    seen.insert(nodes[0].form.clone());
    let mut heap: std::collections::BinaryHeap<Reverse<(usize, usize)>> =
        std::collections::BinaryHeap::from([Reverse((1, 0))]);
    let mut accepted: HashSet<String> = HashSet::new();
    let mut steps_left = max_steps;

    while let Some(Reverse((_, index))) = heap.pop() {
        let form = nodes[index].form.clone();
        if var_count(&form) == 0 {
            if concatenate(&form).chars().count() <= max_len {
                accepted.insert(concatenate(&form));
            }
            continue;
        }
        let position = form
            .iter()
            .position(|symbol| grammar.nonterminals().contains(symbol))
            .expect("non-terminal form holds a variable");
        let variable = form[position].clone();
        let bodies: Vec<Vec<String>> = grammar
            .productions_of(&variable)
            .cloned()
            .unwrap_or_default();
        for body in bodies {
            if steps_left == 0 {
                return accepted;
            }
            steps_left -= 1;
            let mut next = form.clone();
            next.splice(position..position + 1, body.iter().cloned());
            if terminal_length_bound(&next) > max_len
                || total_symbol_bound(&next) > 2 * max_len + 2
            {
                continue;
            }
            if !seen.insert(next.clone()) {
                continue;
            }
            nodes.push(Node { form: next });
            heap.push(Reverse((var_count(&nodes.last().unwrap().form), nodes.len() - 1)));
        }
    }
    accepted
}

/* Validates a derivation chain produced by derive_leftmost/derive_rightmost:
 * starts at [start], each step replaces exactly one variable occurrence at
 * the extreme position demanded by the mode, and ends with the input. */
fn validate_chain(
    grammar: &Grammar,
    chain: &[Vec<String>],
    input: &str,
    leftmost: bool,
) -> Result<(), String> {
    if chain.is_empty() {
        return Err("empty chain".to_string());
    }
    if chain[0] != vec![grammar.start_symbol().to_string()] {
        return Err(format!("chain does not start with [{:?}]", grammar.start_symbol()));
    }
    if concatenate(chain.last().expect("non-empty")) != input {
        return Err("chain does not end with the input".to_string());
    }
    for pair in chain.windows(2) {
        let before = &pair[0];
        let after = &pair[1];
        let range: Box<dyn Iterator<Item = usize>> = if leftmost {
            Box::new(0..before.len())
        } else {
            Box::new((0..before.len()).rev())
        };
        let mut replaced_any = false;
        for index in range {
            if grammar.nonterminals().contains(&before[index]) {
                let variable = before[index].clone();
                let bodies = grammar.productions_of(&variable).cloned().unwrap_or_default();
                let mut valid_replacement = false;
                for body in bodies {
                    let candidate = replace_at(before, index, &body);
                    if candidate == *after {
                        valid_replacement = true;
                        break;
                    }
                }
                if !valid_replacement {
                    return Err(format!(
                        "step {:?} -> {:?} is not a valid expansion of {}",
                        before, after, variable
                    ));
                }
                replaced_any = true;
                break;
            }
        }
        if !replaced_any {
            return Err(format!(
                "step {:?} -> {:?} expands no extreme variable",
                before, after
            ));
        }
    }
    Ok(())
}

fn replace_at(form: &[String], index: usize, body: &[String]) -> Vec<String> {
    let mut next = form.to_vec();
    next.splice(index..index + 1, body.iter().cloned());
    next
}

fn concatenate(symbols: &[String]) -> String {
    symbols.concat()
}

/* CNF shape invariant: every body is one terminal, two variables, or ε on the
 * start symbol. */
fn assert_cnf_shape(grammar: &Grammar) {
    for (variable, body) in grammar.productions() {
        let ok = (body.len() == 1 && grammar.nonterminals().contains(&body[0]) == false)
            || (body.len() == 2 && body.iter().all(|s| grammar.nonterminals().contains(s)))
            || (body.is_empty() && variable == grammar.start_symbol());
        assert!(
            ok,
            "body {:?} of {} violates Chomsky normal form",
            body, variable
        );
    }
}

/* ---------- Parser ---------- */

#[test]
fn grammar_parse_basic_test() {
    let grammar = grammar::parse_grammar(BALANCED_PARENS).unwrap();
    assert_eq!(grammar.start_symbol(), "S");
    assert_eq!(grammar.nonterminals().len(), 1);
    let terminals = grammar.terminals();
    assert!(terminals.contains("a") && terminals.contains("b"));
    assert_eq!(terminals.len(), 2);
    assert_eq!(grammar.productions_of("S").unwrap().len(), 3);
    assert!(grammar.contains_epsilon());

    // Production groups may span multiple lines.
    let grammar = grammar::parse_grammar("S -> a\nS -> b\nA -> a").unwrap();
    assert_eq!(grammar.start_symbol(), "S");
    assert_eq!(grammar.nonterminals().len(), 2);
    assert_eq!(grammar.productions_of("S").unwrap().len(), 2);
}

#[test]
fn grammar_parse_errors_test() {
    for (source, expected_line) in [
        ("", 0usize),
        ("S a b", 1),
        ("S -> a\nbroken line", 2),
        ("S T -> a", 1),
        ("-> a", 1),
    ] {
        let error: ParseError =
            grammar::parse_grammar(source).expect_err("should fail to parse");
        assert_eq!(error.line, expected_line, "case {:?}", source);
        assert!(!error.message.is_empty());
    }

    // The Display impl renders line information.
    let error = grammar::parse_grammar("broken").unwrap_err();
    assert_eq!(error.to_string(), format!("{} (line {})", error.message, error.line));
}

#[test]
fn grammar_lenient_empty_alternatives_test() {
    // A blank alternative means the empty body exactly like ε.
    let trailing = grammar::parse_grammar("S -> a | b | ").unwrap();
    assert!(trailing.contains_epsilon());
    assert_eq!(trailing.productions_of("S").unwrap().len(), 3);
    assert!(trailing.generate(""));
    assert!(trailing.generate("a"));
    assert!(!trailing.generate("c"));

    let leading = grammar::parse_grammar("S -> | a").unwrap();
    assert!(leading.contains_epsilon());
    assert!(leading.generate(""));
    assert!(leading.generate("a"));
    assert!(!leading.generate("aa"));

    let doubled = grammar::parse_grammar("S -> a | | b").unwrap();
    assert!(doubled.contains_epsilon());
    assert!(doubled.generate("b"));

    let whole_rhs_blank = grammar::parse_grammar("T -> \nT -> x").unwrap();
    assert_eq!(whole_rhs_blank.start_symbol(), "T");
    assert!(whole_rhs_blank.contains_epsilon());
    assert!(whole_rhs_blank.generate("x"));
    let whole_rhs_tab: String = "T ->\t".to_string();
    assert!(grammar::parse_grammar(&whole_rhs_tab).unwrap().contains_epsilon());

    // Whitespace between symbols stays a separator, not an epsilon.
    let multi = grammar::parse_grammar("S -> a b c").unwrap();
    assert!(multi.generate("abc"));
    assert!(!multi.generate("ac"));

    // The explicit-spelling and lenient sources render identically through
    // Display and therefore reparse identically.
    let explicit = grammar::parse_grammar("S -> a | ε").unwrap();
    let lenient = grammar::parse_grammar("S -> a | ").unwrap();
    assert_eq!(format!("{}", explicit), format!("{}", lenient));
}

/* ---------- Derivations ---------- */

#[test]
fn grammar_derivation_chain_validity_test() {
    let grammar = grammar::parse_grammar(BALANCED_PARENS).unwrap();
    // Words stay small: this test validates chain structure, not search
    // scalability on wide grammars (the membership battery covers agreement).
    for input in ["", "ab", "aabb", "abab", "aaabbb"] {
        let chain = grammar
            .derive_leftmost(input, 200_000)
            .unwrap_or_else(|| panic!("no leftmost derivation for {:?}", input));
        validate_chain(&grammar, &chain, input, true)
            .unwrap_or_else(|reason| panic!("bad leftmost chain for {:?}: {}", input, reason));

        let chain = grammar
            .derive_rightmost(input, 200_000)
            .unwrap_or_else(|| panic!("no rightmost derivation for {:?}", input));
        validate_chain(&grammar, &chain, input, false)
            .unwrap_or_else(|reason| panic!("bad rightmost chain for {:?}: {}", input, reason));
    }
}

#[test]
fn grammar_derivation_extreme_position_test() {
    // The leftmost derivation always expands the FIRST variable of the
    // current form, which distinguishes it from the rightmost one.
    let grammar = grammar::parse_grammar(BALANCED_PARENS).unwrap();
    let chain = grammar.derive_leftmost("abab", 1000).unwrap();
    assert!(chain.len() >= 2);
    // First step must expand position 0.
    assert_eq!(chain[0], vec!["S".to_string()]);
    assert_ne!(chain[1], vec!["S".to_string()]);
}

#[test]
fn grammar_derivation_negative_and_budget_test() {
    let grammar = grammar::parse_grammar(BALANCED_PARENS).unwrap();
    // Not in the language at all.
    assert_eq!(grammar.derive_leftmost("aab", 10_000), None);
    assert_eq!(grammar.derive_rightmost("a", 10_000), None);

    // A starved budget fails even on derivable words...
    let tight = 2usize;
    assert_eq!(grammar.derive_leftmost("aabb", tight), None);
    // ...while the same word succeeds with room to spare.
    assert!(grammar.derive_leftmost("aabb", 10_000).is_some());
}

/* ---------- Membership (CYK vs naive reference vs derivations) ---------- */

#[test]
fn grammar_membership_agreement_test() {
    let sources = [BALANCED_PARENS, ANBN, UNIT_CHAINS];
    for source in sources {
        let grammar = grammar::parse_grammar(source).unwrap();
        // One enumeration pass backs the whole battery for this grammar.
        let accepted = enumerate_accepted(&grammar, 6, 500_000);
        for input in alphabet_inputs(&['a', 'b'], 6) {
            let expected = accepted.contains(&input);
            let cyk_result = grammar.generate(&input);
            assert_eq!(
                cyk_result, expected,
                "CYK and the reference enumeration disagree on {:?} for grammar:\n{}",
                input, source
            );
        }
    }
}

#[test]
fn grammar_derivation_membership_agreement_test() {
    // Exhaustive derivation search over S->SS style grammars explodes into
    // distinct ordered sentential forms on rejected words, so the symmetric
    // membership battery uses grammars whose reachable form space stays
    // tiny; language-level agreement for the wide ones is covered by the
    // enumeration battery, and chain structure by the validity test.
    let sources = [ANBN, UNIT_CHAINS];
    for source in sources {
        let grammar = grammar::parse_grammar(source).unwrap();
        let accepted_short = enumerate_accepted(&grammar, 4, 100_000);
        for input in alphabet_inputs(&['a', 'b'], 4) {
            assert_eq!(
                grammar.derive_leftmost(&input, 50_000).is_some(),
                accepted_short.contains(&input),
                "derivation search disagrees on {:?} for:\n{}",
                input, source
            );
        }
    }
}

#[test]
fn grammar_generate_tokens_multichar_terminal_test() {
    // Explicit tokens allow multi-character terminals that plain-string
    // generate() could not segment unambiguously.
    let mut grammar = Grammar::new("S");
    grammar.add_production("S", &["aa", "B"]);
    grammar.add_production("S", &["c"]);
    grammar.add_production("B", &["bb"]);

    assert!(grammar.generate_tokens(&["aa".to_string(), "bb".to_string()]));
    assert!(grammar.generate_tokens(&["c".to_string()]));
    assert!(!grammar.generate_tokens(&["a".to_string(), "bb".to_string()]));
}

/* ---------- Chomsky normal form ---------- */

#[test]
fn grammar_cnf_shape_invariants_test() {
    let sources = [BALANCED_PARENS, ANBN, UNIT_CHAINS, USELESS_DECORATION];
    for source in sources {
        let grammar = grammar::parse_grammar(source).unwrap();
        let cnf = grammar.to_chomsky_normal_form();
        // Shape check doubles as the ε-only-on-start invariant: empty bodies
        // are only allowed on the start symbol.
        assert_cnf_shape(&cnf);
    }
}

#[test]
fn grammar_cnf_language_preservation_test() {
    let sources = [BALANCED_PARENS, ANBN, UNIT_CHAINS, USELESS_DECORATION];
    for source in sources {
        let grammar = grammar::parse_grammar(source).unwrap();
        let cnf = grammar.to_chomsky_normal_form();
        let accepted = enumerate_accepted(&grammar, 6, 500_000);
        for input in alphabet_inputs(&['a', 'b'], 6) {
            assert_eq!(
                cnf.generate(&input),
                accepted.contains(&input),
                "CNF changed the language on {:?} for:\n{}",
                input, source
            );
        }
        // ε membership agrees too.
        assert_eq!(cnf.generate(""), grammar.contains_epsilon());
    }
}

#[test]
fn grammar_cnf_useless_symbol_removal_test() {
    let grammar = grammar::parse_grammar(USELESS_DECORATION).unwrap();
    let cnf = grammar.to_chomsky_normal_form();
    // V and W are non-generating/unreachable junk and must not survive;
    // S/T/U's chain content remains (modulo helpers).
    assert!(!cnf.nonterminals().contains("V"));
    assert!(!cnf.nonterminals().contains("W"));
    assert!(naive_accepts(&grammar, "aba", 20_000));
    // The surviving language still includes the junk-free derivations.
    assert!(cnf.generate("aba"));
    assert!(cnf.generate("c"));
}

#[test]
fn grammar_epsilon_edge_cases_test() {
    // Language { ε } exactly.
    let grammar = grammar::parse_grammar("S -> ε").unwrap();
    assert!(grammar.contains_epsilon());
    assert!(grammar.generate(""));
    assert!(!grammar.generate("a"));
    let cnf = grammar.to_chomsky_normal_form();
    assert_cnf_shape(&cnf);
    assert!(cnf.generate(""));

    // Unit chains carrying ε through several variables.
    let grammar = grammar::parse_grammar("S -> T\nT -> U\nU -> ε").unwrap();
    assert!(grammar.contains_epsilon());
    assert!(grammar.generate(""));
    assert!(!grammar.generate("a"));

    // A pure self-loop generates no finite strings ever.
    let grammar = grammar::parse_grammar("S -> S").unwrap();
    assert!(!grammar.contains_epsilon());
    assert!(!grammar.generate(""));
    assert!(!grammar.generate("a"));
    assert_eq!(grammar.derive_leftmost("", 1000), None);
}


/* ---------- Regular grammar <-> finite automata ---------- */

#[test]
fn grammar_right_linear_to_finite_automata_test() {
    // a*b* is right-linear: S -> a S | T, T -> b T | ε
    let source = "S -> a S | T\nT -> b T | ε";
    let grammar = grammar::parse_grammar(source).unwrap();
    let automata = grammar.to_finite_automata().unwrap();
    let accepted = enumerate_accepted(&grammar, 5, 100_000);
    for input in alphabet_inputs(&['a', 'b'], 5) {
        assert_eq!(
            automata.accepts(&input),
            accepted.contains(&input),
            "right-linear -> FA disagrees on {:?}",
            input
        );
    }

    // Even number of a's: E -> a O | ε, O -> a E | a
    let mut even = Grammar::new("E");
    even.add_production("E", &["a", "O"]);
    even.add_production("E", &[]);
    even.add_production("O", &["a", "E"]);
    even.add_production("O", &["a"]);
    let automata = even.to_finite_automata().unwrap();
    // The generated language lives over {a} alone: anything with other
    // characters must be rejected regardless of parity.
    for input in alphabet_inputs(&['a', 'x'], 6) {
        let only_a = input.chars().all(|c| c == 'a');
        let count = input.chars().filter(|c| *c == 'a').count();
        assert_eq!(
            automata.accepts(&input),
            only_a && count % 2 == 0,
            "even-a grammar converted wrong on {:?}",
            input
        );
    }
}

#[test]
fn grammar_non_right_linear_rejection_test() {
    // Left-linear suffix form.
    let mut left = Grammar::new("S");
    left.add_production("S", &["S", "a"]);
    assert!(left.to_finite_automata().is_err());

    // Unit productions are eliminated before construction these days, so an
    // all-unit grammar converts fine.
    let mut unit = Grammar::new("S");
    unit.add_production("S", &["T"]);
    unit.add_production("T", &["a"]);
    let automata = unit.to_finite_automata().unwrap();
    assert!(automata.accepts("a"));
    assert!(!automata.accepts(""));

    // Three-symbol bodies are still too long.
    let mut long_body = Grammar::new("S");
    long_body.add_production("S", &["a", "S", "a"]);
    assert!(long_body.to_finite_automata().is_err());
}

#[test]
fn grammar_fa_to_right_linear_roundtrip_test() {
    use crate::finite_automata::FiniteAutomata;

    // Odd number of 'a' DFA fixture.
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_initial(0);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    automata.add_transition(0, 0, "b".to_string());
    automata.add_transition(1, 0, "a".to_string());
    automata.add_transition(1, 1, "b".to_string());

    let grammar = Grammar::from_finite_automata(&automata).unwrap();
    assert_eq!(grammar.start_symbol().chars().next(), Some('S'));

    // Roundtrip back into an automaton and compare languages exactly.
    let rebuilt = grammar.to_finite_automata().unwrap();
    for input in alphabet_inputs(&['a', 'b'], 8) {
        let odd_count = input.chars().filter(|c| *c == 'a').count() % 2 == 1;
        assert_eq!(
            automata.accepts(&input),
            odd_count,
            "original DFA broke on {:?}",
            input
        );
        assert_eq!(
            rebuilt.accepts(&input),
            odd_count,
            "roundtripped NFA disagrees on {:?}",
            input
        );
        assert_eq!(
            grammar.generate(&input),
            odd_count,
            "roundtripped grammar disagrees on {:?}",
            input
        );
    }

    // An accepting initial state must become an epsilon production.
    automata.make_final(0);
    let grammar = Grammar::from_finite_automata(&automata).unwrap();
    assert!(grammar.contains_epsilon());
    assert!(grammar.generate(""));

    // Variable names never collide with terminal labels ("S" as a label).
    let mut label_collision = FiniteAutomata::new();
    label_collision.add_n_states(2);
    label_collision.make_initial(0);
    label_collision.make_final(1);
    label_collision.add_transition(0, 1, "A".to_string());
    let grammar = Grammar::from_finite_automata(&label_collision).unwrap();
    let rebuilt = grammar.to_finite_automata().unwrap();
    assert!(rebuilt.accepts("A"));
    assert!(!rebuilt.accepts(""));
}

/* ---------- CFG to pushdown automaton ---------- */

fn pda_matches_enumeration(grammar: &Grammar, max_len: usize) {
    let pda = grammar.to_pushdown_automata().expect("PDA construction");
    let accepted = enumerate_accepted(grammar, max_len, 200_000);
    for input in alphabet_inputs(&['a', 'b'], max_len) {
        let mut owned = input.clone();
        assert_eq!(
            pda.check_input(&mut owned),
            accepted.contains(&input),
            "CFG->PDA disagrees on {:?} (expected {}, got {})",
            input,
            accepted.contains(&input),
            !accepted.contains(&input)
        );
    }
}

#[test]
fn grammar_cnf_free_to_pda_membership_test() {
    // a^n b^n through the recognizer construction: linear grammar, so even
    // rejected words explore a small space.
    let anbn = grammar::parse_grammar(ANBN).unwrap();
    pda_matches_enumeration(&anbn, 6);

    // Balanced parentheses style with the S -> SS nesting rule: rejected
    // words explode combinatorially under any exhaustive search, so the
    // battery keeps to words where the reachable configurations stay small
    // (see PushdownAutomata::check_input bounds).
    let balanced = grammar::parse_grammar(BALANCED_PARENS).unwrap();
    pda_matches_enumeration(&balanced, 4);

    // Unit chains and terminals only.
    let units = grammar::parse_grammar(UNIT_CHAINS).unwrap();
    pda_matches_enumeration(&units, 3);

    // The empty word follows contains_epsilon exactly.
    assert_eq!(anbn.contains_epsilon(), true);
    assert!(anbn.to_pushdown_automata().unwrap().check_input(&mut String::new()));
    let non_epsilon = grammar::parse_grammar("S -> a S | b").unwrap();
    assert!(!non_epsilon.contains_epsilon());
    assert!(!non_epsilon.to_pushdown_automata().unwrap().check_input(&mut String::new()));
    assert!(non_epsilon.to_pushdown_automata().unwrap().check_input(&mut "ab".to_string()));
}

#[test]
fn grammar_to_pda_multichar_terminal_test() {
    // Multi-character terminals and variables now work: entries are atomic.
    let mut multi = Grammar::new("S");
    multi.add_production("S", &["aa"]);
    let pda = multi.to_pushdown_automata().unwrap();
    assert!(pda.check_input(&mut "aa".to_string()));
    assert!(!pda.check_input(&mut "a".to_string()));
    assert!(!pda.check_input(&mut "aaa".to_string()));
    assert!(pda.validate().is_ok());
}

#[test]
fn grammar_to_pda_reserved_symbol_rejected_test() {
    for bad in [",", "a;b", "a/b"] {
        let mut grammar = Grammar::new("S");
        grammar.add_production("S", &[bad]);
        let error = grammar
            .to_pushdown_automata()
            .expect_err("reserved characters must be rejected");
        assert!(error.contains("reserved"), "{}", error);
    }
    // A variable with reserved characters is caught too.
    let mut grammar = Grammar::new("S,x");
    grammar.add_production("S,x", &["a"]);
    assert!(grammar.to_pushdown_automata().is_err());
}

#[test]
fn grammar_to_pda_multichar_language_agreement_test() {
    // L = (ab)^+ c over disjoint character sets {a,b} / {c}: concatenations
    // segment uniquely, so string-level comparison is exact.
    let mut grammar = Grammar::new("S");
    grammar.add_production("S", &["ab", "T"]);
    grammar.add_production("T", &["ab", "T"]);
    grammar.add_production("T", &["c"]);
    let pda = grammar.to_pushdown_automata().unwrap();

    let mut accepted: HashSet<String> = HashSet::new();
    // Token-level enumeration, then concatenation for comparison.
    let token_inputs: Vec<Vec<String>> = (1..=3usize)
        .flat_map(|pairs| {
            (0..=pairs)
                .map(move |t| {
                    let mut tokens = vec!["ab".to_string(); pairs];
                    tokens.push("c".to_string());
                    let _ = t;
                    tokens
                })
                .collect::<Vec<_>>()
        })
        .collect();
    for tokens in token_inputs {
        accepted.insert(tokens.concat());
    }
    assert!(accepted.contains("abc"));
    assert!(accepted.contains("ababc"));

    for input in alphabet_inputs(&['a', 'b', 'c'], 7) {
        let expected = accepted.contains(&input);
        assert_eq!(
            pda.check_input(&mut input.clone()),
            expected,
            "multichar CFG->PDA disagrees on {:?}",
            input
        );
    }

    // Variables keep their names: expansion labels carry them verbatim on
    // the pop side (the string_transitions table is keyed by read symbol, so
    // the states themselves are the source of truth here).
    let has_named_expansion = pda
        .get_states_by_id_ref()
        .values()
        .flat_map(|state| state.iter_by_transition())
        .flat_map(|(_, labels)| labels.iter())
        .any(|label| label.starts_with("ε;T/"));
    assert!(has_named_expansion);
}

/* ---------- Display round-trip ---------- */

#[test]
fn grammar_display_reparses_equivalently_test() {
    for source in [BALANCED_PARENS, ANBN, UNIT_CHAINS] {
        let grammar = grammar::parse_grammar(source).unwrap();
        let rendered = format!("{}", grammar);
        let reparsed = grammar::parse_grammar(&rendered)
            .unwrap_or_else(|e| panic!("rendered grammar failed to parse: {} for\n{}", e, rendered));
        assert_eq!(
            reparsed.productions(),
            grammar.productions(),
            "display round-trip changed productions"
        );
        assert_eq!(reparsed.start_symbol(), grammar.start_symbol());

        // Rendered text of the CNF also stays parseable.
        let cnf = grammar.to_chomsky_normal_form();
        let rendered = format!("{}", cnf);
        grammar::parse_grammar(&rendered)
            .unwrap_or_else(|e| panic!("CNF rendering failed to parse: {} for\n{}", e, rendered));
    }
}
