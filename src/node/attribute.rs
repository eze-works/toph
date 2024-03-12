use crate::{allowlist, encode};
use std::borrow::Cow;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;

/// HTML Attribute map
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttributeMap {
    boolean: BTreeSet<&'static str>,
    regular: BTreeMap<&'static str, Cow<'static, str>>,
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
    pub fn entry(&mut self, key: &'static str) -> Entry<'_, &'static str, Cow<'static, str>> {
        self.regular.entry(key)
    }

    /// Add a new HTML attribute.
    ///
    /// Boolean attributes can be inserted by setting the value to `None`
    ///
    /// For regular attributes, if the value is not a `'static` string slice, it will be attribute
    /// encoded
    ///
    /// Attribute values that are comma or space-separated according to the WHATWG spec are
    /// appended if they are inserted more than once
    ///
    /// For all other attributes, existing values will be overwritten if the attribute appears more
    /// than once
    pub fn insert(&mut self, key: &'static str, value: Option<Cow<'static, str>>) {
        // Don't care to store empty keys
        if key.is_empty() || key.chars().all(|c| c.is_ascii_whitespace()) {
            return;
        }

        match value {
            // Boolean attributes are stored verbatim
            None => {
                self.boolean.insert(key);
            }
            // Borrowed values are url encoded if applicable
            Some(value @ Cow::Borrowed(_)) => {
                let value = if URL_ATTRIBUTES.contains(&key) {
                    encode::url(&value, false).map(|v| Cow::Owned(v))
                } else {
                    Some(value)
                };

                let Some(value) = value else { return };

                self.insert_or_modify(key, value);
            }
            // Owned values are url encoded (if applicable) and attribute encoded
            Some(v @ Cow::Owned(_)) => {
                let value = v.into_owned();

                let value = if URL_ATTRIBUTES.contains(&key) {
                    encode::url(&value, true)
                } else {
                    Some(value)
                };

                let Some(value) = value else { return };

                if allowlist::ALLOWED_ATTR_NAMES.contains(&key) || key.starts_with("data_") {
                    let encoded_value = encode::attr(&value);
                    self.insert_or_modify(key, encoded_value.into());
                }
            }
        }
    }

    fn insert_or_modify(&mut self, key: &'static str, value: Cow<'static, str>) {
        if SPACE_SEPARATED.contains(&key) {
            if let Some(existing) = self.regular.get_mut(key) {
                *existing += " ";
                *existing += value;
            } else {
                self.regular.insert(key, value);
            }
        } else if COMMA_SEPARATED.contains(&key) {
            if let Some(existing) = self.regular.get_mut(key) {
                *existing += ",";
                *existing += value;
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
/// The attribute key must be a valid rust identifier. This means that `data-*` attributes must be
/// written in Rust code as `data_*`. When printed to a string `data_*` attributes are convereted
/// to `data-` as you would expect.
///
/// ```
/// use toph::attr;
/// // let list = attr![data-custom="true"]; // Syntax error
/// let list = attr![data_custom="true"];
/// ```
///
/// The attribute value can be either:
/// - A string slice with `'static` lifetime
/// - An owned `String`
///
/// Using a non  non `'static` string slice will cause a borrow-checker error:
///
/// ```
/// use toph::attr;
///
/// let string_slice = "hello";
/// let heap_allocated_string = "world".to_string();
/// let reference_to_heap_allocated_string = &heap_allocated_string;
///
/// let list = attr![class=string_slice]; // OK
/// let list = attr![class=heap_allocated_string]; // OK
/// // let list = attr![class=reference_to_heap_allocated_string];
/// // Error: borrowed value does not live long enough
/// ```
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
        $crate::attr_impl!([$($attr,)* (stringify!($name), Some(<std::borrow::Cow<'static, str>>::from($value)))] -> $($rest)*)
    };
    ([$($attr:expr),*] -> $name:ident = $value:expr) => {
        $crate::attr_impl!([$($attr,)* (stringify!($name), Some(<std::borrow::Cow<'static, str>>::from($value)))] ->)
    };

    // Match boolean attributes
    ([$($attr:expr),*] -> $name:ident , $($rest:tt)*) => {
        $crate::attr_impl!([$($attr,)* (stringify!($name), <Option<std::borrow::Cow<'static, str>>>::None)] -> $($rest)*)
    };
    ([$($attr:expr),*] -> $name:ident) => {
        $crate::attr_impl!([$($attr,)* (stringify!($name), <Option<std::borrow::Cow<'static, str>>>::None)] ->)
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
        map.insert("accept", Some("video/*".into()));
        map.insert("accept", Some("audio/*".into()));

        assert_eq!(
            map.regular.get("accept").expect("key should be set"),
            "video/*,audio/*",
        );
    }

    #[test]
    fn inserting_space_separated_attributes() {
        let mut map = AttributeMap::new();
        map.insert("for", Some("form1".into()));
        map.insert("for", Some("form2".into()));
        map.insert("for", Some("form3".into()));

        assert_eq!(
            map.regular.get("for").expect("key should be set"),
            "form1 form2 form3"
        );
    }

    #[test]
    fn inserting_regular_attributes() {
        let mut map = AttributeMap::new();
        map.insert("id", Some("id1".into()));
        map.insert("id", Some("id2".into()));
        map.insert("id", Some("id3".into()));

        assert_eq!(map.regular.get("id").expect("key should be set"), "id3");
    }

    #[test]
    fn inserting_boolean_attributes() {
        let mut map = AttributeMap::new();
        map.insert("garbage", None);
        map.insert("something", None);
        map.insert("lol\"wut", None);

        assert_eq!(
            map.boolean,
            BTreeSet::from(["garbage", "something", "lol\"wut"])
        );
    }

    #[test]
    fn borrowed_attributes_are_stored_verbatim() {
        let mut map = AttributeMap::new();

        // literal values do not get blacklisted or html attribute encoded
        map.insert("onclick", Some("look at this \" mess".into()));
        assert_eq!(
            map.regular.get("onclick").expect("key should be set"),
            "look at this \" mess"
        );
    }

    #[test]
    fn borrwed_url_attributes_are_percent_encoded() {
        let mut map = AttributeMap::new();
        map.insert("src", Some("/about me".into()));

        assert_eq!(
            map.regular.get("src").expect("key should be set"),
            "/about%20me"
        );
    }

    #[test]
    fn borrowed_url_attributes_do_not_get_filtered_using_scheme() {
        let mut map = AttributeMap::new();
        map.insert("src", Some("javascript:alert()".into()));

        assert_eq!(
            map.regular.get("src").expect("key should be set"),
            "javascript:alert()"
        );
    }

    #[test]
    fn owned_attributes_are_encoded() {
        let mut map = AttributeMap::new();
        let owned = String::from("no mess\" here");
        map.insert("class", Some(owned.into()));

        assert_eq!(
            map.regular.get("class").expect("key should be set"),
            "no mess&quot; here"
        );
    }

    #[test]
    fn owned_attributes_get_filtered_out_using_allowlist() {
        let mut map = AttributeMap::new();
        let owned = String::from("boom");
        map.insert("onclick", Some(owned.into()));

        assert!(map.is_empty());
    }

    #[test]
    fn owned_url_attributes_get_filtered_out_using_scheme() {
        let mut map = AttributeMap::new();
        let owned = String::from("javascript:alert(1)");
        map.insert("src", Some(owned.into()));
        assert!(map.is_empty());
    }

    #[test]
    fn empty_key() {
        let mut map = AttributeMap::new();
        map.insert("", None);
        map.insert("  ", None);
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
