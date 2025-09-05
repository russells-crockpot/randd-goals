use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, cell::RefCell, ops::Deref, rc::Rc};
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
/// state model, a config, and a list of goals, each of which have a reference to the _goal's_
/// state and config). We use this so that when we alter the desire goal or state in one place, it
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
