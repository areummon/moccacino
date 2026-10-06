use std::collections::HashMap;

use iced::{Point, Vector};

use super::edge::EdgeGeometry;
use super::node::StateNode;
use super::program::StateMachine;
use super::util::VectorExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hit {
    State(usize),
    Transition((usize, usize)),
}

impl StateMachine<'_> {
    pub(crate) fn node_by_id(&self) -> HashMap<usize, &StateNode> {
        self.states.iter().map(|node| (node.id, node)).collect()
    }

    pub(crate) fn edge_geometry(
        &self,
        node_by_id: &HashMap<usize, &StateNode>,
        from: usize,
        to: usize,
    ) -> Option<EdgeGeometry> {
        let from_node = node_by_id.get(&from)?;
        let to_node = node_by_id.get(&to)?;
        let has_reverse = from != to && self.transitions.contains_key(&(to, from));
        let loop_direction = if from == to {
            self.loop_direction(node_by_id, from_node)
        } else {
            Vector::new(0.0, -1.0)
        };
        Some(EdgeGeometry::new(from_node, to_node, has_reverse, loop_direction))
    }

    fn loop_direction(&self, node_by_id: &HashMap<usize, &StateNode>, node: &StateNode) -> Vector {
        let mut sum = Vector::new(0.0, 0.0);
        for &(from, to) in self.transitions.keys() {
            let other = if from == node.id && to != node.id {
                to
            } else if to == node.id && from != node.id {
                from
            } else {
                continue;
            };
            if let Some(neighbor) = node_by_id.get(&other) {
                sum = sum + (neighbor.position - node.position).unit();
            }
        }
        if sum.length() < 0.35 {
            Vector::new(0.0, -1.0)
        } else {
            (sum * -1.0).unit()
        }
    }

    pub(crate) fn hit_test(&self, point: Point) -> Option<Hit> {
        if let Some(node) = self
            .states
            .iter()
            .rev()
            .find(|node| (point - node.position).length() <= node.radius)
        {
            return Some(Hit::State(node.id));
        }
        self.find_transition_at_point(point).map(Hit::Transition)
    }

    pub(crate) fn find_transition_at_point(&self, point: Point) -> Option<(usize, usize)> {
        let node_by_id = self.node_by_id();
        self.transitions.iter().find_map(|(&(from, to), labels)| {
            let geometry = self.edge_geometry(&node_by_id, from, to)?;
            geometry.hit(point, labels).then_some((from, to))
        })
    }
}
