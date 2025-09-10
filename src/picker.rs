use crate::{Result, state::State};
use rand::{SeedableRng as _, rngs::SmallRng, seq::IndexedRandom};

/// Picks the given number of tasks from chooseable tasks _and_ updates them.
pub fn pick_tasks(num_tasks: usize, state: &State) -> Result<Vec<String>> {
    let tasks: Vec<_> = state
        .tasks()
        .into_iter()
        .filter(|t| t.choosable(state))
        .collect();
    let mut rng = SmallRng::from_os_rng();
    Ok(tasks
        .choose_multiple_weighted(&mut rng, num_tasks, |t| t.weight())?
        .inspect(|t| t.choose(state))
        .map(|t| String::from(t.slug()))
        .collect())
}

/// Picks todays tasks, if needed. Returns `true` if any new tasks were added.
pub fn pick_todays_tasks(state: &mut State) -> Result<bool> {
    let num_tasks_to_generate = if state.todays_date() > state.last_generated_date() {
        state.todays_tasks_mut().clear();
        state.daily_tasks()
    } else {
        state.daily_tasks() - state.todays_tasks().len()
    };
    if num_tasks_to_generate > 0 {
        log::debug!("Picking {num_tasks_to_generate} new task(s)");
        let new_tasks = pick_tasks(num_tasks_to_generate, state)?;
        state.todays_tasks_mut().extend(new_tasks);
        state.mark_generated();
        Ok(true)
    } else {
        log::debug!("No new tasks to pick.");
        Ok(false)
    }
}
