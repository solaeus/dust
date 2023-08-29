//! Representation of a moment in time.
//!
//! Dust tries to represent time values correctly. To do this, there must be a clear separation
//! between monotonic timestamps, naive times that do not know their locale and those that have a ..
//! timezone.
//!
//! Only monotonic time instances are guaranteed not to repeat, although an Instant can be used to
//! create and of these variants. Users generally want the timezone included, so the `as_local` is
//! included, which will use no timezone offset if one is not available.

use std::{
    fmt::{self, Display, Formatter},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, FixedOffset, Local as LocalTime, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Time {
    Utc(NaiveDateTime),
    Local(DateTime<LocalTime>),
    Monotonic(Instant),
}

impl Time {
    pub fn utc(instant: Instant) -> Self {
        let utc =
            NaiveDateTime::from_timestamp_micros(instant.elapsed().as_micros() as i64).unwrap();

        Time::Utc(utc)
    }

    pub fn from_timestamp(microseconds: i64) -> Self {
        let utc = NaiveDateTime::from_timestamp_micros(microseconds).unwrap();

        Time::Utc(utc)
    }

    pub fn local(instant: Instant) -> Self {
        let local = DateTime::from_local(
            NaiveDateTime::from_timestamp_micros(instant.elapsed().as_micros() as i64).unwrap(),
            FixedOffset::west_opt(0).unwrap(),
        );

        Time::Local(local)
    }

    pub fn monotonic(instant: Instant) -> Self {
        Time::Monotonic(instant)
    }

    pub fn as_local(&self) -> String {
        let date_time = match *self {
            Time::Utc(utc) => DateTime::from_utc(utc, FixedOffset::west_opt(0).unwrap()),
            Time::Local(local) => local,
            Time::Monotonic(instant) => DateTime::from_utc(
                NaiveDateTime::from_timestamp_millis(instant.elapsed().as_millis() as i64).unwrap(),
                FixedOffset::west_opt(0).unwrap(),
            ),
        };

        date_time.to_rfc2822()
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Time::Utc(inner) => write!(f, "{}", inner),
            Time::Local(inner) => write!(f, "{}", inner),
            Time::Monotonic(inner) => write!(f, "{:?}", inner),
        }
    }
}

impl Serialize for Time {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl From<SystemTime> for Time {
    fn from(value: SystemTime) -> Self {
        Time::Local(value.into())
    }
}
