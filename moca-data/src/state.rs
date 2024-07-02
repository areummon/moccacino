/* Type to represent the id of a given state. */
pub type StateID = u64;

/* Struct that represents an state of a machine.
 * The name can be repeated but it's id must be unique. */
#[derive(PartialEq, Debug, Eq, Hash)]
pub struct State {
    pub name: String,
    pub initial_flag: bool,
    pub final_flag: bool,
}

impl State {
    pub fn new(name: String) -> Self {
        Self {
            name,
            initial_flag: false,
            final_flag: false,
        }
    }
}

