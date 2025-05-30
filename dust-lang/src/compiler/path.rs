use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::DustString;

/// A correctly formatted relative or absolute path to a module or value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Path {
    inner: DustString,
}

impl Path {
    pub fn new(inner: impl Into<DustString>) -> Option<Self> {
        let inner = inner.into();

        if inner.split("::").any(|module_name| {
            module_name.is_empty()
                || module_name
                    .split('.')
                    .any(|value_name| value_name.is_empty())
        }) {
            None
        } else {
            Some(Self { inner })
        }
    }

    pub fn inner(&self) -> &DustString {
        &self.inner
    }

    pub fn into_inner(self) -> DustString {
        self.inner
    }

    pub fn module_names(&self) -> Vec<&str> {
        let mut module_names = self.inner.rsplit("::").skip(1).collect::<Vec<_>>();

        module_names.reverse();

        module_names
    }

    pub fn value_names(&self) -> impl Iterator<Item = &str> {
        self.inner.rsplit("::").take(1).next().unwrap().split('.')
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_module_and_one_value() {
        let path = Path::new(DustString::from("foo")).unwrap();

        assert_eq!(path.module_names().len(), 0);
        assert_eq!(path.value_names().collect::<Vec<_>>(), vec!["foo"]);
    }

    #[test]
    fn no_module_and_two_values() {
        let path = Path::new(DustString::from("bar.baz")).unwrap();

        assert_eq!(path.module_names().len(), 0);
        assert_eq!(path.value_names().collect::<Vec<_>>(), vec!["bar", "baz"]);
    }

    #[test]
    fn one_module_and_one_value() {
        let path = Path::new(DustString::from("foo::bar")).unwrap();

        assert_eq!(path.module_names(), vec!["foo"]);
        assert_eq!(path.value_names().collect::<Vec<_>>(), vec!["bar"]);
    }

    #[test]
    fn one_module_and_two_values() {
        let path = Path::new(DustString::from("foo::bar.baz")).unwrap();

        assert_eq!(path.module_names(), vec!["foo"]);
        assert_eq!(path.value_names().collect::<Vec<_>>(), vec!["bar", "baz"]);
    }

    #[test]
    fn two_modules_and_one_value() {
        let path = Path::new(DustString::from("foo::bar::baz")).unwrap();

        assert_eq!(path.module_names(), vec!["foo", "bar"]);
        assert_eq!(path.value_names().collect::<Vec<_>>(), vec!["baz"]);
    }

    #[test]
    fn two_modules_and_two_values() {
        let path = Path::new(DustString::from("foo::bar::baz.qux")).unwrap();

        assert_eq!(path.module_names(), vec!["foo", "bar"]);
        assert_eq!(path.value_names().collect::<Vec<_>>(), vec!["baz", "qux"]);
    }
}
