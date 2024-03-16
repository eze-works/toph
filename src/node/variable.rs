use crate::encode;
use fastrand;
use std::collections::BTreeMap;
use std::fmt::{Display, Write};

/// The value associated with a variable + a random suffix that will be appended
/// to the variable name to make it unique to Node instance
///
/// This addition is necessary because in the case of a nested component, the
/// ancestor variable value gets  overridden in descendants.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Entry {
    pub value: String,
    pub suffix: u32,
}

/// Map of custom CSS variable name to value
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CSSVariableMap(BTreeMap<&'static str, Entry>);

impl CSSVariableMap {
    /// Add a new custom CSS variable to the map
    ///
    /// The value will be attribute encoded  
    pub fn insert(&mut self, key: &'static str, value: &str) {
        let encoded = encode::attr(value);
        let suffix = fastrand::u32(0..u32::MAX);
        self.0.insert(
            key,
            Entry {
                value: encoded,
                suffix,
            },
        );
    }

    /// Create a new variable map
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Returns `true` if the variable map is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> IntoIterator for &'a CSSVariableMap {
    type Item = (&'a &'static str, &'a Entry);
    type IntoIter = std::collections::btree_map::Iter<'a, &'static str, Entry>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Display for CSSVariableMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }
        let mut result = String::new();
        for (k, v) in self.0.iter() {
            write!(result, "--{}-{}: {};", k, v.suffix, v.value)?;
        }
        write!(f, "{}", result)
    }
}
