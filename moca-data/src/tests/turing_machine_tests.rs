use crate::state_machine::{Machine, StateMachine};
use crate::turing_machine::{
    Configuration, RunOutcome, TuringMachine, DEFAULT_MAX_STEPS,
};

/* Builds the classic a^n b^n (n >= 0) decider over blank '_':
 * - q0: mark an unmarked 'a' as X and look for a b; skip X's; once only Y's
 *   remain go check the tail; accept on blank.
 * - q1: scan right over a's/Y's to the first unmarked b, mark it Y.
 * - q2: sweep left back to the leftmost X.
 * - q3: verify everything left is Y, accept on blank. */
fn build_anbn_machine() -> TuringMachine {
    let mut tm = TuringMachine::new('_');
    assert_eq!(tm.get_blank_symbol(), '_');
    tm.add_n_states(5);
    tm.make_initial(0);
    tm.make_final(4);
    // q0
    tm.add_transition(0, 1, "a;X/R".to_string());
    tm.add_transition(0, 0, "X;X/R".to_string());
    tm.add_transition(0, 3, "Y;Y/R".to_string());
    tm.add_transition(0, 4, "_;_/S".to_string());
    // q1
    tm.add_transition(1, 1, "a;a/R".to_string());
    tm.add_transition(1, 1, "Y;Y/R".to_string());
    tm.add_transition(1, 2, "b;Y/L".to_string());
    // q2
    tm.add_transition(2, 2, "a;a/L".to_string());
    tm.add_transition(2, 2, "Y;Y/L".to_string());
    tm.add_transition(2, 0, "X;X/R".to_string());
    // q3
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

/* Even number of 'a's: sweep right toggling states, then halt on blank in a
 * dedicated accepting (even) or rejecting (odd) state. */
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
    // Marks each input symbol while moving right (a->X, b->Y, c->Z), steps
    // left on blank, then stays and accepts on Z.
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(3);
    tm.make_initial(0);
    tm.make_final(2);
    tm.add_transition(0, 0, "a;X/R".to_string());
    tm.add_transition(0, 0, "b;Y/R".to_string());
    tm.add_transition(0, 0, "c;Z/R".to_string());
    tm.add_transition(0, 1, "_;_/L".to_string());
    tm.add_transition(1, 2, "Z;Z/S".to_string());

    // Manual step-by-step trace over "abc".
    let mut config = tm.initial_configuration("abc").unwrap();
    assert_eq!(config.state_id(), 0);
    assert_eq!(config.tape().read(), 'a');

    config = tm.step(&config).unwrap(); // a -> X, move right
    assert_eq!(config.state_id(), 0);
    assert_eq!(config.tape().head_position(), 1);
    assert_eq!(config.tape().read(), 'b');

    config = tm.step(&config).unwrap(); // b -> Y, move right
    assert_eq!(config.tape().head_position(), 2);

    config = tm.step(&config).unwrap(); // c -> Z, move right
    assert_eq!(config.tape().head_position(), 3);
    assert_eq!(config.tape().read(), '_'); // blank cell

    config = tm.step(&config).unwrap(); // blank -> step left onto Z
    assert_eq!(config.state_id(), 1);
    assert_eq!(config.tape().head_position(), 2);

    config = tm.step(&config).unwrap(); // stay and accept
    assert_eq!(config.state_id(), 2);
    assert_eq!(
        tm.run("abc", DEFAULT_MAX_STEPS),
        Some(RunOutcome::Accepted)
    );
    // The tape was rewritten by the run.
    assert_ne!(tm.initial_configuration("abc").unwrap().tape().read(), 'X');

    // Halting without any applicable transition rejects.
    assert_eq!(
        tm.run("abd", DEFAULT_MAX_STEPS),
        Some(RunOutcome::Rejected)
    );

    // Tape mechanics directly: negative positions via L moves and blanks.
    let mut tape = crate::turing_machine::Tape::new_with_input("ab", '#');
    assert_eq!(tape.get_blank(), '#');
    assert_eq!(tape.read(), 'a');
    tape.move_head(crate::turing_machine::Direction::Right);
    assert_eq!(tape.read(), 'b');
    tape.write('Z');
    assert_eq!(tape.read(), 'Z');
    tape.move_head(crate::turing_machine::Direction::Left);
    tape.move_head(crate::turing_machine::Direction::Left);
    assert_eq!(tape.read(), '#'); // one cell left of the input
    tape.move_head(crate::turing_machine::Direction::Left);
    assert_eq!(tape.head_position(), -2);

    // Snapshot renders blanks outside the used region and locates the head:
    // window covers [-4, 0], so the head at -2 sits at offset 2.
    let (window, head_offset) = tape.snapshot(2);
    assert_eq!(window.chars().count(), 5);
    assert_eq!(head_offset, 2);
    assert_eq!(window.chars().last(), Some('a')); // cell 0 still holds 'a'
}

#[test]
fn turing_run_max_steps_exceeded_test() {
    // Moves right forever over blanks: never halts.
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(1);
    tm.make_initial(0);
    tm.add_transition(0, 0, "_;_/R".to_string());
    assert_eq!(
        tm.run("", 1000),
        Some(RunOutcome::MaxStepsExceeded)
    );
    // The trait-level accepts() uses the default budget and must not report
    // acceptance for a looping machine.
    assert_eq!(tm.accepts(""), false);
}

#[test]
fn turing_validate_test() {
    // Well-formed machine passes.
    let tm = build_anbn_machine();
    assert!(tm.validate().is_ok());

    // Missing initial state.
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(1);
    assert!(tm.validate().is_err());

    // Malformed labels of every flavor.
    for bad_label in [
        "ab;c/d",   // multi-character read
        "a;bc/d",   // multi-character write
        "a;b/QR",   // unknown direction
        "a;b",      // missing direction
        "a",        // single field
        "a;b/c/d",  // extra direction field
        "",         // empty label
        "ε;b/c",    // ε is not a tape symbol
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
        // ...and the engine skips malformed labels instead of crashing.
        let outcome = tm.run("anything", 100);
        assert_eq!(outcome, Some(RunOutcome::Rejected));
    }

    // Dangling transition target.
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

    // Same read symbol, different target: ambiguous machine.
    tm.add_transition(0, 0, "a;b/S".to_string());
    assert_eq!(tm.is_deterministic(), false);

    // step refuses to pick a branch when ambiguous...
    let config = tm.initial_configuration("a").unwrap();
    assert_eq!(tm.step(&config), None);

    // ...while identical duplicates do not break determinism.
    // Entering the final state accepts immediately (arrival semantics), so
    // "a" is accepted without needing a halting transition.
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(2);
    tm.make_initial(0);
    tm.make_final(1);
    tm.add_transition(0, 1, "a;a/R".to_string());
    tm.add_transition(0, 1, "a;a/R".to_string());
    assert_eq!(tm.is_deterministic(), true);
    assert_eq!(tm.run("a", 10), Some(RunOutcome::Accepted));

    // Halting in a NON-accepting state rejects: same shape but no finals.
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(2);
    tm.make_initial(0);
    tm.add_transition(0, 1, "a;a/R".to_string());
    assert_eq!(tm.run("a", 10), Some(RunOutcome::Rejected));
}

#[test]
fn turing_configuration_helpers_test() {
    let tm = build_anbn_machine();
    // Without an initial state there is no configuration to start from.
    let mut headless = TuringMachine::new('_');
    headless.add_n_states(1);
    assert_eq!(headless.initial_configuration("ab"), None);
    assert_eq!(headless.run("ab", 10), None);

    // The initial configuration places the input at positions 0..n under a
    // head at 0.
    let config: Configuration = tm.initial_configuration("ab").unwrap();
    assert_eq!(config.state_id(), 0);
    assert_eq!(config.tape().read(), 'a');

    // Removing the initial state later is reported by validate (forget_state).
    let mut tm = build_anbn_machine();
    tm.remove_state(0);
    assert!(tm.validate().is_err());
}

/* ---------- Nondeterministic machines ---------- */

/* L = (even number of a's) UNION (contains "bb"), over blank '_'.
 * The initial state branches on every real symbol into two parallel
 * strategies; the first symbol is accounted for by entering the parity
 * counter in the right state. An empty input goes straight to accept. */
fn build_union_machine() -> TuringMachine {
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(7);
    // 0: start, 1/2: odd/even parity sweep, 3/4: bb scan, 5/6: accept/reject halts
    tm.make_initial(0);
    tm.make_final(5);
    // Branching: every read symbol leads to BOTH strategies.
    tm.add_transition(0, 1, "a;a/R".to_string()); // first 'a': parity odd
    tm.add_transition(0, 2, "b;b/R".to_string()); // first 'b': parity even
    tm.add_transition(0, 3, "a;a/R".to_string()); // bb-scan strategy
    tm.add_transition(0, 3, "b;b/R".to_string());
    tm.add_transition(0, 5, "_;_/S".to_string()); // empty input is even
    // Parity branch.
    tm.add_transition(1, 2, "a;a/R".to_string());
    tm.add_transition(1, 1, "b;b/R".to_string());
    tm.add_transition(2, 1, "a;a/R".to_string());
    tm.add_transition(2, 2, "b;b/R".to_string());
    tm.add_transition(1, 6, "_;_/S".to_string());
    tm.add_transition(2, 5, "_;_/S".to_string());
    // Contains-"bb" branch.
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
        ("", true),      // even (0 a's)
        ("aa", true),    // even
        ("abba", true),  // contains bb
        ("abb", true),   // odd, but contains bb
        ("bb", true),    // contains bb (and 0 a's is even anyway)
        ("b", true),     // 0 a's is even
        ("aab", true),   // even
        ("bab", false),  // odd, no bb
        ("ab", false),   // odd, no bb
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
    // On "abb": the parity branch runs to the end of the tape and rejects,
    // while the bb branch accepts earlier. Mixed-depth branches must not
    // confuse the level-by-level exploration.
    let tm = build_union_machine();
    assert_eq!(
        tm.run_nondeterministic("abb", 1000),
        Some(RunOutcome::Accepted)
    );
    // "ab" has an odd number of a's and no bb: every branch dies rejecting.
    assert_eq!(
        tm.run_nondeterministic("ab", 1000),
        Some(RunOutcome::Rejected)
    );
}

#[test]
fn turing_step_all_branching_test() {
    // Two transitions on 'a': step_all yields both branches, step refuses.
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

    // Both stay on the cell they wrote over, so the scanned symbol is the
    // freshly written one.
    let writes: Vec<char> = branches.iter().map(|c| c.tape().read()).collect();
    assert!(writes.contains(&'X'));
    assert!(writes.contains(&'Y'));

    assert_eq!(tm.step(&config), None);

    // A deterministic machine yields exactly one branch.
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
    // One branch loops forever moving right; the other walks off the input
    // and halts without accepting. No branch accepts, but one never halts,
    // so the outcome is MaxStepsExceeded rather than Rejected.
    let mut tm = TuringMachine::new('_');
    tm.add_n_states(4);
    tm.make_initial(0);
    tm.make_final(3);
    // Two same-read rules from state 0: this is what makes it
    // nondeterministic.
    tm.add_transition(0, 1, "a;a/R".to_string()); // looping strategy
    tm.add_transition(0, 2, "a;a/R".to_string()); // halting strategy
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
    // Both branches halt without accepting on every input: Rejected, not
    // MaxStepsExceeded.
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

/* ---------- Multitape machines ---------- */

/* Two-tape machine deciding {a*}: it sweeps the input on tape 1 while
 * dropping one '*' per 'a' on tape 2, accepting when both heads sit on
 * blanks. Exercises simultaneous reads/writes across tapes. */
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

    // Manual trace over "aa": both heads advance together, tape 2 accumulates
    // the stars.
    let mut config = tm.initial_configuration("aa").unwrap();
    assert_eq!(config.tapes().len(), 2);
    assert_eq!(config.tapes()[0].read(), 'a');
    assert_eq!(config.tapes()[1].read(), '_');

    config = tm.step(&config).unwrap();
    assert_eq!(config.tapes()[0].head_position(), 1);
    assert_eq!(config.tapes()[1].head_position(), 1);
    // The star was written one cell behind the moved head.
    let (window, _) = config.tapes()[1].snapshot(1);
    assert_eq!(window.chars().next(), Some('*'));

    config = tm.step(&config).unwrap();
    assert_eq!(config.tapes()[0].head_position(), 2);

    // Third step reads blanks on both tapes and moves into the accept state.
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
    // A single-segment label does not match the established two-tape shape:
    // stored, flagged by validate, skipped by the engine.
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
    // Two transitions reading the same pair of symbols ('a', '_'): one
    // accepts immediately, the other dies.
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
    // No transition reads 'b': every branch halts immediately.
    assert_eq!(
        tm.run_nondeterministic("b", 100),
        Some(RunOutcome::Rejected)
    );
}

#[test]
fn turing_tape_canonical_form_test() {
    use crate::turing_machine::{Direction, Tape};
    // Writing the blank erases the cell instead of materializing it, so a
    // tape with an erased cell compares equal to a never-written one.
    let mut tape = Tape::new_with_input("ax", '#');
    tape.move_head(Direction::Right);
    tape.write('X');
    assert_eq!(tape.read(), 'X');
    tape.write('#');
    let mut reference = Tape::new_with_input("a", '#');
    reference.move_head(Direction::Right);
    assert_eq!(tape, reference);

    // Rewriting the same symbol changes nothing.
    let mut tape = Tape::new_with_input("ab", '_');
    let before = tape.clone();
    tape.write('a');
    assert_eq!(tape, before);
}

/* The stepping engine serves transitions from a pre-parsed table; these
 * tests pin the table's coherence across in-place label edits, transition
 * removals and state deletions performed between runs. */
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

    // Editing the label in place must be reflected by the engine.
    tm.modify_input(0, 1, "a;a/R", "b;b/R".to_string());
    assert!(tm.accepts("b"));
    assert!(!tm.accepts("a"));

    // Removing the transition halts the machine immediately.
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

    // The removed state must vanish from every cached entry, including as
    // a transition target of other states.
    tm.remove_state(1);
    assert!(!tm.accepts("ab"));

    // Re-wiring around the removed state works again.
    tm.add_transition(0, 2, "a;a/S".to_string());
    assert!(tm.accepts("a"));
}
