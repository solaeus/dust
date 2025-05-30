use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

/// A correctly formatted relative or absolute path to a module or value.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Path<'a> {
    inner: &'a str,
}

impl<'a> Path<'a> {
    pub fn new(inner: &'a str) -> Option<Self> {
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

    pub fn inner(&self) -> &str {
        self.inner
    }

    pub fn module_names(&self) -> Vec<Path<'a>> {
        let mut module_names = self
            .inner
            .rsplit("::")
            .skip(1)
            .map(|inner| Path { inner })
            .collect::<Vec<_>>();

        module_names.reverse();

        module_names
    }

    pub fn item_name(&self) -> Path<'a> {
        self.inner
            .rsplit("::")
            .next()
            .map(|inner| Path { inner })
            .unwrap()
    }

    pub fn contains_scope(&self, other: &str) -> bool {
        let other_root = other.split("::").next().unwrap_or("");

        self.module_names()
            .iter()
            .any(|module_name| module_name.inner == other_root)
            || self.item_name().inner == other_root
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
        let path = Path::new("foo").unwrap();

        assert_eq!(path.module_names().len(), 0);
        assert_eq!(path.item_name(), Path::new("foo").unwrap());
    }

    #[test]
    fn no_module_and_two_values() {
        let path = Path::new("bar.baz").unwrap();

        assert_eq!(path.module_names().len(), 0);
        assert_eq!(path.item_name(), Path::new("bar.baz").unwrap());
    }

    #[test]
    fn one_module_and_one_value() {
        let path = Path::new("foo::bar").unwrap();

        assert_eq!(path.module_names(), vec![Path::new("foo").unwrap()]);
        assert_eq!(path.item_name(), Path::new("bar").unwrap());
    }

    #[test]
    fn one_module_and_two_values() {
        let path = Path::new("foo::bar.baz").unwrap();

        assert_eq!(path.module_names(), vec![Path::new("foo").unwrap()]);
        assert_eq!(path.item_name(), Path::new("bar.baz").unwrap());
    }

    #[test]
    fn two_modules_and_one_value() {
        let path = Path::new("foo::bar::baz").unwrap();

        assert_eq!(
            path.module_names(),
            vec![Path::new("foo").unwrap(), Path::new("bar").unwrap()]
        );
        assert_eq!(path.item_name(), Path::new("baz").unwrap());
    }

    #[test]
    fn two_modules_and_two_values() {
        let path = Path::new("foo::bar::baz.qux").unwrap();

        assert_eq!(
            path.module_names(),
            vec![Path::new("foo").unwrap(), Path::new("bar").unwrap()]
        );
        assert_eq!(path.item_name(), Path::new("baz.qux").unwrap());
    }
}
