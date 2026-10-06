use crate::finite_automata::FiniteAutomata;
use crate::state::StateID;
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

fn add_epsilon(automata: &mut FiniteAutomata, from: StateID, to: StateID) {
    automata.add_transition(from, to, "ε".to_string());
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
