use time::{Date, OffsetDateTime};

#[inline]
pub(crate) fn now() -> OffsetDateTime {
    OffsetDateTime::now_local().unwrap()
}

#[inline]
pub(crate) fn today() -> Date {
    now().date()
}
