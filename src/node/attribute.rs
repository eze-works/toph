use crate::{allowlist, encode};
use std::borrow::Cow;
use std::fmt::Write;

/// An HTML Attribute
///
/// You will generally be using the [attribute list builder](crate::attr) to create HTML attributes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Attribute {
    /// A boolean attribute (e.g. `<input hidden>`
    Bool(Bool),
    /// A regular attribute (e.g. `<span class="something"></span>`)
    Regular(Regular),
    /// An Inline CSS custom variable definition
    Variable(Variable),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Bool(&'static str);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Regular(&'static str, Cow<'static, str>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Variable(&'static str, String);

impl Attribute {
    /// Creates a new regular attribute with a key & value.
    ///
    /// If the value is not a `'static` string slice, it will be attribute encoded
    pub fn new_regular(key: &'static str, value: impl Into<Cow<'static, str>>) -> Self {
        let key = key.trim();
        let value = value.into();
        let attribute = match value {
            v @ Cow::Borrowed(_) => Attribute::Regular(Regular(key, v)),
            v @ Cow::Owned(_) => {
                let value = v.into_owned();
                let value = value.trim();

                if allowlist::URL_ATTRIBUTES.contains(&key) {
                    if let Some(url) = encode::url(value) {
                        Attribute::Regular(Regular(key, url.into()))
                    } else {
                        Attribute::Regular(Regular("", "".into()))
                    }
                } else if allowlist::ALLOWED_ATTR_NAMES.contains(&key) || key.starts_with("data_") {
                    let encoded_value = encode::attr(value);
                    Attribute::Regular(Regular(key, encoded_value.into()))
                } else {
                    Attribute::Regular(Regular("", "".into()))
                }
            }
        };

        attribute
    }

    /// Creates a new boolean attribute
    pub fn new_boolean(key: &'static str) -> Self {
        Attribute::Bool(Bool(key))
    }

    /// Creates a new css variable definitions
    ///
    /// The value is always attribute encoded  
    pub fn new_variable(name: &'static str, value: &str) -> Self {
        Attribute::Variable(Variable(name, encode::attr(value)))
    }

    /// Formats and writes a list of attributes to a `String`
    pub fn write_to_string<'a, I>(attributes: I) -> String
    where
        I: IntoIterator<Item = &'a Attribute>,
    {
        let attributes = attributes.into_iter();

        let mut css_variables = vec![];

        let mut result = String::new();

        // Looping through the list of attributes:
        //
        // * Empty keys are skipped. This permits users to use `unwrap_or_default()`-like patterns
        // when conditionally setting attributes
        //
        // * data-* attributes have to be expressed in Rust as data_ because dashes are not valid
        // Rust identifiers. When displaying attributes, the underscore is converted back to a
        // dash.
        //
        // * CSS variables need to be collected so they can be written as one style="..." attribute
        for attr in attributes {
            match attr {
                Attribute::Bool(Bool(b)) => {
                    if b.is_empty() {
                        continue;
                    }
                    // data-* attributes have to be written as data_*
                    if b.contains('_') {
                        write!(result, " {}", b.replace('_', "-")).unwrap();
                    } else {
                        write!(result, " {}", b).unwrap();
                    }
                }
                Attribute::Regular(Regular(k, v)) => {
                    if k.is_empty() {
                        continue;
                    }
                    if k.contains('_') {
                        write!(result, " {}", k.replace('_', "-")).unwrap();
                    } else {
                        write!(result, " {}", k).unwrap();
                    }
                    write!(result, "=\"{}\"", v).unwrap();
                }
                Attribute::Variable(var) => {
                    if var.0.is_empty() {
                        continue;
                    }
                    css_variables.push(var);
                }
            }
        }

        let mut definitions = String::new();
        for Variable(k, v) in css_variables {
            write!(definitions, "--{}: {};", k, v).unwrap();
        }
        if !definitions.is_empty() {
            write!(result, " style=\"{}\"", definitions).unwrap();
        }
        result
    }
}

/// Creates a `Vec` with the given [attributes](crate::Attribute)
///
/// `attr!` allows `Vec<Attribute>` to be defined using similar syntax as you would to define a list
/// of attributes in HTML.
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
/// written as `data_*` because the dash (`-`) cannot be part of a rust identifier.
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
        $crate::attr_impl!([$($attr,)* $crate::Attribute::new_regular(stringify!($name), $value)] -> $($rest)*)
    };
    ([$($attr:expr),*] -> $name:ident = $value:expr) => {
        $crate::attr_impl!([$($attr,)* $crate::Attribute::new_regular(stringify!($name), $value)] ->)
    };

    // Match boolean attributes
    ([$($attr:expr),*] -> $name:ident , $($rest:tt)*) => {
        $crate::attr_impl!([$($attr,)* $crate::Attribute::new_boolean(stringify!($name))] -> $($rest)*)
    };
    ([$($attr:expr),*] -> $name:ident) => {
        $crate::attr_impl!([$($attr,)* $crate::Attribute::new_boolean(stringify!($name))] ->)
    };

    // Create vec once there is no more input to consume
    ([$($attr:expr),*] ->) => {
        <Vec<$crate::Attribute>>::from([$($attr,)*])
    };
}
