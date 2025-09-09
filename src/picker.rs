use crate::{Error, Result, state::State, task::Task};
use rand::{SeedableRng as _, rngs::SmallRng, seq::IndexedRandom};

pub fn pick_tasks(num_tasks: usize, state: &State) -> Result<Vec<String>> {
    let tasks: Vec<_> = state
        .tasks()
        .into_iter()
        .filter(|t| t.choosable(state))
        .collect();
    let mut rng = SmallRng::from_os_rng();
    Ok(tasks
        .choose_multiple_weighted(&mut rng, num_tasks, |t| t.weight())?
        .map(|t| String::from(t.slug()))
        .collect())
}

pub fn pick_todays_tasks(state: &mut State) -> Result<()> {
    let num_tasks_to_generate = if state.last_generated_date() > state.todays_date() {
        state.todays_tasks_mut().clear();
        state.todays_tasks().len()
    } else {
        state.daily_tasks() - state.todays_tasks().len()
    };
    if num_tasks_to_generate > 0 {
        let new_tasks = pick_tasks(num_tasks_to_generate, state)?;
        state.todays_tasks_mut().extend(new_tasks);
    }
    Ok(())
}
