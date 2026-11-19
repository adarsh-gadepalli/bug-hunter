//! The framework's random walk, run against the key-value cluster.

use bughunter::{Oracle, SystemModel, random_walk};
use kvstore::{Converged, KvModel, Merge};

const BUGGY: KvModel = KvModel {
    merge: Merge::Overwrite,
};
const FIXED: KvModel = KvModel { merge: Merge::Max };
const STEPS: usize = 300;

#[test]
fn same_seed_gives_the_same_walk() {
    let first = random_walk(&BUGGY, &Converged, 42, STEPS);
    let second = random_walk(&BUGGY, &Converged, 42, STEPS);
    assert_eq!(first.actions, second.actions);
    assert_eq!(first.bug, second.bug);
}

#[test]
fn some_seed_finds_the_planted_bug() {
    let walk = (0..20)
        .map(|seed| random_walk(&BUGGY, &Converged, seed, STEPS))
        .find(|walk| walk.bug.is_some())
        .expect("some seed in 0..20 should hit the overwrite bug");

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
