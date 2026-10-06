use crate::state_machine::{Machine, StateMachine};
use crate::turing_machine::{
    Configuration, RunOutcome, TuringMachine, DEFAULT_MAX_STEPS,
};

fn build_anbn_machine() -> TuringMachine {
    let mut tm = TuringMachine::new('_');
    assert_eq!(tm.get_blank_symbol(), '_');
    tm.add_n_states(5);
    tm.make_initial(0);
    tm.make_final(4);
    tm.add_transition(0, 1, "a;X/R".to_string());
    tm.add_transition(0, 0, "X;X/R".to_string());
    tm.add_transition(0, 3, "Y;Y/R".to_string());
    tm.add_transition(0, 4, "_;_/S".to_string());
    tm.add_transition(1, 1, "a;a/R".to_string());
    tm.add_transition(1, 1, "Y;Y/R".to_string());
    tm.add_transition(1, 2, "b;Y/L".to_string());
    tm.add_transition(2, 2, "a;a/L".to_string());
    tm.add_transition(2, 2, "Y;Y/L".to_string());
    tm.add_transition(2, 0, "X;X/R".to_string());
    tm.add_transition(3, 3, "Y;Y/R".to_string());
    tm.add_transition(3, 4, "_;_/S".to_string());
    tm
}

#[test]
fn turing_anbn_decider_test() {
    let tm = build_anbn_machine();
    assert_eq!(tm.kind(), crate::state_machine::MachineKind::Turing);
    assert!(tm.validate().is_ok());

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
        assert_eq!(
            tm.accepts(input),
            expected,
            "wrong answer for input {:?}",
            input
        );
        let outcome = tm.run(input, DEFAULT_MAX_STEPS).unwrap();
        let expected_outcome = if expected { RunOutcome::Accepted } else { RunOutcome::Rejected };
        assert_eq!(outcome, expected_outcome, "run outcome differs for {:?}", input);
    }
}

fn build_parity_machine() -> TuringMachine {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(4);
    tm.make_initial(0);
    tm.make_final(2);
    tm.add_transition(0, 1, "a;a/R".to_string());
    tm.add_transition(1, 0, "a;a/R".to_string());
    tm.add_transition(0, 2, "_;_/S".to_string());
    tm.add_transition(1, 3, "_;_/S".to_string());
    tm
}

#[test]
fn turing_parity_machine_test() {
    let tm = build_parity_machine();
    assert_eq!(tm.run("", DEFAULT_MAX_STEPS), Some(RunOutcome::Accepted));
    assert_eq!(tm.run("aaaa", DEFAULT_MAX_STEPS), Some(RunOutcome::Accepted));
    assert_eq!(tm.run("a", DEFAULT_MAX_STEPS), Some(RunOutcome::Rejected));
    assert_eq!(tm.run("aaaba", DEFAULT_MAX_STEPS), Some(RunOutcome::Rejected));
}

#[test]
fn turing_step_trace_and_tape_test() {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(3);
    tm.make_initial(0);
    tm.make_final(2);
    tm.add_transition(0, 0, "a;X/R".to_string());
    tm.add_transition(0, 0, "b;Y/R".to_string());
    tm.add_transition(0, 0, "c;Z/R".to_string());
    tm.add_transition(0, 1, "_;_/L".to_string());
    tm.add_transition(1, 2, "Z;Z/S".to_string());

    let mut config = tm.initial_configuration("abc").unwrap();
    assert_eq!(config.state_id(), 0);
    assert_eq!(config.tape().read(), 'a');

    config = tm.step(&config).unwrap();
    assert_eq!(config.state_id(), 0);
    assert_eq!(config.tape().head_position(), 1);
    assert_eq!(config.tape().read(), 'b');

    config = tm.step(&config).unwrap();
    assert_eq!(config.tape().head_position(), 2);

    config = tm.step(&config).unwrap();
    assert_eq!(config.tape().head_position(), 3);
    assert_eq!(config.tape().read(), '_');

    config = tm.step(&config).unwrap();
    assert_eq!(config.state_id(), 1);
    assert_eq!(config.tape().head_position(), 2);

    config = tm.step(&config).unwrap();
    assert_eq!(config.state_id(), 2);
    assert_eq!(
        tm.run("abc", DEFAULT_MAX_STEPS),
        Some(RunOutcome::Accepted)
    );
    assert_ne!(tm.initial_configuration("abc").unwrap().tape().read(), 'X');

    assert_eq!(
        tm.run("abd", DEFAULT_MAX_STEPS),
        Some(RunOutcome::Rejected)
    );

    let mut tape = crate::turing_machine::Tape::new_with_input("ab", '#');
    assert_eq!(tape.get_blank(), '#');
    assert_eq!(tape.read(), 'a');
    tape.move_head(crate::turing_machine::Direction::Right);
    assert_eq!(tape.read(), 'b');
    tape.write('Z');
    assert_eq!(tape.read(), 'Z');
    tape.move_head(crate::turing_machine::Direction::Left);
    tape.move_head(crate::turing_machine::Direction::Left);
    assert_eq!(tape.read(), '#');
    tape.move_head(crate::turing_machine::Direction::Left);
    assert_eq!(tape.head_position(), -2);

    let (window, head_offset) = tape.snapshot(2);
    assert_eq!(window.chars().count(), 5);
    assert_eq!(head_offset, 2);
    assert_eq!(window.chars().last(), Some('a'));
}

#[test]
fn turing_run_max_steps_exceeded_test() {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(1);
    tm.make_initial(0);
    tm.add_transition(0, 0, "_;_/R".to_string());
    assert_eq!(
        tm.run("", 1000),
        Some(RunOutcome::MaxStepsExceeded)
    );
    assert_eq!(tm.accepts(""), false);
}

#[test]
fn turing_validate_test() {
    let tm = build_anbn_machine();
    assert!(tm.validate().is_ok());

    let mut tm = TuringMachine::new('_');
    tm.add_n_states(1);
    assert!(tm.validate().is_err());

    for bad_label in [
        "ab;c/d",
        "a;bc/d",
        "a;b/QR",
        "a;b",
        "a",
        "a;b/c/d",
        "",
        "ε;b/c",
    ] {
        let mut tm = TuringMachine::new('_');
        tm.add_n_states(2);
        tm.make_initial(0);
        tm.make_final(1);
        tm.add_transition(0, 1, bad_label.to_string());
        assert!(
            Machine::validate(&tm).is_err(),
            "label {:?} must not validate",
            bad_label
        );
        let outcome = tm.run("anything", 100);
        assert_eq!(outcome, Some(RunOutcome::Rejected));
    }

    let mut tm = TuringMachine::new('_');
    tm.add_n_states(2);
    tm.make_initial(0);
    tm.add_transition(0, 1, "_;_/S".to_string());
    tm.get_states_by_id_mut_ref().remove(&1);
    assert!(tm.validate().is_err());
}

#[test]
fn turing_nondeterminism_flag_test() {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(2);
    tm.make_initial(0);
    tm.add_transition(0, 1, "a;a/R".to_string());
    assert_eq!(tm.is_deterministic(), true);

    tm.add_transition(0, 0, "a;b/S".to_string());
    assert_eq!(tm.is_deterministic(), false);

    let config = tm.initial_configuration("a").unwrap();
    assert_eq!(tm.step(&config), None);

    let mut tm = TuringMachine::new('_');
    tm.add_n_states(2);
    tm.make_initial(0);
    tm.make_final(1);
    tm.add_transition(0, 1, "a;a/R".to_string());
    tm.add_transition(0, 1, "a;a/R".to_string());
    assert_eq!(tm.is_deterministic(), true);
    assert_eq!(tm.run("a", 10), Some(RunOutcome::Accepted));

    let mut tm = TuringMachine::new('_');
    tm.add_n_states(2);
    tm.make_initial(0);
    tm.add_transition(0, 1, "a;a/R".to_string());
    assert_eq!(tm.run("a", 10), Some(RunOutcome::Rejected));
}

#[test]
fn turing_configuration_helpers_test() {
    let tm = build_anbn_machine();
    let mut headless = TuringMachine::new('_');
    headless.add_n_states(1);
    assert_eq!(headless.initial_configuration("ab"), None);
    assert_eq!(headless.run("ab", 10), None);

    let config: Configuration = tm.initial_configuration("ab").unwrap();
    assert_eq!(config.state_id(), 0);
    assert_eq!(config.tape().read(), 'a');

    let mut tm = build_anbn_machine();
    tm.remove_state(0);
    assert!(tm.validate().is_err());
}

fn build_union_machine() -> TuringMachine {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(7);
    tm.make_initial(0);
    tm.make_final(5);
    tm.add_transition(0, 1, "a;a/R".to_string());
    tm.add_transition(0, 2, "b;b/R".to_string());
    tm.add_transition(0, 3, "a;a/R".to_string());
    tm.add_transition(0, 3, "b;b/R".to_string());
    tm.add_transition(0, 5, "_;_/S".to_string());
    tm.add_transition(1, 2, "a;a/R".to_string());
    tm.add_transition(1, 1, "b;b/R".to_string());
    tm.add_transition(2, 1, "a;a/R".to_string());
    tm.add_transition(2, 2, "b;b/R".to_string());
    tm.add_transition(1, 6, "_;_/S".to_string());
    tm.add_transition(2, 5, "_;_/S".to_string());
    tm.add_transition(3, 3, "a;a/R".to_string());
    tm.add_transition(3, 4, "b;b/R".to_string());
    tm.add_transition(4, 3, "a;a/R".to_string());
    tm.add_transition(4, 5, "b;b/S".to_string());
    tm.add_transition(3, 6, "_;_/S".to_string());
    tm.add_transition(4, 6, "_;_/S".to_string());
    tm
}

#[test]
fn turing_nondeterministic_union_language_test() {
    let tm = build_union_machine();
    assert_eq!(tm.is_deterministic(), false);
    assert!(tm.validate().is_ok());

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
            "wrong outcome for {:?}",
            input
        );
        assert_eq!(
            tm.accepts(input),
            expected,
            "accepts() must dispatch to the nondeterministic run for {:?}",
            input
        );
    }
}

#[test]
fn turing_nondeterministic_branch_depths_test() {
    let tm = build_union_machine();
    assert_eq!(
        tm.run_nondeterministic("abb", 1000),
        Some(RunOutcome::Accepted)
    );
    assert_eq!(
        tm.run_nondeterministic("ab", 1000),
        Some(RunOutcome::Rejected)
    );
}

#[test]
fn turing_step_all_branching_test() {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(3);
    tm.make_initial(0);
    tm.add_transition(0, 1, "a;X/S".to_string());
    tm.add_transition(0, 2, "a;Y/S".to_string());

    let config = tm.initial_configuration("a").unwrap();
    let branches = tm.step_all(&config);
    assert_eq!(branches.len(), 2);

    let targets: Vec<u64> = branches.iter().map(|c| c.state_id()).collect();
    assert!(targets.contains(&1));
    assert!(targets.contains(&2));

    let writes: Vec<char> = branches.iter().map(|c| c.tape().read()).collect();
    assert!(writes.contains(&'X'));
    assert!(writes.contains(&'Y'));

    assert_eq!(tm.step(&config), None);

    let mut det_tm = TuringMachine::new('_');
    det_tm.add_n_states(2);
    det_tm.make_initial(0);
    det_tm.add_transition(0, 1, "a;X/S".to_string());
    let config = det_tm.initial_configuration("a").unwrap();
    assert_eq!(det_tm.step_all(&config).len(), 1);
    assert!(det_tm.step(&config).is_some());
}

#[test]
fn turing_nondeterministic_looping_branch_test() {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(4);
    tm.make_initial(0);
    tm.make_final(3);
    tm.add_transition(0, 1, "a;a/R".to_string());
    tm.add_transition(0, 2, "a;a/R".to_string());
    tm.add_transition(1, 1, "a;a/R".to_string());
    tm.add_transition(1, 1, "_;_/R".to_string());
    tm.add_transition(2, 2, "a;a/R".to_string());
    tm.add_transition(0, 3, "z;z/S".to_string());
    assert_eq!(tm.is_deterministic(), false);
    assert_eq!(
        tm.run_nondeterministic("aaaa", 500),
        Some(RunOutcome::MaxStepsExceeded)
    );
    assert_eq!(
        tm.run_nondeterministic("z", 10),
        Some(RunOutcome::Accepted)
    );
}

#[test]
fn turing_nondeterministic_all_reject_test() {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(4);
    tm.make_initial(0);
    tm.add_transition(0, 1, "a;a/R".to_string());
    tm.add_transition(0, 2, "a;a/R".to_string());
    tm.add_transition(1, 3, "_;_/S".to_string());
    tm.add_transition(2, 3, "_;_/S".to_string());
    assert_eq!(
        tm.run_nondeterministic("a", 100),
        Some(RunOutcome::Rejected)
    );
}

fn build_marker_machine() -> TuringMachine {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(2);
    tm.make_initial(0);
    tm.make_final(1);
    tm.add_transition(0, 0, "a;a/R,_;*/R".to_string());
    tm.add_transition(0, 1, "_;_/S,_;_/S".to_string());
    tm
}

#[test]
fn turing_multitape_machine_test() {
    let tm = build_marker_machine();
    assert_eq!(tm.get_tape_count(), 2);
    assert!(tm.validate().is_ok());

    let mut config = tm.initial_configuration("aa").unwrap();
    assert_eq!(config.tapes().len(), 2);
    assert_eq!(config.tapes()[0].read(), 'a');
    assert_eq!(config.tapes()[1].read(), '_');

    config = tm.step(&config).unwrap();
    assert_eq!(config.tapes()[0].head_position(), 1);
    assert_eq!(config.tapes()[1].head_position(), 1);
    let (window, _) = config.tapes()[1].snapshot(1);
    assert_eq!(window.chars().next(), Some('*'));

    config = tm.step(&config).unwrap();
    assert_eq!(config.tapes()[0].head_position(), 2);

    config = tm.step(&config).unwrap();
    assert_eq!(config.tapes()[0].read(), '_');
    assert_eq!(config.tapes()[1].read(), '_');
    assert_eq!(config.state_id(), 1);

    assert_eq!(tm.run("aaa", DEFAULT_MAX_STEPS), Some(RunOutcome::Accepted));
    assert_eq!(tm.run("", DEFAULT_MAX_STEPS), Some(RunOutcome::Accepted));
    assert_eq!(
        tm.run("aab", DEFAULT_MAX_STEPS),
        Some(RunOutcome::Rejected)
    );
    assert!(tm.accepts("aaaa"));
    assert!(!tm.accepts("aba"));
}

#[test]
fn turing_multitape_inconsistent_label_test() {
    let mut tm = build_marker_machine();
    tm.add_transition(0, 0, "z;z/S".to_string());
    assert!(Machine::validate(&tm).is_err());
    assert_eq!(
        tm.run("z", 100),
        Some(RunOutcome::Rejected)
    );
}

#[test]
fn turing_multitape_nondeterministic_test() {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(3);
    tm.make_initial(0);
    tm.make_final(1);
    tm.add_transition(0, 1, "a;a/S,_;_/S".to_string());
    tm.add_transition(0, 2, "a;a/S,_;_/S".to_string());
    assert_eq!(tm.is_deterministic(), false);

    let config = tm.initial_configuration("ab").unwrap();
    assert_eq!(tm.step_all(&config).len(), 2);
    assert_eq!(tm.step(&config), None);

    assert_eq!(
        tm.run_nondeterministic("ab", 100),
        Some(RunOutcome::Accepted)
    );
    assert_eq!(
        tm.run_nondeterministic("b", 100),
        Some(RunOutcome::Rejected)
    );
}

#[test]
fn turing_tape_canonical_form_test() {
    use crate::turing_machine::{Direction, Tape};
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
    let mut tm = TuringMachine::new('_');
    tm.add_state_with_id_label(0, "q0");
    tm.add_state_with_id_label(1, "q1");
    tm.add_state_with_id_label(2, "qf");
    tm.add_transition(0, 1, "a;a/R".to_string());
    tm.add_transition(1, 2, "b;b/R".to_string());
    tm.make_initial(0);
    tm.make_final(2);
    assert!(tm.accepts("ab"));

    tm.remove_state(1);
    assert!(!tm.accepts("ab"));

    tm.add_transition(0, 2, "a;a/S".to_string());
    assert!(tm.accepts("a"));
}

#[test]
fn tm_accepts_on_last_budgeted_step_test() {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(3);
    tm.make_initial(0);
    tm.make_final(2);
    tm.add_transition(0, 1, "a;a/R".to_string());
    tm.add_transition(1, 2, "b;b/R".to_string());
    assert_eq!(tm.run("ab", 2), Some(RunOutcome::Accepted));
    assert_eq!(tm.run_nondeterministic("ab", 2), Some(RunOutcome::Accepted));
    assert_eq!(tm.run("ab", 1), Some(RunOutcome::MaxStepsExceeded));
}

#[test]
fn tm_determinism_recovers_after_removal_test() {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(2);
    tm.make_initial(0);
    tm.add_transition(0, 1, "a;a/R,_;_/S".to_string());
    tm.add_transition(0, 0, "a;b/R,_;_/S".to_string());
    assert!(!tm.is_deterministic());
    tm.remove_transition(0, 0, "a;b/R,_;_/S");
    assert!(tm.is_deterministic());
    assert_eq!(tm.get_tape_count(), 2);
    tm.remove_transition(0, 1, "a;a/R,_;_/S");
    tm.add_transition(0, 1, "a;a/R".to_string());
    assert_eq!(tm.get_tape_count(), 1);
}
