use crate::encode;
use std::collections::BTreeMap;
use std::fmt::{Display, Write};

/// Map of custom CSS variable name to value
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CSSVariableMap(BTreeMap<&'static str, String>);

impl CSSVariableMap {
    /// Add a new custom CSS variable to the map
    ///
    /// The value will be attribute encoded  
    pub fn insert(&mut self, key: &'static str, value: &str) {
        let encoded = encode::attr(value);
        self.0.insert(key, encoded);
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

impl Display for CSSVariableMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }
        let mut result = String::new();
        for (k, v) in self.0.iter() {
            write!(result, "--{}: {};", k, v)?;
        }
        write!(f, "{}", result)
    }
}
