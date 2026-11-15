use std::collections::VecDeque;

use crate::state_machine::Drawing;

const LIMIT: usize = 200;

/// Undo and redo over whole-drawing snapshots. Edits never report themselves: `record`
/// compares the drawing with the last snapshot, so no edit path can forget a checkpoint.
#[derive(Default)]
pub(crate) struct History {
    committed: Drawing,
    undo: VecDeque<Drawing>,
    redo: Vec<Drawing>,
}

impl History {
    pub(crate) fn starting_at(drawing: &Drawing) -> Self {
        Self { committed: drawing.clone(), ..Self::default() }
    }

    pub(crate) fn record(&mut self, current: &Drawing) {
        if *current == self.committed {
            return;
        }
        self.undo.push_back(std::mem::replace(&mut self.committed, current.clone()));
        if self.undo.len() > LIMIT {
            self.undo.pop_front();
        }
        self.redo.clear();
    }

    pub(crate) fn undo(&mut self, current: &mut Drawing) -> bool {
        self.record(current);
        let Some(previous) = self.undo.pop_back() else {
            return false;
        };
        self.redo.push(std::mem::replace(&mut self.committed, previous));
        current.clone_from(&self.committed);
        true
    }

    pub(crate) fn redo(&mut self, current: &mut Drawing) -> bool {
        self.record(current);
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push_back(std::mem::replace(&mut self.committed, next));
        current.clone_from(&self.committed);
        true
    }

    pub(crate) fn can_undo(&self, current: &Drawing) -> bool {
        !self.undo.is_empty() || *current != self.committed
    }

    pub(crate) fn can_redo(&self, current: &Drawing) -> bool {
        !self.redo.is_empty() && *current == self.committed
    }
}
