use super::Task;
use crate::{Error, Result, State};
use serde::{Deserialize, Serialize};
use std::{
    borrow::{Borrow, BorrowMut},
    collections::{
        BTreeSet,
        btree_set::{IntoIter as BTreeSetIntoIter, Iter as BTreeSetIter},
    },
    iter::{Extend, FromIterator},
    ops::{BitOr, Deref, DerefMut, Sub},
};

/// A new type wrapper around a BTreeSet to allow extra methods.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(transparent)]
pub struct TaskSet(BTreeSet<String>);

impl TaskSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve<'a>(&self, state: &'a State) -> Result<Vec<&'a Task>> {
        self.0
            .iter()
            .map(|s| state.get_task(s).ok_or_else(|| Error::task_not_found(s)))
            .collect()
    }
}

impl From<&TaskSet> for Vec<String> {
    #[inline]
    fn from(value: &TaskSet) -> Self {
        value.0.iter().cloned().collect()
    }
}

impl From<TaskSet> for Vec<String> {
    #[inline]
    fn from(value: TaskSet) -> Self {
        value.0.into_iter().collect()
    }
}

impl Sub<&TaskSet> for &TaskSet {
    type Output = TaskSet;

    #[inline]
    fn sub(self, rhs: &TaskSet) -> TaskSet {
        TaskSet(self.0.sub(&rhs.0))
    }
}

impl BitOr<&BTreeSet<String>> for &TaskSet {
    type Output = TaskSet;

    #[inline]
    fn bitor(self, rhs: &BTreeSet<String>) -> TaskSet {
        TaskSet(self.0.bitor(rhs))
    }
}

impl BitOr<&TaskSet> for &BTreeSet<String> {
    type Output = BTreeSet<String>;

    #[inline]
    fn bitor(self, rhs: &TaskSet) -> BTreeSet<String> {
        self.bitor(&rhs.0)
    }
}

impl BitOr<&TaskSet> for &TaskSet {
    type Output = TaskSet;

    #[inline]
    fn bitor(self, rhs: &TaskSet) -> TaskSet {
        TaskSet(self.0.bitor(&rhs.0))
    }
}

impl AsRef<TaskSet> for &TaskSet {
    #[inline]
    fn as_ref(&self) -> &TaskSet {
        self
    }
}

impl<S: AsRef<str>> FromIterator<S> for TaskSet {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = S>,
    {
        Self(iter.into_iter().map(|v| String::from(v.as_ref())).collect())
    }
}

impl<S: AsRef<str>> Extend<S> for TaskSet {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = S>,
    {
        self.0
            .extend(iter.into_iter().map(|s| String::from(s.as_ref())))
    }
}

impl<'a> IntoIterator for &'a TaskSet {
    type Item = &'a String;
    type IntoIter = BTreeSetIter<'a, String>;

    fn into_iter(self) -> BTreeSetIter<'a, String> {
        self.0.iter()
    }
}

impl IntoIterator for TaskSet {
    type Item = String;
    type IntoIter = BTreeSetIntoIter<String>;

    fn into_iter(self) -> BTreeSetIntoIter<String> {
        self.0.into_iter()
    }
}

impl From<TaskSet> for BTreeSet<String> {
    fn from(value: TaskSet) -> Self {
        value.0
    }
}

impl From<BTreeSet<String>> for TaskSet {
    fn from(value: BTreeSet<String>) -> Self {
        Self(value)
    }
}

impl AsRef<BTreeSet<String>> for TaskSet {
    fn as_ref(&self) -> &BTreeSet<String> {
        self.deref()
    }
}

impl AsMut<BTreeSet<String>> for TaskSet {
    fn as_mut(&mut self) -> &mut BTreeSet<String> {
        self.deref_mut()
    }
}

impl Borrow<BTreeSet<String>> for TaskSet {
    fn borrow(&self) -> &BTreeSet<String> {
        self.deref()
    }
}

impl BorrowMut<BTreeSet<String>> for TaskSet {
    fn borrow_mut(&mut self) -> &mut BTreeSet<String> {
        self.deref_mut()
    }
}

impl Deref for TaskSet {
    type Target = BTreeSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TaskSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
