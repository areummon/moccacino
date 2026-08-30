// This module generates LaTeX export code: TikZ drawings for automata
// diagrams and an align* listing for grammars.
// The generated TikZ code uses only the tikzpicture environment and automata library styles.

use crate::state_machine::{StateNode, Transition};
use moca_data::grammar::Grammar;
use std::collections::HashSet;

/// Exports the automaton to TikZ/PGF code using tikzpicture and automata styles.
/// - `states`: all state nodes (with positions and labels)
/// - `transitions`: all transitions (with from/to, label, and points)
/// - `initial_state`: optional id of the initial state
/// - `final_states`: set of ids of final states
///
/// Returns a String containing the TikZ code.
pub fn export_to_tikz(
    states: &[StateNode],
    transitions: &[Transition],
    initial_state: Option<usize>,
    final_states: &HashSet<usize>,
) -> String {
    // Find bounds for normalization
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

    // Map state id to node name
    let mut id_to_name = std::collections::HashMap::new();
    for (i, s) in states.iter().enumerate() {
        id_to_name.insert(s.id, format!("q{}", i));
    }

    let mut tikz = String::new();
    tikz.push_str("% Paste this into your LaTeX document\n");
    tikz.push_str("% Requires: \\usepackage{tikz} and \\usetikzlibrary{arrows.meta, automata, positioning}\n");
    tikz.push_str("\\begin{center}\n");
    tikz.push_str("\\begin{tikzpicture}[shorten >=1pt, node distance=2cm, on grid, initial text=, auto]\n");

    // Draw states using automata library shapes
    for s in states {
        let x = (s.position.x - min_x) * scale;
        let y = (s.position.y - min_y) * scale;
        let name = id_to_name.get(&s.id).unwrap();
        let is_initial = initial_state == Some(s.id);
        let is_final = final_states.contains(&s.id);
        let mut style = String::from("state");
        if is_initial {
            style.push_str(", initial");
        }
        if is_final {
            style.push_str(", accepting");
        }
        // Format label for math mode: convert q0 -> q_0, q12 -> q_{12}, etc.
        let mut latex_label = s.label.to_string();
        if let Some((_prefix, _digits)) = latex_label.split_once(|c: char| c.is_ascii_digit()) {
            let idx = s.label.chars().position(|c| c.is_ascii_digit()).unwrap_or(0);
            let (prefix, digits) = s.label.split_at(idx);
            if !digits.is_empty() {
                if digits.len() == 1 {
                    latex_label = format!("{}_{}", prefix, digits);
                } else {
                    latex_label = format!("{}_{{{}}}", prefix, digits);
                }
            }
        }
        tikz.push_str(&format!(
            "  \\node[{}] ({}) at ({:.2}, {:.2}) {{${}$}};\n",
            style,
            name,
            x,
            -y, 
            latex_label
        ));
    }

    // Group transitions by (from, to) pairs to handle multiple labels
    let mut transition_groups: std::collections::HashMap<(usize, usize), Vec<String>> = std::collections::HashMap::new();
    
    for t in transitions {
        let key = (t.from_state_id, t.to_state_id);
        let mut label = t.label.to_string();
        if label.trim() == "ε" {
            label = String::from("$\\varepsilon$");
        }
        transition_groups.entry(key).or_insert_with(Vec::new).push(label);
    }

    // Draw transitions with stacked labels
    for ((from_id, to_id), labels) in &transition_groups {
        let from = id_to_name.get(from_id).unwrap();
        let to = id_to_name.get(to_id).unwrap();
        
        if from_id == to_id {
            // Self-loop with stacked labels
            if labels.len() == 1 {
                tikz.push_str(&format!(
                    "  \\path[->] ({}) edge[loop above] node{{{}}} ({});\n",
                    from, labels[0], to
                ));
            } else {
                // Multiple labels for loop - stack them vertically
                tikz.push_str(&format!(
                    "  \\path[->] ({}) edge[loop above] node[align=center]{{{}}} ({});\n",
                    from, labels.join("\\\\"), to
                ));
            }
        } else {
            // Check for reverse edge for curve
            let has_reverse = transition_groups.contains_key(&(*to_id, *from_id));
            
            if has_reverse {
                if labels.len() == 1 {
                    tikz.push_str(&format!(
                        "  \\path[->] ({}) edge[bend left] node{{{}}} ({}) ;\n",
                        from, labels[0], to
                    ));
                } else {
                    // Multiple labels for curved edge - stack them vertically
                    tikz.push_str(&format!(
                        "  \\path[->] ({}) edge[bend left] node[align=center]{{{}}} ({}) ;\n",
                        from, labels.join("\\\\"), to
                    ));
                }
            } else {
                if labels.len() == 1 {
                    tikz.push_str(&format!(
                        "  \\path[->] ({}) edge node{{{}}} ({}) ;\n",
                        from, labels[0], to
                    ));
                } else {
                    // Multiple labels for straight edge - stack them vertically
                    tikz.push_str(&format!(
                        "  \\path[->] ({}) edge node[align=center]{{{}}} ({}) ;\n",
                        from, labels.join("\\\\"), to
                    ));
                }
            }
        }
    }

    tikz.push_str("\\end{tikzpicture}\n");
    tikz.push_str("\\end{center}\n");
    tikz
}

/* Escapes characters with special meaning in LaTeX so arbitrary grammar
 * symbols survive rendering. */
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

/* One math-mode symbol: multi-character terminals are wrapped in \text
 * so their token boundary stays visible (a S b vs. \text{if}). */
fn latex_symbol(symbol: &str) -> String {
    if symbol.chars().count() > 1 {
        format!("\\text{{{}}}", escape_latex(symbol))
    } else {
        escape_latex(symbol)
    }
}

/// Exports the grammar to LaTeX: a comment header carrying the tuple
/// G = (V, Σ, P, S) and one align* line per variable listing its
/// alternatives joined by \mid (empty bodies render as \varepsilon).
pub fn export_grammar_to_latex(grammar: &Grammar) -> String {
    let variables: Vec<String> = grammar.nonterminals().iter().cloned().collect();
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
