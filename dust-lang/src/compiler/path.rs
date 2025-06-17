use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    iter::repeat,
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

    pub fn new_borrowed_item_name(text: &'a str) -> Option<Self> {
        if text.is_empty() {
            None
        } else {
            text.rsplit("::")
                .next()
                .map(|item_name| Path::new_borrowed(item_name).unwrap())
        }
    }

    pub fn inner(&self) -> Cow<'a, str> {
        self.inner.clone()
    }

    pub fn module_names(&'a self) -> impl Iterator<Item = Path<'a>> {
        let item_name = self.item_name();

        self.inner
            .split("::")
            .zip(repeat(item_name))
            .map_while(|(next, item_name)| {
                if next == item_name.inner {
                    None
                } else {
                    let module_path = Path::new_borrowed(next).unwrap();

                    Some(module_path)
                }
            })
    }

    pub fn item_name(&'a self) -> Path<'a> {
        self.inner
            .rsplit("::")
            .next()
            .map(|item_name| Path::new_borrowed(item_name).unwrap())
            .unwrap()
    }

    pub fn contains_scope(&self, other: &Self) -> bool {
        let mut found_module = false;

        for (self_module_name, other_module_name) in self.module_names().zip(other.module_names()) {
            if self_module_name != other_module_name {
                found_module = true;
            }

            if found_module && self_module_name != other_module_name {
                return false;
            }
        }

        if !found_module {
            return false;
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

        assert_eq!(path.module_names().count(), 0);
        assert_eq!(path.item_name().inner(), "foo");
    }

    #[test]
    fn no_module_and_two_values() {
        let path = Path::new_borrowed("bar.baz").unwrap();

        assert_eq!(path.module_names().count(), 0);
        assert_eq!(path.item_name().inner(), "bar.baz");
    }

    #[test]
    fn one_module_and_one_value() {
        let path = Path::new_borrowed("foo::bar").unwrap();

        assert_eq!(
            path.module_names()
                .map(|path| path.inner())
                .collect::<Vec<_>>(),
            vec!["foo"]
        );
        assert_eq!(path.item_name().inner(), "bar");
    }

    #[test]
    fn one_module_and_two_values() {
        let path = Path::new_borrowed("foo::bar.baz").unwrap();

        assert_eq!(
            path.module_names()
                .map(|path| path.inner())
                .collect::<Vec<_>>(),
            vec!["foo"]
        );
        assert_eq!(path.item_name().inner(), "bar.baz");
    }

    #[test]
    fn two_modules_and_one_value() {
        let path = Path::new_borrowed("foo::bar::baz").unwrap();

        assert_eq!(
            path.module_names()
                .map(|path| path.inner())
                .collect::<Vec<_>>(),
            vec!["foo", "bar"]
        );
        assert_eq!(path.item_name().inner(), "baz");
    }

    #[test]
    fn two_modules_and_two_values() {
        let path = Path::new_borrowed("foo::bar::baz.qux").unwrap();

        assert_eq!(
            path.module_names()
                .map(|path| path.inner())
                .collect::<Vec<_>>(),
            vec!["foo", "bar"]
        );
        assert_eq!(path.item_name().inner(), "baz.qux");
    }
}
