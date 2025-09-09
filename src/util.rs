use crate::{Error, Result, Task, state::State};
use serde::{Deserialize, Serialize};
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    collections::{
        BTreeSet,
        btree_set::{IntoIter as BTreeSetIntoIter, Iter as BTreeSetIter},
    },
    iter::{Extend, FromIterator},
    ops::{Deref, DerefMut},
    rc::Rc,
};
use time::{Date, Duration, OffsetDateTime, Time, UtcOffset};

lazy_static! {
    pub(crate) static ref LOCAL_OFFSET: UtcOffset = UtcOffset::current_local_offset().unwrap();
}

#[inline]
pub(crate) fn now() -> OffsetDateTime {
    OffsetDateTime::now_local().unwrap()
}

#[inline]
pub(crate) fn today() -> Date {
    now().date()
}

/// If a provided date-time occurs before a provided cut-off, then this will act like a date on the
/// previous date. Otherwise, it will act like the provided date-time's date.
pub fn dt_with_cutoff(dt: &OffsetDateTime, cut_off: Time) -> Date {
    if dt.time() < cut_off {
        dt.date() - Duration::DAY
    } else {
        dt.date()
    }
}

#[inline]
pub fn now_with_cutoff(cut_off: Time) -> Date {
    dt_with_cutoff(&now(), cut_off)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Default)]
/// A new type that wraps a `Rc<RefCell<V>>`. This is _super_ useful in our case because we're
/// often referring to the same object across different objects (mostly because our state has a
/// state model, a config, and a list of tasks, each of which have a reference to the _task's_
/// state and config). We use this so that when we alter the desire task or state in one place, it
/// alters it in all places so that we don't have to keep track of every place we need to update
/// it.
pub struct RcCell<V>(Rc<RefCell<V>>);

impl<V> RcCell<V> {
    pub fn new(value: V) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn into_inner(self) -> V {
        Rc::into_inner(self.0).unwrap().into_inner()
    }
}

impl<V> Deref for RcCell<V> {
    type Target = RefCell<V>;

    fn deref(&self) -> &RefCell<V> {
        self.0.deref()
    }
}

impl<V> AsRef<RefCell<V>> for RcCell<V> {
    fn as_ref(&self) -> &RefCell<V> {
        self.0.deref()
    }
}

impl<V> Borrow<RefCell<V>> for RcCell<V> {
    fn borrow(&self) -> &RefCell<V> {
        self.0.deref()
    }
}

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
