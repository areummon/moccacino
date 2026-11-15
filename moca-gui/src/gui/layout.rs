use std::collections::{BTreeSet, HashMap, VecDeque};

use indexmap::IndexSet;

use crate::state_machine::StateNode;

pub(crate) fn arrange(states: &mut [StateNode], transitions: &HashMap<(usize, usize), IndexSet<String>>, initial: Option<usize>) {
    match initial {
        Some(initial) => layered(states, transitions, initial),
        None => grid(states),
    }
}

const START_X: f32 = 150.0;
const START_Y: f32 = 150.0;
const X_SPACING: f32 = 190.0;
const Y_SPACING: f32 = 140.0;

fn layered(states: &mut [StateNode], transitions: &HashMap<(usize, usize), IndexSet<String>>, initial: usize) {
    if states.is_empty() {
        return;
    }
    let index_of: HashMap<usize, usize> = states
        .iter()
        .enumerate()
        .map(|(index, state)| (state.id, index))
        .collect();
    let Some(&root) = index_of.get(&initial) else {
        grid(states);
        return;
    };
    let neighbors = adjacency(states.len(), transitions, &index_of);
    let (preorder, back_edge) = depth_first(&neighbors, root);
    let mut reachable = vec![false; neighbors.len()];
    for &u in &preorder {
        reachable[u] = true;
    }
    let mut layers = assign_layers(&neighbors, &back_edge, &reachable, &preorder);
    let routed = route_long_edges(&mut layers, &neighbors, &reachable);
    order_layers(&mut layers, &routed);
    place(states, &layers, &routed, &reachable);
}

/// Splits every edge spanning more than one layer into a chain through
/// placeholder nodes (indices past the real states), so ordering and
/// alignment reserve room for it in every layer it crosses.
fn route_long_edges(layers: &mut [Vec<usize>], neighbors: &[Vec<usize>], reachable: &[bool]) -> Vec<Vec<usize>> {
    let mut layer_of = vec![0usize; neighbors.len()];
    for (layer_index, layer_nodes) in layers.iter().enumerate() {
        for &u in layer_nodes {
            layer_of[u] = layer_index;
        }
    }
    let mut routed: Vec<Vec<usize>> = vec![Vec::new(); neighbors.len()];
    for u in (0..neighbors.len()).filter(|&u| reachable[u]) {
        for &v in neighbors[u].iter().filter(|&&v| reachable[v]) {
            let (from, to) = if layer_of[u] <= layer_of[v] { (u, v) } else { (v, u) };
            let mut previous = from;
            for crossed in &mut layers[layer_of[from] + 1..layer_of[to]] {
                let placeholder = routed.len();
                routed.push(Vec::new());
                routed[previous].push(placeholder);
                crossed.push(placeholder);
                previous = placeholder;
            }
            routed[previous].push(to);
        }
    }
    routed
}

fn adjacency(
    node_count: usize,
    transitions: &HashMap<(usize, usize), IndexSet<String>>,
    index_of: &HashMap<usize, usize>,
) -> Vec<Vec<usize>> {
    let mut adjacency: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); node_count];
    for &(from, to) in transitions.keys() {
        if from != to {
            if let (Some(&fi), Some(&ti)) = (index_of.get(&from), index_of.get(&to)) {
                adjacency[fi].insert(ti);
            }
        }
    }
    adjacency
        .into_iter()
        .map(|set| set.into_iter().collect())
        .collect()
}

fn depth_first(neighbors: &[Vec<usize>], root: usize) -> (Vec<usize>, Vec<BTreeSet<usize>>) {
    let node_count = neighbors.len();
    let mut color = vec![0u8; node_count];
    let mut back_edge = vec![BTreeSet::new(); node_count];
    let mut preorder: Vec<usize> = Vec::new();
    let mut iter_pos = vec![0usize; node_count];
    let mut stack: Vec<usize> = vec![root];
    color[root] = 1;
    preorder.push(root);
    while let Some(&u) = stack.last() {
        if iter_pos[u] < neighbors[u].len() {
            let v = neighbors[u][iter_pos[u]];
            iter_pos[u] += 1;
            match color[v] {
                0 => {
                    color[v] = 1;
                    preorder.push(v);
                    stack.push(v);
                }
                1 => {
                    back_edge[u].insert(v);
                }
                _ => {}
            }
        } else {
            color[u] = 2;
            stack.pop();
        }
    }
    (preorder, back_edge)
}

fn assign_layers(
    neighbors: &[Vec<usize>],
    back_edge: &[BTreeSet<usize>],
    reachable: &[bool],
    preorder: &[usize],
) -> Vec<Vec<usize>> {
    let node_count = neighbors.len();
    let mut indegree = vec![0usize; node_count];
    let mut kept: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); node_count];
    for u in 0..node_count {
        if !reachable[u] {
            continue;
        }
        for &v in &neighbors[u] {
            if reachable[v] && !back_edge[u].contains(&v) {
                kept[u].insert(v);
                indegree[v] += 1;
            }
        }
    }
    let mut layer = vec![0usize; node_count];
    let mut queue: VecDeque<usize> = (0..node_count)
        .filter(|&u| reachable[u] && indegree[u] == 0)
        .collect();
    while let Some(u) = queue.pop_front() {
        for &v in &kept[u] {
            layer[v] = layer[v].max(layer[u] + 1);
            indegree[v] -= 1;
            if indegree[v] == 0 {
                queue.push_back(v);
            }
        }
    }

    let max_layer = preorder.iter().map(|&u| layer[u]).max().unwrap_or(0);
    let mut layers: Vec<Vec<usize>> = vec![Vec::new(); max_layer + 1];
    for &u in preorder {
        layers[layer[u]].push(u);
    }
    layers
}

fn order_layers(layers: &mut [Vec<usize>], neighbors: &[Vec<usize>]) {
    let node_count = neighbors.len();
    let max_layer = layers.len() - 1;
    let mut undirected: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); node_count];
    for u in 0..node_count {
        for &v in &neighbors[u] {
            if u != v {
                undirected[u].insert(v);
                undirected[v].insert(u);
            }
        }
    }
    let mut pos = vec![0usize; node_count];
    for layer_nodes in layers.iter() {
        for (index, &u) in layer_nodes.iter().enumerate() {
            pos[u] = index;
        }
    }
    for sweep in 0..4 {
        let order: Vec<usize> = if sweep % 2 == 0 {
            (1..=max_layer).collect()
        } else {
            (0..max_layer).rev().collect()
        };
        for l in order {
            let mut keys: Vec<(usize, f32)> = layers[l]
                .iter()
                .map(|&u| {
                    let neighbor_set = &undirected[u];
                    let key = if neighbor_set.is_empty() {
                        pos[u] as f32
                    } else {
                        let total: f32 = neighbor_set.iter().map(|&v| pos[v] as f32).sum();
                        total / neighbor_set.len() as f32
                    };
                    (u, key)
                })
                .collect();
            keys.sort_by(|a, b| {
                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            });
            layers[l] = keys.into_iter().map(|(u, _)| u).collect();
            for (index, &u) in layers[l].iter().enumerate() {
                pos[u] = index;
            }
        }
    }
}

fn place(states: &mut [StateNode], layers: &[Vec<usize>], routed: &[Vec<usize>], reachable: &[bool]) {
    let mut linked: Vec<Vec<usize>> = vec![Vec::new(); routed.len()];
    for (u, targets) in routed.iter().enumerate() {
        for &v in targets {
            linked[u].push(v);
            linked[v].push(u);
        }
    }
    let mut y = vec![0.0f32; routed.len()];
    for layer_nodes in layers {
        for (i, &u) in layer_nodes.iter().enumerate() {
            y[u] = (i as f32 - (layer_nodes.len() / 2) as f32) * Y_SPACING;
        }
    }
    for sweep in 0..8 {
        let order: Vec<usize> = if sweep % 2 == 0 {
            (0..layers.len()).collect()
        } else {
            (0..layers.len()).rev().collect()
        };
        for l in order {
            align_layer(&layers[l], &linked, states.len(), &mut y);
        }
    }
    clear_long_edges(layers, routed, states.len(), &mut y);

    let top = layers.iter().flatten().map(|&u| y[u]).fold(f32::INFINITY, f32::min);
    let bottom = layers.iter().flatten().map(|&u| y[u]).fold(f32::NEG_INFINITY, f32::max);
    for (layer_index, layer_nodes) in layers.iter().enumerate() {
        for &u in layer_nodes {
            if let Some(state) = states.get_mut(u) {
                let x = START_X + layer_index as f32 * X_SPACING;
                state.position = iced::Point::new(x, START_Y + y[u] - top);
            }
        }
    }

    let unreachable: Vec<usize> = (0..states.len()).filter(|&u| !reachable[u]).collect();
    if !unreachable.is_empty() {
        let rows = ((bottom - top) / Y_SPACING).round() as usize + 1;
        let per_column = rows.max((unreachable.len() as f32).sqrt().ceil() as usize);
        let center_y = START_Y + (bottom - top) / 2.0;
        for (column, chunk) in unreachable.chunks(per_column).enumerate() {
            let x = START_X + (layers.len() + column) as f32 * X_SPACING;
            let count = chunk.len() as f32;
            for (i, &u) in chunk.iter().enumerate() {
                let y = center_y + (i as f32 - (count - 1.0) / 2.0) * Y_SPACING;
                states[u].position = iced::Point::new(x, y);
            }
        }
    }
}

/// Edges are drawn straight between their states, ignoring the placeholders,
/// so a state left on such a line is pushed (with its layer neighbors on that
/// side) one row away until no long edge runs through a state.
fn clear_long_edges(layers: &[Vec<usize>], routed: &[Vec<usize>], state_count: usize, y: &mut [f32]) {
    let mut layer_of = vec![0usize; routed.len()];
    for (layer_index, layer_nodes) in layers.iter().enumerate() {
        for &u in layer_nodes {
            layer_of[u] = layer_index;
        }
    }
    let mut long_edges: Vec<(usize, usize)> = Vec::new();
    for u in 0..state_count {
        for &next in &routed[u] {
            let mut v = next;
            while v >= state_count {
                v = routed[v][0];
            }
            if layer_of[v] >= layer_of[u] + 2 {
                long_edges.push((u, v));
            }
        }
    }
    let clearance = StateNode::RADIUS * 1.5;
    for _ in 0..4 * state_count {
        let blocked = long_edges.iter().find_map(|&(u, v)| {
            (layer_of[u] + 1..layer_of[v]).find_map(|l| {
                let t = (l - layer_of[u]) as f32 / (layer_of[v] - layer_of[u]) as f32;
                let line_y = y[u] + (y[v] - y[u]) * t;
                let slope = (y[v] - y[u]) / ((layer_of[v] - layer_of[u]) as f32 * X_SPACING);
                layers[l]
                    .iter()
                    .find(|&&w| w < state_count && (y[w] - line_y).abs() / slope.hypot(1.0) < clearance)
                    .map(|&w| (l, w, y[w] >= line_y))
            })
        });
        let Some((l, w, down)) = blocked else {
            break;
        };
        let pivot = y[w];
        for &v in &layers[l] {
            if down && y[v] >= pivot {
                y[v] += Y_SPACING;
            } else if !down && y[v] <= pivot {
                y[v] -= Y_SPACING;
            }
        }
    }
}

/// Priority placement on whole rows: each node moves toward the mean row of
/// its neighbors, placeholders first so long edges tend to stay straight.
fn align_layer(nodes: &[usize], linked: &[Vec<usize>], state_count: usize, y: &mut [f32]) {
    let desired: Vec<f32> = nodes
        .iter()
        .map(|&u| match linked[u].len() {
            0 => y[u],
            degree => (linked[u].iter().map(|&v| y[v]).sum::<f32>() / degree as f32 / Y_SPACING).round() * Y_SPACING,
        })
        .collect();
    let mut by_priority: Vec<usize> = (0..nodes.len()).collect();
    by_priority.sort_by_key(|&k| {
        let u = nodes[k];
        std::cmp::Reverse(if u >= state_count { usize::MAX } else { linked[u].len() })
    });
    let mut placed: Vec<Option<f32>> = vec![None; nodes.len()];
    for k in by_priority {
        let mut value = desired[k];
        if let Some((above, at)) = (0..k).rev().find_map(|j| placed[j].map(|p| (j, p))) {
            value = value.max(at + (k - above) as f32 * Y_SPACING);
        }
        if let Some((below, at)) = (k + 1..nodes.len()).find_map(|j| placed[j].map(|p| (j, p))) {
            value = value.min(at - (below - k) as f32 * Y_SPACING);
        }
        placed[k] = Some(value);
    }
    for (k, &u) in nodes.iter().enumerate() {
        y[u] = placed[k].unwrap_or(y[u]);
    }
}

fn grid(states: &mut [StateNode]) {
    let columns = (states.len() as f32).sqrt().ceil().max(1.0) as usize;
    for (i, state) in states.iter_mut().enumerate() {
        let row = i / columns;
        let column = i % columns;
        state.position = iced::Point::new(
            START_X + column as f32 * X_SPACING,
            START_Y + row as f32 * Y_SPACING,
        );
    }
}
