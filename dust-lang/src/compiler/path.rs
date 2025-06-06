use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

/// A correctly formatted relative or absolute path to a module or value.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Path<'a> {
    inner: Cow<'a, str>,
}

impl<'a> Path<'a> {
    pub fn new_owned(inner: String) -> Option<Self> {
        if inner.split("::").any(|module_name| {
            module_name.is_empty()
                || module_name
                    .split('.')
                    .any(|value_name| value_name.is_empty())
        }) {
            None
        } else {
            Some(Self {
                inner: Cow::Owned(inner),
            })
        }
    }

    pub fn new_borrowed(inner: &'a str) -> Option<Self> {
        if inner.split("::").any(|module_name| {
            module_name.is_empty()
                || module_name
                    .split('.')
                    .any(|value_name| value_name.is_empty())
        }) {
            None
        } else {
            Some(Self {
                inner: Cow::Borrowed(inner),
            })
        }
    }

    pub fn inner(&self) -> Cow<'a, str> {
        self.inner.clone()
    }

    pub fn module_names(&self) -> Vec<&str> {
        let mut module_names = self.inner.rsplit("::").skip(1).collect::<Vec<_>>();

        module_names.reverse();

        module_names
    }

    pub fn item_name(&self) -> &str {
        self.inner.rsplit("::").next().unwrap()
    }

    pub fn contains_scope(&self, other: &Self) -> bool {
        for (self_module_name, other_module_name) in
            self.module_names().iter().zip(other.module_names().iter())
        {
            if self_module_name != other_module_name {
                return false;
            }
        }

        self.item_name() == other.item_name()
    }
}

impl<'a> Display for Path<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_module_and_one_value() {
        let path = Path::new_borrowed("foo").unwrap();

        assert_eq!(path.module_names().len(), 0);
        assert_eq!(path.item_name(), "foo");
    }

    #[test]
    fn no_module_and_two_values() {
        let path = Path::new_borrowed("bar.baz").unwrap();

        assert_eq!(path.module_names().len(), 0);
        assert_eq!(path.item_name(), "bar.baz");
    }

    #[test]
    fn one_module_and_one_value() {
        let path = Path::new_borrowed("foo::bar").unwrap();

        assert_eq!(path.module_names(), vec!["foo"]);
        assert_eq!(path.item_name(), "bar");
    }

    #[test]
    fn one_module_and_two_values() {
        let path = Path::new_borrowed("foo::bar.baz").unwrap();

        assert_eq!(path.module_names(), vec!["foo"]);
        assert_eq!(path.item_name(), "bar.baz");
    }

    #[test]
    fn two_modules_and_one_value() {
        let path = Path::new_borrowed("foo::bar::baz").unwrap();

        assert_eq!(path.module_names(), vec!["foo", "bar"]);
        assert_eq!(path.item_name(), "baz");
    }

    #[test]
    fn two_modules_and_two_values() {
        let path = Path::new_borrowed("foo::bar::baz.qux").unwrap();

        assert_eq!(path.module_names(), vec!["foo", "bar"]);
        assert_eq!(path.item_name(), "baz.qux");
    }
}
