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

/* Convenience wrapper: parse a pattern and compile it in one step. */
pub fn compile_str(pattern: &str) -> Result<FiniteAutomata, super::parser::ParseError> {
    let ast = super::parser::parse(pattern)?;
    Ok(compile(&ast))
}

/* Adds a fresh state and returns its id (ids are assigned densely). */
fn add_state(automata: &mut FiniteAutomata) -> StateID {
    let id = automata.get_states_by_id_ref().len() as StateID;
    automata.add_state();
    id
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
