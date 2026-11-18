//! Random walk: pick one legal action at random, apply it, check the
//! invariant, and repeat until a bug or the step limit.

use std::collections::HashSet;

use crate::rng::Rng;
use crate::{BugReport, Oracle, SystemModel, SystemState};

pub struct Walk<A> {
    pub seed: u64,
    // a list of actions taken, in order.
    pub actions: Vec<A>,
    // a bug report if a violation was found.
    pub bug: Option<BugReport>,
    // the number of distinct states seen, including the initial state.
    pub unique_states: usize,
}


// the random walk function
pub fn random_walk<M: SystemModel>(
    model: &M,
    oracle: &dyn Oracle<M::State>,
    seed: u64,
    max_steps: usize,
) -> Walk<M::Action> {
    let mut rng = Rng::new(seed);
    let mut state = model.initial_state();
    let mut seen = HashSet::from([state.fingerprint()]);
    let mut actions = Vec::new();
    let mut bug = check(oracle, &state, 0);

    // while no bug and within threshold, select an action
    while bug.is_none() && actions.len() < max_steps {
        let mut choices = model.get_actions(&state);
        if choices.is_empty() {
            break;
        }
        // select an action at random
        let action = choices.swap_remove(rng.below(choices.len()));
        state = model.transition(&state, &action);
        // add the new state to the set of seen states
        seen.insert(state.fingerprint());
        // add the action to the list of actions
        actions.push(action);
        // check if the state violates the invariant
        bug = check(oracle, &state, actions.len());
    }

    // return the walk result.
    Walk {
        seed,
        actions,
        bug,
        unique_states: seen.len(),
    }
}

fn check<S>(oracle: &dyn Oracle<S>, state: &S, step: usize) -> Option<BugReport> {
    oracle.check(state).map(|reason| BugReport {
        oracle: oracle.name().to_string(),
        reason,
        step,
    })
}
