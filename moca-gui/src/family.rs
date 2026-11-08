#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Finite,
    Pushdown,
    Turing,
    Grammar,
}

impl Family {
    pub(crate) const ALL: [Family; 4] =
        [Family::Finite, Family::Turing, Family::Pushdown, Family::Grammar];

    pub(crate) fn title(self) -> &'static str {
        match self {
            Family::Finite => "Finite Automaton",
            Family::Pushdown => "Pushdown Automaton",
            Family::Turing => "Turing Machine",
            Family::Grammar => "Grammar",
        }
    }

    pub(crate) fn blurb(self) -> &'static str {
        match self {
            Family::Finite => "DFAs and NFAs, regex, minimization",
            Family::Pushdown => "Stack-powered context-free recognizers",
            Family::Turing => "Tapes, heads and multi-tape machines",
            Family::Grammar => "Context-free rules, CNF and derivations",
        }
    }
}
