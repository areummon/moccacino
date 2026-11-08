
use std::collections::{HashMap, HashSet};

use indexmap::IndexSet;
use moca_data::grammar::Grammar;
use moca_data::state::EPSILON;

use crate::state_machine::StateNode;

pub fn export_to_tikz(
    states: &[StateNode],
    transitions: &HashMap<(usize, usize), IndexSet<String>>,
    initial_state: Option<usize>,
    final_states: &HashSet<usize>,
) -> String {
    let (min_x, min_y, max_x, max_y) = states.iter().fold(
        (f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        |(min_x, min_y, max_x, max_y), s| {
            (
                min_x.min(s.position.x),
                min_y.min(s.position.y),
                max_x.max(s.position.x),
                max_y.max(s.position.y),
            )
        },
    );
    let width = (max_x - min_x).max(1.0);
    let height = (max_y - min_y).max(1.0);
    let scale = 5.0 / width.max(height);

    let id_to_name: HashMap<usize, String> =
        states.iter().enumerate().map(|(i, s)| (s.id, format!("q{i}"))).collect();

    let mut tikz = String::new();
    tikz.push_str("% Paste this into your LaTeX document\n");
    tikz.push_str("% Requires: \\usepackage{tikz} and \\usetikzlibrary{arrows.meta, automata, positioning}\n");
    tikz.push_str("\\begin{center}\n");
    tikz.push_str("\\begin{tikzpicture}[shorten >=1pt, node distance=2cm, on grid, initial text=, auto]\n");

    for s in states {
        let x = (s.position.x - min_x) * scale;
        let y = (s.position.y - min_y) * scale;
        let mut style = String::from("state");
        if initial_state == Some(s.id) {
            style.push_str(", initial");
        }
        if final_states.contains(&s.id) {
            style.push_str(", accepting");
        }
        tikz.push_str(&format!(
            "  \\node[{}] ({}) at ({:.2}, {:.2}) {{${}$}};\n",
            style,
            id_to_name[&s.id],
            x,
            -y,
            state_label(&s.label)
        ));
    }

    let mut edges: Vec<(&(usize, usize), &IndexSet<String>)> = transitions.iter().collect();
    edges.sort_unstable_by_key(|(pair, _)| **pair);
    for (&(from_id, to_id), labels) in edges {
        let (Some(from), Some(to)) = (id_to_name.get(&from_id), id_to_name.get(&to_id)) else { continue };
        let shape = if from_id == to_id {
            "[loop above]"
        } else if transitions.contains_key(&(to_id, from_id)) {
            "[bend left]"
        } else {
            ""
        };
        let align = if labels.len() > 1 { "[align=center]" } else { "" };
        let text: Vec<String> = labels.iter().map(|label| edge_label(label)).collect();
        tikz.push_str(&format!("  \\path[->] ({from}) edge{shape} node{align}{{{}}} ({to});\n", text.join("\\\\")));
    }

    tikz.push_str("\\end{tikzpicture}\n");
    tikz.push_str("\\end{center}\n");
    tikz
}

fn state_label(label: &str) -> String {
    match label.find(|c: char| c.is_ascii_digit()) {
        Some(index) => {
            let (prefix, digits) = label.split_at(index);
            let prefix = escape_latex(prefix);
            if digits.len() == 1 {
                format!("{prefix}_{digits}")
            } else {
                format!("{prefix}_{{{}}}", escape_latex(digits))
            }
        }
        None => escape_latex(label),
    }
}

fn edge_label(label: &str) -> String {
    format!("${}$", escape_latex(label.trim()).replace(EPSILON, "\\varepsilon "))
}

fn escape_latex(symbol: &str) -> String {
    let mut escaped = String::with_capacity(symbol.len());
    for c in symbol.chars() {
        match c {
            '&' | '%' | '$' | '#' | '_' | '{' | '}' => {
                escaped.push('\\');
                escaped.push(c);
            }
            '~' => escaped.push_str("\\sim "),
            '^' => escaped.push_str("\\wedge "),
            '\\' => escaped.push_str("\\backslash "),
            _ => escaped.push(c),
        }
    }
    escaped
}

fn latex_symbol(symbol: &str) -> String {
    if symbol.chars().count() > 1 {
        format!("\\text{{{}}}", escape_latex(symbol))
    } else {
        escape_latex(symbol)
    }
}

pub fn export_grammar_to_latex(grammar: &Grammar) -> String {
    let variables: Vec<String> = grammar.nonterminals().cloned().collect();
    let terminals: Vec<String> = grammar.terminals().into_iter().collect();

    let mut latex = String::new();
    latex.push_str("% Grammar in tuple form\n");
    latex.push_str("% Requires: \\usepackage{amsmath}\n");
    latex.push_str(&format!(
        "% G = ({{{}}}, {{{}}}, P, {})\n",
        variables.join(", "),
        terminals.join(", "),
        escape_latex(grammar.start_symbol()),
    ));
    latex.push_str("\\begin{center}\n");
    latex.push_str("\\begin{align*}\n");

    let mut lines = Vec::new();
    for variable in &variables {
        let bodies = grammar
            .productions_of(variable)
            .map(|bodies| bodies.as_slice())
            .unwrap_or(&[]);
        let alternatives: Vec<String> = bodies
            .iter()
            .map(|body| {
                if body.is_empty() {
                    String::from("\\varepsilon")
                } else {
                    body.iter()
                        .map(|symbol| latex_symbol(symbol))
                        .collect::<Vec<_>>()
                        .join(" ")
                }
            })
            .collect();
        lines.push(format!(
            "{} &\\rightarrow {}",
            latex_symbol(variable),
            alternatives.join(" \\mid "),
        ));
    }
    latex.push_str(&lines.join(" \\\\\n"));
    latex.push_str("\n\\end{align*}\n");
    latex.push_str("\\end{center}\n");
    latex
}
