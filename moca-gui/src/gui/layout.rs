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
const Y_SPACING: f32 = 150.0;
const TARGET_WIDTH: f32 = 1500.0;

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
    order_layers(&mut layers, &neighbors, &reachable);
    place(states, &layers, &reachable);
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

fn order_layers(layers: &mut [Vec<usize>], neighbors: &[Vec<usize>], reachable: &[bool]) {
    let node_count = neighbors.len();
    let max_layer = layers.len() - 1;
    let mut undirected: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); node_count];
    for u in 0..node_count {
        if !reachable[u] {
            continue;
        }
        for &v in &neighbors[u] {
            if reachable[v] && u != v {
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

fn place(states: &mut [StateNode], layers: &[Vec<usize>], reachable: &[bool]) {
    let widest = layers
        .iter()
        .map(|layer_nodes| layer_nodes.len())
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    let x_spacing = (TARGET_WIDTH / widest).clamp(120.0, 260.0);
    for (layer_index, layer_nodes) in layers.iter().enumerate() {
        let count = layer_nodes.len() as f32;
        for (i, &u) in layer_nodes.iter().enumerate() {
            let x = START_X + (i as f32 - (count - 1.0) / 2.0) * x_spacing;
            let y = START_Y + layer_index as f32 * Y_SPACING;
            states[u].position = iced::Point::new(x, y);
        }
    }

    let unreachable: Vec<usize> = (0..states.len()).filter(|&u| !reachable[u]).collect();
    if !unreachable.is_empty() {
        let per_row = ((TARGET_WIDTH / x_spacing).floor() as usize).max(1);
        for (row, chunk) in unreachable.chunks(per_row).enumerate() {
            let count = chunk.len() as f32;
            for (i, &u) in chunk.iter().enumerate() {
                let x = START_X + (i as f32 - (count - 1.0) / 2.0) * x_spacing;
                let y = START_Y + (layers.len() + row) as f32 * Y_SPACING;
                states[u].position = iced::Point::new(x, y);
            }
        }
    }
}

fn grid(states: &mut [StateNode]) {
    if states.is_empty() {
        return;
    }

    let grid_size = (states.len() as f32).sqrt().ceil() as usize;
    let spacing = 150.0;
    let start_x = 100.0;
    let start_y = 100.0;

    for (i, state) in states.iter_mut().enumerate() {
        let row = i / grid_size;
        let col = i % grid_size;
        state.position = iced::Point::new(
            start_x + (col as f32 * spacing),
            start_y + (row as f32 * spacing)
        );
    }
}
