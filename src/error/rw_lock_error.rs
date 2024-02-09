use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::PoisonError,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RwLockError;

impl Display for RwLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Map error: failed to acquire a read/write lock because another thread has panicked."
        )
    }
}

impl Debug for RwLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl<T> From<PoisonError<T>> for RwLockError {
    fn from(_: PoisonError<T>) -> Self {
        RwLockError
    }
}
