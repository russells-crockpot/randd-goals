use crate::{Config, Error, Result, State, Task, state::StateModel};
use clap_complete::CompletionCandidate;
use std::{collections::BTreeMap, ffi::OsStr};

//TODO allow ignoring case?
fn filter_candidate_tasks<I>(current: &OsStr, all_candidates: I) -> Vec<CompletionCandidate>
where
    I: IntoIterator<Item = (String, String)>,
{
    let current = current.to_str().unwrap();
    let mut starts_with = BTreeMap::new();
    let mut contains = BTreeMap::new();
    let mut ends_with = BTreeMap::new();
    let mut exact = None;
    for (slug, help) in all_candidates.into_iter() {
        if slug == current {
            exact = Some((slug, help))
        } else if slug.starts_with(current) {
            starts_with.insert(slug, help);
        } else if slug.ends_with(current) {
            ends_with.insert(slug, help);
        } else if slug.contains(current) {
            contains.insert(slug, help);
        }
    }
    let mut results = Vec::new();
    if let Some((slug, help)) = exact {
        results.push(
            CompletionCandidate::new(slug)
                .display_order(Some(results.len()))
                .help(Some(help.into())),
        );
    }
    for (slug, help) in starts_with {
        results.push(
            CompletionCandidate::new(slug)
                .display_order(Some(results.len()))
                .help(Some(help.into())),
        );
    }
    for (slug, help) in contains {
        results.push(
            CompletionCandidate::new(slug)
                .display_order(Some(results.len()))
                .help(Some(help.into())),
        );
    }
    for (slug, help) in ends_with {
        results.push(
            CompletionCandidate::new(slug)
                .display_order(Some(results.len()))
                .help(Some(help.into())),
        );
    }
    results
}

pub(crate) fn all_tasks(current: &OsStr) -> Vec<CompletionCandidate> {
    let config = Config::load().unwrap();
    filter_candidate_tasks(
        current,
        config
            .tasks()
            .iter()
            .map(|t| (t.borrow().slug().into(), t.borrow().task.clone())),
    )
}

pub(crate) fn todays_tasks(current: &OsStr) -> Vec<CompletionCandidate> {
    let state = State::load().unwrap();
    filter_candidate_tasks(
        current,
        state
            .todays_tasks()
            .resolve(&state)
            .unwrap()
            .into_iter()
            .map(|t| (t.slug().into(), t.task())),
    )
}

pub(crate) fn uncompleted_tasks(current: &OsStr) -> Vec<CompletionCandidate> {
    let state = State::load().unwrap();
    filter_candidate_tasks(
        current,
        state
            .uncompleted_tasks()
            .unwrap()
            .into_iter()
            .map(|t| (t.slug().into(), t.task())),
    )
}

pub(crate) fn completed_tasks(current: &OsStr) -> Vec<CompletionCandidate> {
    let state = State::load().unwrap();
    filter_candidate_tasks(
        current,
        state
            .completed_tasks()
            .unwrap()
            .into_iter()
            .map(|t| (t.slug().into(), t.task())),
    )
}

pub(crate) fn disabled_tasks(current: &OsStr) -> Vec<CompletionCandidate> {
    let state = State::load().unwrap();
    filter_candidate_tasks(
        current,
        state
            .disabled_tasks()
            .into_iter()
            .map(|t| (t.slug().into(), t.task())),
    )
}

pub(crate) fn enabled_tasks(current: &OsStr) -> Vec<CompletionCandidate> {
    let state = State::load().unwrap();
    filter_candidate_tasks(
        current,
        state
            .enabled_tasks()
            .into_iter()
            .map(|t| (t.slug().into(), t.task())),
    )
}
