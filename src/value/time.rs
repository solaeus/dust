use std::{
    fmt::{self, Display, Formatter},
    time::{Instant, SystemTime, UNIX_EPOCH},
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
                NaiveDateTime::from_timestamp_micros(instant.elapsed().as_micros() as i64).unwrap(),
                FixedOffset::west_opt(0).unwrap(),
            ),
        };

        date_time.to_string()
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_local())
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
        let timestamp = value.duration_since(UNIX_EPOCH).unwrap().as_micros();
        let naive = NaiveDateTime::from_timestamp_micros(timestamp as i64).unwrap();

        Time::Utc(naive)
    }
}
