/* Thompson construction: compiles a RegexAst into a nondeterministic finite
 * automaton using the existing FiniteAutomata structure. Every fragment has a
 * single start state (no incoming transitions from other fragments) and a
 * single end state (no outgoing transitions until further composition), which
 * is what makes the local compositions below sound.
 *
 * ε-transitions use the "ε" label per the library-wide conventions. */
use crate::finite_automata::FiniteAutomata;
use crate::state::{StateID, Input};
use crate::state_machine::StateMachine;

use super::ast::RegexAst;

/* A built piece of the automaton: the ids of its entry and exit states. */
struct Fragment {
    start: StateID,
    end: StateID,
}

/* Compiles an already parsed expression into a new automaton. The resulting
 * automaton has exactly one initial state and one final state. */
pub fn compile(ast: &RegexAst) -> FiniteAutomata {
    let mut automata = FiniteAutomata::new();
    let fragment = build(ast, &mut automata);
    automata.make_initial(fragment.start);
    automata.make_final(fragment.end);
    automata
}

/* Convenience wrapper: parse a pattern and compile it in one step. A
 * literal ε symbol (`\ε`) parses, but automaton labels reserve "ε" for the
 * empty word, so it cannot be compiled faithfully and is rejected here
 * instead of silently turning into an ε move. */
pub fn compile_str(pattern: &str) -> Result<FiniteAutomata, super::parser::ParseError> {
    let ast = super::parser::parse(pattern)?;
    if contains_literal_epsilon(&ast) {
        let chars: Vec<char> = pattern.chars().collect();
        let position = chars
            .windows(2)
            .position(|pair| pair == ['\\', 'ε'])
            .unwrap_or(0);
        return Err(super::parser::ParseError {
            position,
            message: "a literal 'ε' symbol cannot be compiled: automaton labels use ε for the empty word".to_string(),
        });
    }
    Ok(compile(&ast))
}

fn contains_literal_epsilon(node: &RegexAst) -> bool {
    match node {
        RegexAst::Char(c) => *c == 'ε',
        RegexAst::Empty | RegexAst::Epsilon => false,
        RegexAst::Concat(left, right) | RegexAst::Union(left, right) => {
            contains_literal_epsilon(left) || contains_literal_epsilon(right)
        },
        RegexAst::Star(inner) | RegexAst::Plus(inner) | RegexAst::Quest(inner) => {
            contains_literal_epsilon(inner)
        },
    }
}

/* Adds a fresh state and returns its id. */
fn add_state(automata: &mut FiniteAutomata) -> StateID {
    automata.add_state()
}

fn add_epsilon(automata: &mut FiniteAutomata, from: StateID, to: StateID) {
    let epsilon: Input = "ε".to_string();
    automata.add_transition(from, to, epsilon);
}

fn build(node: &RegexAst, automata: &mut FiniteAutomata) -> Fragment {
    match node {
        /* Two states with no connection: the end is unreachable, so nothing
         * is ever accepted. */
        RegexAst::Empty => {
            let start = add_state(automata);
            let end = add_state(automata);
            Fragment { start, end }
        },
        RegexAst::Epsilon => {
            let start = add_state(automata);
            let end = add_state(automata);
            add_epsilon(automata, start, end);
            Fragment { start, end }
        },
        RegexAst::Char(c) => {
            let start = add_state(automata);
            let end = add_state(automata);
            automata.add_transition(start, end, c.to_string());
            Fragment { start, end }
        },
        RegexAst::Concat(left, right) => {
            let left_fragment = build(left, automata);
            let right_fragment = build(right, automata);
            add_epsilon(automata, left_fragment.end, right_fragment.start);
            Fragment {
                start: left_fragment.start,
                end: right_fragment.end,
            }
        },
        RegexAst::Union(left, right) => {
            let start = add_state(automata);
            let left_fragment = build(left, automata);
            let right_fragment = build(right, automata);
            let end = add_state(automata);
            add_epsilon(automata, start, left_fragment.start);
            add_epsilon(automata, start, right_fragment.start);
            add_epsilon(automata, left_fragment.end, end);
            add_epsilon(automata, right_fragment.end, end);
            Fragment { start, end }
        },
        RegexAst::Star(inner) => {
            let start = add_state(automata);
            let inner_fragment = build(inner, automata);
            let end = add_state(automata);
            // Zero repetitions...
            add_epsilon(automata, start, end);
            // ...and looping back for more.
            add_epsilon(automata, start, inner_fragment.start);
            add_epsilon(automata, inner_fragment.end, end);
            add_epsilon(automata, inner_fragment.end, inner_fragment.start);
            Fragment { start, end }
        },
        RegexAst::Plus(inner) => {
            let start = add_state(automata);
            let inner_fragment = build(inner, automata);
            let end = add_state(automata);
            add_epsilon(automata, start, inner_fragment.start);
            add_epsilon(automata, inner_fragment.end, end);
            add_epsilon(automata, inner_fragment.end, inner_fragment.start);
            Fragment { start, end }
        },
        RegexAst::Quest(inner) => {
            let start = add_state(automata);
            let inner_fragment = build(inner, automata);
            let end = add_state(automata);
            add_epsilon(automata, start, inner_fragment.start);
            add_epsilon(automata, inner_fragment.end, end);
            add_epsilon(automata, start, end);
            Fragment { start, end }
        },
    }
}
