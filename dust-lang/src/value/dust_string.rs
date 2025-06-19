use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DustString {
    pub inner: SmartString<LazyCompact>,
}

impl DustString {
    pub fn new() -> Self {
        DustString {
            inner: SmartString::new(),
        }
    }
}

impl Default for DustString {
    fn default() -> Self {
        DustString::new()
    }
}

impl<T: Into<SmartString<LazyCompact>>> From<T> for DustString {
    fn from(inner: T) -> Self {
        DustString {
            inner: inner.into(),
        }
    }
}

impl Display for DustString {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl PartialEq<str> for DustString {
    fn eq(&self, other: &str) -> bool {
        **self == *other
    }
}

impl PartialEq<String> for DustString {
    fn eq(&self, other: &String) -> bool {
        **self == **other
    }
}

impl PartialEq<&String> for DustString {
    fn eq(&self, other: &&String) -> bool {
        **self == **other
    }
}

impl AsRef<DustStringRef> for DustString {
    fn as_ref(&self) -> &DustStringRef {
        DustStringRef::new(&self.inner)
    }
}

impl AsRef<str> for DustString {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl Deref for DustString {
    type Target = DustStringRef;

    fn deref(&self) -> &Self::Target {
        DustStringRef::new(&self.inner)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct DustStringRef {
    inner: str,
}

impl DustStringRef {
    pub fn new<S: AsRef<str> + ?Sized>(inner: &S) -> &Self {
        // This code is incredibly cryptic, but it is sound and was copied from the standard
        // library: https://doc.rust-lang.org/std/path/struct.Path.html#method.new
        unsafe { &*(inner.as_ref() as *const str as *const DustStringRef) }
    }
}

impl AsRef<str> for DustStringRef {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl Display for DustStringRef {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

impl PartialEq<str> for DustStringRef {
    fn eq(&self, other: &str) -> bool {
        &self.inner == other
    }
}

impl PartialEq<DustString> for DustStringRef {
    fn eq(&self, other: &DustString) -> bool {
        *self == **other
    }
}

impl PartialEq<String> for DustStringRef {
    fn eq(&self, other: &String) -> bool {
        &self.inner == other.as_str()
    }
}

impl Deref for DustStringRef {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
