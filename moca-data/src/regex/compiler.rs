use crate::finite_automata::FiniteAutomata;
use crate::state::{StateID, EPSILON};
use crate::state_machine::StateMachine;

use super::ast::RegexAst;

struct Fragment {
    start: StateID,
    end: StateID,
}

pub fn compile(ast: &RegexAst) -> FiniteAutomata {
    let mut automata = FiniteAutomata::new();
    let fragment = build(ast, &mut automata);
    automata.make_initial(fragment.start);
    automata.make_final(fragment.end);
    automata
}

pub fn compile_str(pattern: &str) -> Result<FiniteAutomata, super::parser::ParseError> {
    Ok(compile(&super::parser::parse(pattern)?))
}

fn add_epsilon(automata: &mut FiniteAutomata, from: StateID, to: StateID) {
    automata.add_transition(from, to, EPSILON.to_string());
}

fn build(node: &RegexAst, automata: &mut FiniteAutomata) -> Fragment {
    match node {
        RegexAst::Empty => {
            let start = automata.add_state();
            let end = automata.add_state();
            Fragment { start, end }
        },
        RegexAst::Epsilon => {
            let start = automata.add_state();
            let end = automata.add_state();
            add_epsilon(automata, start, end);
            Fragment { start, end }
        },
        RegexAst::Char(c) => {
            let start = automata.add_state();
            let end = automata.add_state();
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
            let start = automata.add_state();
            let left_fragment = build(left, automata);
            let right_fragment = build(right, automata);
            let end = automata.add_state();
            add_epsilon(automata, start, left_fragment.start);
            add_epsilon(automata, start, right_fragment.start);
            add_epsilon(automata, left_fragment.end, end);
            add_epsilon(automata, right_fragment.end, end);
            Fragment { start, end }
        },
        RegexAst::Star(inner) => {
            let start = automata.add_state();
            let inner_fragment = build(inner, automata);
            let end = automata.add_state();
            add_epsilon(automata, start, end);
            add_epsilon(automata, start, inner_fragment.start);
            add_epsilon(automata, inner_fragment.end, end);
            add_epsilon(automata, inner_fragment.end, inner_fragment.start);
            Fragment { start, end }
        },
        RegexAst::Plus(inner) => {
            let start = automata.add_state();
            let inner_fragment = build(inner, automata);
            let end = automata.add_state();
            add_epsilon(automata, start, inner_fragment.start);
            add_epsilon(automata, inner_fragment.end, end);
            add_epsilon(automata, inner_fragment.end, inner_fragment.start);
            Fragment { start, end }
        },
        RegexAst::Quest(inner) => {
            let start = automata.add_state();
            let inner_fragment = build(inner, automata);
            let end = automata.add_state();
            add_epsilon(automata, start, inner_fragment.start);
            add_epsilon(automata, inner_fragment.end, end);
            add_epsilon(automata, start, end);
            Fragment { start, end }
        },
    }
}
