use crate::encode;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;

/// HTML Attribute map
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttributeMap {
    boolean: BTreeSet<&'static str>,
    regular: BTreeMap<&'static str, String>,
}

// List of attributes that are space separated
const SPACE_SEPARATED: [&str; 12] = [
    "accesskey",
    "blocking",
    "class",
    "for",
    "headers",
    "itemprop",
    "itemref",
    "itemtype",
    "ping",
    "rel",
    "sandbox",
    "sizes",
];

// List of attributes that are comma separated
const COMMA_SEPARATED: [&str; 2] = ["accept", "imagesrcset"];

// List of attributes that require url encoding
const URL_ATTRIBUTES: [&str; 7] = [
    "action",
    "cite",
    "data",
    "formaction",
    "href",
    "poster",
    "src",
];

impl AttributeMap {
    /// Create a new attribute map
    pub const fn new() -> Self {
        Self {
            regular: BTreeMap::new(),
            boolean: BTreeSet::new(),
        }
    }

    /// Returns the key's entry in the regular attribute map
    pub fn entry(&mut self, key: &'static str) -> Entry<'_, &'static str, String> {
        self.regular.entry(key)
    }

    /// Add a new HTML attribute.
    ///
    /// Attributes values are url encoded when necessary. They are alway attribute encoded.
    ///
    /// Attribute values that are comma or space-separated according to the WHATWG spec are
    /// appended if they are inserted more than once For all other attributes, existing values will
    /// be overwritten if the attribute appears more than once
    pub fn insert(&mut self, key: &'static str, value: &str, boolean_attr: bool) {
        // Don't care to store empty keys
        if key.is_empty() || key.chars().all(|c| c.is_ascii_whitespace()) {
            return;
        }

        if boolean_attr {
            // Boolean attributes are stored verbatim
            self.boolean.insert(key);
        } else {
            let value = if URL_ATTRIBUTES.contains(&key) {
                encode::url(&value)
            } else {
                Some(value.into())
            };

            let Some(value) = value else { return };

            let encoded_value = encode::attr(&value);
            self.insert_or_modify(key, encoded_value);
        }
    }

    fn insert_or_modify(&mut self, key: &'static str, value: String) {
        if SPACE_SEPARATED.contains(&key) {
            if let Some(existing) = self.regular.get_mut(key) {
                *existing += " ";
                *existing += &value;
            } else {
                self.regular.insert(key, value);
            }
        } else if COMMA_SEPARATED.contains(&key) {
            if let Some(existing) = self.regular.get_mut(key) {
                *existing += ",";
                *existing += &value;
            } else {
                self.regular.insert(key, value);
            }
        } else {
            self.regular.insert(key, value);
        }
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.regular.is_empty() && self.boolean.is_empty()
    }
}

impl Display for AttributeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.regular.iter() {
            if k.contains('_') {
                write!(f, " {}=\"{}\"", k.replace('_', "-"), v)?;
            } else {
                write!(f, " {}=\"{}\"", k, v)?;
            }
        }

        for k in self.boolean.iter() {
            if k.contains('_') {
                write!(f, " {}", k.replace('_', "-"))?;
            } else {
                write!(f, " {}", k)?;
            }
        }
        Ok(())
    }
}

/// Attribute list builder
///
/// The macro has a single form:
/// ```
/// use toph::attr;
/// let list = attr![class="my-class", async, checked, id="my-id"];
/// ```
///
/// You may have a trailing comma:
///
/// ```
/// use toph::attr;
/// let list = attr![hidden,];
/// ```
///
/// The attribute key must be a valid rust identifier. This means attributes like  `data-*` must be
/// written in Rust code as `data_*`. When printed to a string, the underscore is replaced with a
/// dash as you would expect:
///
/// ```
/// use toph::attr;
/// // let list = attr![data-custom="true"]; // Syntax error
/// let list = attr![data_custom="true"];
/// ```
///
/// Any type that can be converted to a [`String`] can be used as an attribute value
#[macro_export]
macro_rules! attr {
    ($($input:tt)*) => {{
        $crate::attr_impl!([] -> $($input)*)
    }}
}
// # Macro Implementation details:
//
// This uses a technique called pushdown accumulation
// See: https://veykril.github.io/tlborm/decl-macros/patterns/push-down-acc.html
//
// This is an example trace of the macro's expansion usin the `trace-macros` nightly feature
//
// Consider this invocation:
// ```
// attr![async, class = "hidden", checked]
// ```
// This is the trace:
//
// ```
// expanding `attr! { async, class = "hidden", checked }`
// to `{ $crate :: attr_impl! ([] -> async, class = "hidden", checked) }`
//
// expanding `attr_impl! { [] -> async, class = "hidden", checked }`
// to `$crate :: attr_impl! ([$crate :: Attribute :: new_boolean(stringify! (async))] -> class = "hidden", checked)`
//
// expanding `attr_impl! { [$crate :: Attribute :: new_boolean(stringify! (async))] -> class = "hidden", checked }`
// to `$crate :: attr_impl! ([crate::Attribute::new_boolean(stringify!(async)), $crate :: Attribute :: new(stringify! (class), "hidden")] -> checked)`
//
// expanding `attr_impl! { [crate::Attribute::new_boolean(stringify!(async)), $crate :: Attribute :: new(stringify! (class), "hidden")] -> checked }`
// to `[crate::Attribute::new_boolean(stringify!(async)), crate::Attribute::new(stringify!(class), "hidden"), $crate :: Attribute :: new_boolean(stringify! (checked))].to_vec()`
// ```
//
// Given a list like [key = value, key, key = value,  ... ] the macro examines the head of the
// list (i.e. the first `key = value`) and creates a new Attribute value from it.
//
// It then recursively calls it self with the attribute expression it created inside what looks
// like an array (i.e. [Attribute]). No array is actually created because there is another
// rule that matches that array structure using a token tree.
//
// The created attribute "expresssions" are seperated from the unparsed input with a `->`
//
// Jumping through these hoops is necessary because declarative macros need to produce
// valid Rust syntax at each expansion. You cannot at any point output a partial `Vec` of
// arrays. The macro uses recursion to assemble all the tokens necessary to create the full
// expression at the end.
#[doc(hidden)]
#[macro_export]
macro_rules! attr_impl {
    // Match regular key/value attributes
    ([$($attr:expr),*] -> $name:ident = $value:expr , $($rest:tt)*) => {
        $crate::attr_impl!([$($attr,)* (stringify!($name), String::from($value), false)] -> $($rest)*)
    };
    ([$($attr:expr),*] -> $name:ident = $value:expr) => {
        $crate::attr_impl!([$($attr,)* (stringify!($name), String::from($value), false)] ->)
    };

    // Match boolean attributes
    ([$($attr:expr),*] -> $name:ident , $($rest:tt)*) => {
        $crate::attr_impl!([$($attr,)* (stringify!($name), String::new(), true)] -> $($rest)*)
    };
    ([$($attr:expr),*] -> $name:ident) => {
        $crate::attr_impl!([$($attr,)* (stringify!($name), String::new(), true)] ->)
    };

    // Create vec once there is no more input to consume
    ([$($attr:expr),*] ->) => {
        [$($attr,)*]
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::*;

    #[test]
    fn inserting_comma_separated_attributes() {
        let mut map = AttributeMap::new();
        map.insert("accept", "video/*", false);
        map.insert("accept", "audio/*", false);

        assert_eq!(
            map.regular.get("accept").expect("key should be set"),
            "video/*,audio/*",
        );
    }

    #[test]
    fn inserting_space_separated_attributes() {
        let mut map = AttributeMap::new();
        map.insert("for", "form1", false);
        map.insert("for", "form2", false);
        map.insert("for", "form3", false);

        assert_eq!(
            map.regular.get("for").expect("key should be set"),
            "form1 form2 form3"
        );
    }

    #[test]
    fn inserting_regular_attributes() {
        let mut map = AttributeMap::new();
        map.insert("id", "id1", false);
        map.insert("id", "id2", false);
        map.insert("id", "id3", false);

        assert_eq!(map.regular.get("id").expect("key should be set"), "id3");
    }

    #[test]
    fn inserting_boolean_attributes() {
        let mut map = AttributeMap::new();
        map.insert("garbage", "", true);
        map.insert("something", "", true);
        map.insert("lol\"wut", "", true);

        assert_eq!(
            map.boolean,
            BTreeSet::from(["garbage", "something", "lol\"wut"])
        );
    }

    #[test]
    fn url_attributes_are_percent_encoded() {
        let mut map = AttributeMap::new();
        map.insert("src", "/about me", false);

        assert_eq!(
            map.regular.get("src").expect("key should be set"),
            "/about%20me"
        );
    }

    #[test]
    fn attributes_are_html_attribute_encoded() {
        let mut map = AttributeMap::new();
        map.insert("class", "mess\"y", false);

        assert_eq!(
            map.regular.get("class").expect("key should be set"),
            "mess&quot;y"
        );
    }

    #[test]
    fn empty_key() {
        let mut map = AttributeMap::new();
        map.insert("", "", true);
        map.insert("  ", "", true);
        assert!(map.is_empty());
    }

    #[test]
    fn attributes_with_underscores() {
        let mut html = span_.with(attr![data_hello = "hi", something_something]);

        assert_eq!(
            html.write_to_string(false),
            r#"<span data-hello="hi" something-something></span>"#
        );
    }
}
