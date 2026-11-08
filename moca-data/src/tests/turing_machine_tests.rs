use super::turing;
use crate::state_machine::{Machine, StateMachine};
use crate::turing_machine::{
    parse_transition, Configuration, Direction, RunOutcome, Tape, TransitionOp, TuringMachine, DEFAULT_MAX_STEPS,
};

fn build_anbn_machine() -> TuringMachine {
    turing('_', 5, &[4], &[
        (0, 1, "a;X/R"), (0, 0, "X;X/R"), (0, 3, "Y;Y/R"), (0, 4, "_;_/S"),
        (1, 1, "a;a/R"), (1, 1, "Y;Y/R"), (1, 2, "b;Y/L"),
        (2, 2, "a;a/L"), (2, 2, "Y;Y/L"), (2, 0, "X;X/R"),
        (3, 3, "Y;Y/R"), (3, 4, "_;_/S"),
    ])
}

fn tape_count(tm: &TuringMachine) -> usize {
    tm.initial_configuration("").unwrap().tapes().len()
}

fn window(tape: &Tape) -> String {
    tape.snapshot(1).0
}

#[test]
fn turing_anbn_decider_test() {
    let tm = build_anbn_machine();
    assert_eq!(tm.get_blank_symbol(), '_');
    assert_eq!(tm.validate(), Ok(()));

    for (input, expected) in [
        ("", true),
        ("ab", true),
        ("aabb", true),
        ("aaabbb", true),
        ("aaaaaaaaaabbbbbbbbbb", true),
        ("a", false),
        ("b", false),
        ("ba", false),
        ("aab", false),
        ("abb", false),
        ("aabbb", false),
        ("abab", false),
    ] {
        assert_eq!(tm.accepts(input), expected, "wrong answer for input {input:?}");
        let expected_outcome = if expected { RunOutcome::Accepted } else { RunOutcome::Rejected };
        assert_eq!(tm.run(input, DEFAULT_MAX_STEPS), Some(expected_outcome), "run outcome differs for {input:?}");
    }
}

#[test]
fn turing_parity_machine_test() {
    let tm = turing('_', 4, &[2], &[(0, 1, "a;a/R"), (1, 0, "a;a/R"), (0, 2, "_;_/S"), (1, 3, "_;_/S")]);
    assert_eq!(tm.run("", DEFAULT_MAX_STEPS), Some(RunOutcome::Accepted));
    assert_eq!(tm.run("aaaa", DEFAULT_MAX_STEPS), Some(RunOutcome::Accepted));
    assert_eq!(tm.run("a", DEFAULT_MAX_STEPS), Some(RunOutcome::Rejected));
    assert_eq!(tm.run("aaaba", DEFAULT_MAX_STEPS), Some(RunOutcome::Rejected));
}

#[test]
fn turing_step_trace_and_tape_test() {
    let tm = turing('_', 3, &[2], &[
        (0, 0, "a;X/R"), (0, 0, "b;Y/R"), (0, 0, "c;Z/R"), (0, 1, "_;_/L"), (1, 2, "Z;Z/S"),
    ]);

    let mut config = tm.initial_configuration("abc").unwrap();
    assert_eq!(config.state_id(), 0);
    assert_eq!(window(config.tape()), "_ab");

    let mut trace = Vec::new();
    while let Some(next) = tm.step(&config) {
        config = next;
        trace.push((config.state_id(), window(config.tape())));
    }
    assert_eq!(
        trace,
        [
            (0, "Xbc".to_string()),
            (0, "Yc_".to_string()),
            (0, "Z__".to_string()),
            (1, "YZ_".to_string()),
            (2, "YZ_".to_string()),
        ]
    );
    assert_eq!(tm.run("abc", DEFAULT_MAX_STEPS), Some(RunOutcome::Accepted));
    assert_eq!(tm.run("abd", DEFAULT_MAX_STEPS), Some(RunOutcome::Rejected));

    let mut tape = Tape::new_with_input("ab", '#');
    assert_eq!(tape.read(), 'a');
    tape.move_head(Direction::Right);
    assert_eq!(tape.read(), 'b');
    tape.write('Z');
    assert_eq!(tape.read(), 'Z');
    for _ in 0..3 {
        tape.move_head(Direction::Left);
    }
    assert_eq!(tape.read(), '#');
    assert_eq!(tape.snapshot(2), ("####a".to_string(), 2));
}

#[test]
fn turing_run_max_steps_exceeded_test() {
    let tm = turing('_', 1, &[], &[(0, 0, "_;_/R")]);
    assert_eq!(tm.run("", 1000), Some(RunOutcome::MaxStepsExceeded));
    assert!(!tm.accepts(""));
}

#[test]
fn turing_validate_test() {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(1);
    assert_eq!(tm.validate(), Err("The Turing machine has no initial state.".to_string()));

    for bad_label in ["ab;c/d", "a;bc/d", "a;b/QR", "a;b", "a", "a;b/c/d", "", "ε;b/c", "a;b;c/R"] {
        let tm = turing('_', 2, &[1], &[(0, 1, bad_label)]);
        assert!(Machine::validate(&tm).is_err(), "label {bad_label:?} must not validate");
        assert_eq!(tm.run("anything", 100), Some(RunOutcome::Rejected));
    }
}

#[test]
fn turing_label_codec_round_trip_test() {
    let ops = parse_transition("a;X/r,_;*/L").unwrap();
    assert_eq!(
        ops,
        [
            TransitionOp { read: 'a', write: 'X', direction: Direction::Right },
            TransitionOp { read: '_', write: '*', direction: Direction::Left },
        ]
    );
    let text = crate::turing_machine::format_transition(&ops);
    assert_eq!(text, "a;X/R,_;*/L");
    assert_eq!(parse_transition(&text).unwrap(), ops);
    assert_eq!(TransitionOp::parse("a;b;c/R"), None);
}

#[test]
fn turing_nondeterminism_flag_test() {
    let mut tm = turing('_', 2, &[], &[(0, 1, "a;a/R")]);
    assert!(tm.is_deterministic());
    tm.add_transition(0, 0, "a;b/S".to_string());
    assert!(!tm.is_deterministic());
    let config = tm.initial_configuration("a").unwrap();
    assert_eq!(tm.step(&config), None);

    let tm = turing('_', 2, &[1], &[(0, 1, "a;a/R"), (0, 1, "a;a/R")]);
    assert!(tm.is_deterministic());
    assert_eq!(tm.run("a", 10), Some(RunOutcome::Accepted));

    let tm = turing('_', 2, &[], &[(0, 1, "a;a/R")]);
    assert_eq!(tm.run("a", 10), Some(RunOutcome::Rejected));
}

#[test]
fn turing_configuration_helpers_test() {
    let mut headless = TuringMachine::new('_');
    headless.add_n_states(1);
    assert_eq!(headless.initial_configuration("ab"), None);
    assert_eq!(headless.run("ab", 10), None);

    let mut tm = build_anbn_machine();
    tm.remove_state(0);
    assert!(tm.validate().is_err());
}

fn build_union_machine() -> TuringMachine {
    turing('_', 7, &[5], &[
        (0, 1, "a;a/R"), (0, 2, "b;b/R"), (0, 3, "a;a/R"), (0, 3, "b;b/R"), (0, 5, "_;_/S"),
        (1, 2, "a;a/R"), (1, 1, "b;b/R"), (2, 1, "a;a/R"), (2, 2, "b;b/R"),
        (1, 6, "_;_/S"), (2, 5, "_;_/S"),
        (3, 3, "a;a/R"), (3, 4, "b;b/R"), (4, 3, "a;a/R"), (4, 5, "b;b/S"),
        (3, 6, "_;_/S"), (4, 6, "_;_/S"),
    ])
}

#[test]
fn turing_nondeterministic_union_language_test() {
    let tm = build_union_machine();
    assert!(!tm.is_deterministic());
    assert_eq!(tm.validate(), Ok(()));

    for (input, expected) in [
        ("", true),
        ("aa", true),
        ("abba", true),
        ("abb", true),
        ("bb", true),
        ("b", true),
        ("aab", true),
        ("bab", false),
        ("ab", false),
        ("ababa", false),
    ] {
        assert_eq!(
            tm.run_nondeterministic(input, DEFAULT_MAX_STEPS).unwrap(),
            if expected { RunOutcome::Accepted } else { RunOutcome::Rejected },
            "wrong outcome for {input:?}"
        );
        assert_eq!(tm.accepts(input), expected, "accepts() must dispatch to the nondeterministic run for {input:?}");
    }
}

#[test]
fn turing_step_all_branching_test() {
    let tm = turing('_', 3, &[], &[(0, 1, "a;X/S"), (0, 2, "a;Y/S")]);
    let config = tm.initial_configuration("a").unwrap();
    let mut branches: Vec<(u64, char)> = tm.step_all(&config).iter().map(|c| (c.state_id(), c.tape().read())).collect();
    branches.sort();
    assert_eq!(branches, [(1, 'X'), (2, 'Y')]);
    assert_eq!(tm.step(&config), None);

    let det_tm = turing('_', 2, &[], &[(0, 1, "a;X/S")]);
    let config = det_tm.initial_configuration("a").unwrap();
    assert_eq!(det_tm.step_all(&config).len(), 1);
    assert_eq!(det_tm.step(&config).map(|c| c.tape().read()), Some('X'));
}

#[test]
fn turing_nondeterministic_looping_branch_test() {
    let tm = turing('_', 4, &[3], &[
        (0, 1, "a;a/R"), (0, 2, "a;a/R"), (1, 1, "a;a/R"), (1, 1, "_;_/R"), (2, 2, "a;a/R"), (0, 3, "z;z/S"),
    ]);
    assert!(!tm.is_deterministic());
    assert_eq!(tm.run_nondeterministic("aaaa", 500), Some(RunOutcome::MaxStepsExceeded));
    assert_eq!(tm.run_nondeterministic("z", 10), Some(RunOutcome::Accepted));
}

#[test]
fn turing_nondeterministic_all_reject_test() {
    let tm = turing('_', 4, &[], &[(0, 1, "a;a/R"), (0, 2, "a;a/R"), (1, 3, "_;_/S"), (2, 3, "_;_/S")]);
    assert_eq!(tm.run_nondeterministic("a", 100), Some(RunOutcome::Rejected));
}

fn build_marker_machine() -> TuringMachine {
    turing('_', 2, &[1], &[(0, 0, "a;a/R,_;*/R"), (0, 1, "_;_/S,_;_/S")])
}

#[test]
fn turing_multitape_machine_test() {
    let tm = build_marker_machine();
    assert_eq!(tape_count(&tm), 2);
    assert_eq!(tm.validate(), Ok(()));

    let mut config: Configuration = tm.initial_configuration("aa").unwrap();
    let windows = |config: &Configuration| -> Vec<String> { config.tapes().iter().map(window).collect() };
    assert_eq!(windows(&config), ["_aa", "___"]);
    config = tm.step(&config).unwrap();
    assert_eq!(windows(&config), ["aa_", "*__"]);
    config = tm.step(&config).unwrap();
    assert_eq!(windows(&config), ["a__", "*__"]);
    config = tm.step(&config).unwrap();
    assert_eq!(config.state_id(), 1);

    assert_eq!(tm.run("aaa", DEFAULT_MAX_STEPS), Some(RunOutcome::Accepted));
    assert_eq!(tm.run("", DEFAULT_MAX_STEPS), Some(RunOutcome::Accepted));
    assert_eq!(tm.run("aab", DEFAULT_MAX_STEPS), Some(RunOutcome::Rejected));
    assert!(tm.accepts("aaaa"));
    assert!(!tm.accepts("aba"));
}

#[test]
fn turing_multitape_inconsistent_label_test() {
    let mut tm = build_marker_machine();
    tm.add_transition(0, 0, "z;z/S".to_string());
    assert_eq!(
        Machine::validate(&tm),
        Err("State 0: inconsistent tape count: label \"z;z/S\" uses 1 tape(s), but the machine has 2.".to_string())
    );
    assert_eq!(tm.run("z", 100), Some(RunOutcome::Rejected));
}

#[test]
fn turing_multitape_nondeterministic_test() {
    let tm = turing('_', 3, &[1], &[(0, 1, "a;a/S,_;_/S"), (0, 2, "a;a/S,_;_/S")]);
    assert!(!tm.is_deterministic());
    let config = tm.initial_configuration("ab").unwrap();
    assert_eq!(tm.step_all(&config).len(), 2);
    assert_eq!(tm.step(&config), None);
    assert_eq!(tm.run_nondeterministic("ab", 100), Some(RunOutcome::Accepted));
    assert_eq!(tm.run_nondeterministic("b", 100), Some(RunOutcome::Rejected));
}

#[test]
fn turing_tape_canonical_form_test() {
    let mut tape = Tape::new_with_input("ax", '#');
    tape.move_head(Direction::Right);
    tape.write('X');
    assert_eq!(tape.read(), 'X');
    tape.write('#');
    let mut reference = Tape::new_with_input("a", '#');
    reference.move_head(Direction::Right);
    assert_eq!(tape, reference);

    let mut tape = Tape::new_with_input("ab", '_');
    let before = tape.clone();
    tape.write('a');
    assert_eq!(tape, before);
}

#[test]
fn turing_machine_modify_input_cache_test() {
    let mut tm = TuringMachine::new('_');
    tm.add_state_with_id_label(0, "q0");
    tm.add_state_with_id_label(1, "qf");
    tm.add_transition(0, 1, "a;a/R".to_string());
    tm.make_initial(0);
    tm.make_final(1);
    assert!(tm.accepts("a"));
    assert!(!tm.accepts("b"));

    tm.modify_input(0, 1, "a;a/R", "b;b/R".to_string());
    assert!(tm.accepts("b"));
    assert!(!tm.accepts("a"));

    tm.remove_transition(0, 1, "b;b/R");
    assert!(!tm.accepts("b"));
}

#[test]
fn turing_machine_remove_state_cache_test() {
    let mut tm = turing('_', 3, &[2], &[(0, 1, "a;a/R"), (1, 2, "b;b/R")]);
    assert!(tm.accepts("ab"));
    tm.remove_state(1);
    assert!(!tm.accepts("ab"));
    tm.add_transition(0, 2, "a;a/S".to_string());
    assert!(tm.accepts("a"));
}

#[test]
fn tm_accepts_on_last_budgeted_step_test() {
    let tm = turing('_', 3, &[2], &[(0, 1, "a;a/R"), (1, 2, "b;b/R")]);
    assert_eq!(tm.run("ab", 2), Some(RunOutcome::Accepted));
    assert_eq!(tm.run_nondeterministic("ab", 2), Some(RunOutcome::Accepted));
    assert_eq!(tm.run("ab", 1), Some(RunOutcome::MaxStepsExceeded));
}

#[test]
fn tm_determinism_recovers_after_removal_test() {
    let mut tm = turing('_', 2, &[], &[(0, 1, "a;a/R,_;_/S"), (0, 0, "a;b/R,_;_/S")]);
    assert!(!tm.is_deterministic());
    tm.remove_transition(0, 0, "a;b/R,_;_/S");
    assert!(tm.is_deterministic());
    assert_eq!(tape_count(&tm), 2);
    tm.remove_transition(0, 1, "a;a/R,_;_/S");
    tm.add_transition(0, 1, "a;a/R".to_string());
    assert_eq!(tape_count(&tm), 1);
}

#[test]
fn tm_tape_count_follows_an_edited_label_test() {
    let mut tm = turing('_', 2, &[1], &[(0, 1, "broken")]);
    assert_eq!(tape_count(&tm), 1);
    tm.modify_input(0, 1, "broken", "a;a/R,_;_/S".to_string());
    assert_eq!(tape_count(&tm), 2, "the first well-formed label sets the tape count, even via modify_input");
    assert_eq!(tm.validate(), Ok(()));
    assert!(tm.accepts("a"));
}

#[test]
fn tm_tape_count_follows_a_valid_to_valid_edit_test() {
    let mut tm = turing('_', 2, &[1], &[(0, 1, "a;a/R")]);
    assert_eq!(tape_count(&tm), 1);
    tm.modify_input(0, 1, "a;a/R", "a;a/R,_;_/S".to_string());
    assert_eq!(tape_count(&tm), 2, "editing the sole transition to two tapes moves the machine to two tapes");
    assert_eq!(tm.validate(), Ok(()));
    assert!(tm.accepts("a"));
}
