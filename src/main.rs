//! Random-walk experiment on the dummy cluster.

use std::process::ExitCode;

use bughunter::{SystemModel, SystemState, Walk, random_walk};
use system::{Action, Converged, DummyModel, Merge};

/// help text printed when the arguments can't be parsed.
const USAGE: &str = "\
usage:
  bug-hunter [--seed N] [--steps N] [--fixed]    print one walk step by step
  bug-hunter --seeds N [--steps N] [--fixed]     one summary line per seed 0..N

The default model has a planted bug: delivery overwrites instead of taking the max.
--fixed uses the correct merge.";

/// main function - runs the experiment and turns an error into a failing exit code.
fn main() -> ExitCode {
    // skip(1) drops the program name, leaving only the user's arguments.
    match run(&Args(std::env::args().skip(1).collect())) {
        Ok(()) => ExitCode::SUCCESS,
        // print the error and the usage to stderr, then exit with failure.
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

/// run function - builds the model from the flags, then runs one walk or a sweep over seeds.
fn run(args: &Args) -> Result<(), String> {
    // walk length, 300 unless --steps is given.
    let steps = args.number("--steps", 300)? as usize;
    // --fixed picks the correct merge; the default is the planted bug.
    let model = DummyModel {
        merge: if args.has("--fixed") {
            Merge::Max
        } else {
            Merge::Overwrite
        },
    };
    println!("merge={:?} max_steps={steps}", model.merge);

    if args.has("--seeds") {
        // sweep mode: one walk per seed in 0..seeds, one summary line each.
        let seeds = args.number("--seeds", 10)?;
        let mut found = 0;
        for seed in 0..seeds {
            let walk = random_walk(&model, &Converged, seed, steps);
            print_summary(&walk);
            // count the seeds whose walk hit the bug.
            found += u64::from(walk.bug.is_some());
        }
        println!("\n{found} of {seeds} seeds found the bug");
    } else {
        // single-walk mode: print every step, then the summary.
        let walk = random_walk(&model, &Converged, args.number("--seed", 0)?, steps);
        print_walk(&model, &walk);
        println!();
        print_summary(&walk);
    }
    Ok(())
}

/// print walk function - re-applies the walk's actions from the start so every state can be printed.
fn print_walk(model: &DummyModel, walk: &Walk<Action>) {
    let mut state = model.initial_state();
    // step 0 is the initial state, before any action.
    println!("{:>4}  {:<28}  {}", 0, "initial", state.summarize());
    for (index, action) in walk.actions.iter().enumerate() {
        // rebuild the next state. transition is deterministic, so this matches the walk.
        state = model.transition(&state, action);
        // format the action first; derived Debug ignores column width.
        let label = format!("{action:?}");
        println!("{:>4}  {label:<28}  {}", index + 1, state.summarize());
    }
}

/// print summary function - one line with the seed, walk length, distinct states, and outcome.
fn print_summary(walk: &Walk<Action>) {
    let outcome = match &walk.bug {
        Some(bug) => format!("BUG at step {} ({}): {}", bug.step, bug.oracle, bug.reason),
        None => "no bug".to_string(),
    };
    println!(
        "seed {:>3}  steps {:>4}  unique states {:>4}  {outcome}",
        walk.seed,
        walk.actions.len(),
        walk.unique_states
    );
}

/// args struct - the command-line arguments after the program name.
struct Args(Vec<String>);

impl Args {
    /// has function - true if the flag appears anywhere in the arguments.
    fn has(&self, name: &str) -> bool {
        self.0.iter().any(|arg| arg == name)
    }

    /// number function - the number after `name`, or `default` if the flag is missing.
    fn number(&self, name: &str, default: u64) -> Result<u64, String> {
        // flag not given: use the default.
        let Some(index) = self.0.iter().position(|arg| arg == name) else {
            return Ok(default);
        };
        // flag given with nothing after it.
        let text = self
            .0
            .get(index + 1)
            .ok_or_else(|| format!("{name} expects a number"))?;
        // flag given with something that isn't a number.
        text.parse()
            .map_err(|_| format!("{name} expects a number, got {text:?}"))
    }
}