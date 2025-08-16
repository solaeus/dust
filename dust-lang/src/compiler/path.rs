use std::{
    borrow::Borrow,
    collections::HashSet,
    fmt::{self, Display, Formatter},
    iter::repeat,
    sync::{Arc, LazyLock, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::{CompileError, Span};

static PATH_CACHE: LazyLock<Mutex<HashSet<Path>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

fn cache_path(path: Path) -> Path {
    PATH_CACHE.lock().unwrap().get_or_insert(path).clone()
}

/// A correctly formatted relative or absolute path to a module or value.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Path {
    inner: Arc<String>,
}

impl Path {
    pub fn new<T: Into<String>>(inner: T) -> Option<Self> {
        let path = Path {
            inner: Arc::new(inner.into()),
        };

        if path.verify() {
            Some(cache_path(path))
        } else {
            None
        }
    }

    pub fn new_at_position<T: Into<String> + ToString>(
        inner: T,
        position: Span,
    ) -> Result<Self, CompileError> {
        let path = Path {
            inner: Arc::new(inner.into()),
        };

        if path.verify() {
            Ok(cache_path(path))
        } else {
            Err(CompileError::InvalidPath {
                found: path.inner().to_string(),
                position,
            })
        }
    }

    pub fn inner(&self) -> &Arc<String> {
        &self.inner
    }

    pub fn modules(&self) -> impl Iterator<Item = Path> {
        let item_path = self.item();

        self.inner()
            .split("::")
            .zip(repeat(item_path))
            .map_while(|(next, item_path)| {
                if next == item_path.as_ref() {
                    None
                } else {
                    Path::new(next)
                }
            })
    }

    pub fn item(&self) -> Path {
        self.inner()
            .rsplit("::")
            .next()
            .map(|item_name| Path::new(item_name).unwrap())
            .unwrap()
    }

    pub fn contains_scope(&self, other: &Self) -> bool {
        let mut found_module = false;

        for (self_module_name, other_module_name) in self.modules().zip(other.modules()) {
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

        self.item() == other.item()
    }

    fn verify(&self) -> bool {
        !self.inner().split("::").any(|module_name| {
            module_name.is_empty()
                || module_name
                    .split('.')
                    .any(|value_name| value_name.is_empty())
        })
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inner())
    }
}

impl AsRef<str> for Path {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl Borrow<str> for Path {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use std::iter::once;

    use super::*;

    #[test]
    fn no_module_and_one_value() {
        let path = Path::new("foo").unwrap();

        assert_eq!(path.modules().count(), 0);
        assert_eq!(path.item().inner().as_str(), "foo");
    }

    #[test]
    fn no_module_and_two_values() {
        let path = Path::new("bar.baz").unwrap();

        assert_eq!(path.modules().count(), 0);
        assert_eq!(path.item().inner().as_str(), "bar.baz");
    }

    #[test]
    fn one_module_and_one_value() {
        let path = Path::new("foo::bar").unwrap();

        assert!(path.modules().eq(once(Path::new("foo").unwrap())));
        assert_eq!(path.item().inner().as_str(), "bar");
    }

    #[test]
    fn one_module_and_two_values() {
        let path = Path::new("foo::bar.baz").unwrap();

        assert!(path.modules().eq(once(Path::new("foo").unwrap())));
        assert_eq!(path.item().inner().as_str(), "bar.baz");
    }

    #[test]
    fn two_modules_and_one_value() {
        let path = Path::new("foo::bar::baz").unwrap();

        assert!(
            path.modules()
                .eq([Path::new("foo").unwrap(), Path::new("bar").unwrap()].into_iter())
        );
        assert_eq!(path.item().inner().as_str(), "baz");
    }

    #[test]
    fn two_modules_and_two_values() {
        let path = Path::new("foo::bar::baz.qux").unwrap();

        assert!(
            path.modules()
                .eq([Path::new("foo").unwrap(), Path::new("bar").unwrap()].into_iter())
        );
        assert_eq!(path.item().inner().as_str(), "baz.qux");
    }
}
