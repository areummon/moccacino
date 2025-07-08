// This module generates TikZ code for automata diagrams.
// The generated code uses only the tikzpicture environment and automata library styles.

use crate::state_machine::{StateNode, Transition};
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
            -y, // TikZ y axis is up, GUI is down
            latex_label
        ));
    }

    // Draw transitions using automata library edge shapes
    for t in transitions {
        let from = id_to_name.get(&t.from_state_id).unwrap();
        let to = id_to_name.get(&t.to_state_id).unwrap();
        let mut label = t.label.to_string();
        if label.trim() == "ε" {
            label = String::from("$\\varepsilon$");
        }
        if t.from_state_id == t.to_state_id {
            // Use loop above for self-loops
            // You can change it to below per case
            tikz.push_str(&format!(
                "  \\path[->] ({}) edge[loop above] node{{{}}} ({});\n",
                from, label, to
            ));
        } else {
            // Check for reverse edge for curve
            let has_reverse = transitions.iter().any(|other|
                other.from_state_id == t.to_state_id && other.to_state_id == t.from_state_id
            );
            if has_reverse {
                tikz.push_str(&format!(
                    "  \\path[->] ({}) edge[bend left] node{{{}}} ({}) ;\n",
                    from, label, to
                ));
            } else {
                tikz.push_str(&format!(
                    "  \\path[->] ({}) edge node{{{}}} ({}) ;\n",
                    from, label, to
                ));
            }
        }
    }

    tikz.push_str("\\end{tikzpicture}\n");
    tikz.push_str("\\end{center}\n");
    tikz
} 
