//! The framework's random walk, run against the dummy cluster.

use bughunter::{Oracle, SystemModel, random_walk};
use system::{Action, Converged, DummyModel, Merge};

const BUGGY: DummyModel = DummyModel {
    merge: Merge::Overwrite,
};
const FIXED: DummyModel = DummyModel { merge: Merge::Max };
const STEPS: usize = 300;

#[test]
fn same_seed_gives_the_same_walk() {
    let first = random_walk(&BUGGY, &Converged, 42, STEPS);
    let second = random_walk(&BUGGY, &Converged, 42, STEPS);
    assert_eq!(first.actions, second.actions);
    assert_eq!(first.bug, second.bug);
}

#[test]
fn different_seeds_give_different_walks() {
    let first = random_walk(&BUGGY, &Converged, 1, 20);
    let second = random_walk(&BUGGY, &Converged, 2, 20);
    assert_ne!(first.actions, second.actions);
}

#[test]
fn some_seed_finds_the_planted_bug() {
    let walk = (0..20)
        .map(|seed| random_walk(&BUGGY, &Converged, seed, STEPS))
        .find(|walk| walk.bug.is_some())
        .expect("some seed in 0..20 should hit the overwrite bug");

    let bug = walk.bug.as_ref().unwrap();
    assert_eq!(bug.step, walk.actions.len());

    let mut state = BUGGY.initial_state();
    for action in &walk.actions {
        state = BUGGY.transition(&state, action);
    }
    assert!(Converged.check(&state).is_some());
}

#[test]
fn no_seed_finds_a_bug_when_merge_is_max() {
    for seed in 0..20 {
        let walk = random_walk(&FIXED, &Converged, seed, STEPS);
        assert!(walk.bug.is_none(), "seed {seed} reported a bug");
        assert_eq!(walk.actions.len(), STEPS);
    }
}

#[test]
fn a_walk_starts_with_a_write() {
    let walk = random_walk(&BUGGY, &Converged, 7, 1);
    assert!(matches!(walk.actions[..], [Action::Write { .. }]));
}
