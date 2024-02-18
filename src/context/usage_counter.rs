use std::{
    cmp::Ordering,
    sync::{Arc, RwLock},
};

use crate::error::rw_lock_error::RwLockError;

#[derive(Clone, Debug)]
pub struct UsageCounter(Arc<RwLock<UsageCounterInner>>);

impl UsageCounter {
    pub fn new() -> UsageCounter {
        UsageCounter(Arc::new(RwLock::new(UsageCounterInner {
            allowances: 0,
            runtime_uses: 0,
        })))
    }

    pub fn get_counts(&self) -> Result<(usize, usize), RwLockError> {
        let inner = self.0.read()?;
        Ok((inner.allowances, inner.runtime_uses))
    }

    pub fn add_allowance(&self) -> Result<(), RwLockError> {
        self.0.write()?.allowances += 1;

        Ok(())
    }

    pub fn add_runtime_use(&self) -> Result<(), RwLockError> {
        self.0.write()?.runtime_uses += 1;

        Ok(())
    }
}

impl Eq for UsageCounter {}

impl PartialEq for UsageCounter {
    fn eq(&self, other: &Self) -> bool {
        let left = self.0.read().unwrap();
        let right = other.0.read().unwrap();

        *left == *right
    }
}

impl PartialOrd for UsageCounter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UsageCounter {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self.0.read().unwrap();
        let right = other.0.read().unwrap();

        left.cmp(&right)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct UsageCounterInner {
    pub allowances: usize,
    pub runtime_uses: usize,
}
