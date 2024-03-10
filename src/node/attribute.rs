use crate::{allowlist, encode};
use std::borrow::Cow;
use std::fmt::Display;

/// An HTML Attribute
///
/// You will generally be using the [attribute list builder](crate::__) to create HTML attributes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Attribute {
    /// A boolean attribute (e.g. `<input hidden>`
    Bool(BooleanAttribute),
    /// A regular attribute (e.g. `<span class="something"></span>`)
    Regular(RegularAttribute),
    /// Special attribute for storing css associated with a node.
    Css(&'static str),
    /// Special attribute for storing js associated with a node
    Js(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BooleanAttribute(&'static str);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct RegularAttribute(&'static str, Cow<'static, str>);

impl Attribute {
    /// Creates a new regular attribute with a key & value.
    ///
    /// If the value is not a `'static` string slice, it will be html encoded
    pub fn new(key: &'static str, value: impl Into<Cow<'static, str>>) -> Self {
        let key = key.trim();
        let value = value.into();
        let attribute = match value {
            v @ Cow::Borrowed(_) => Attribute::Regular(RegularAttribute(key, v)),
            v @ Cow::Owned(_) => {
                let value = v.into_owned();
                let value = value.trim();

                if allowlist::URL_ATTRIBUTES.contains(&key) {
                    if let Some(url) = encode::url(value) {
                        Attribute::Regular(RegularAttribute(key, url.into()))
                    } else {
                        Attribute::Regular(RegularAttribute("", "".into()))
                    }
                } else if allowlist::ALLOWED_ATTR_NAMES.contains(&key) || key.starts_with("data_") {
                    let encoded_value = encode::attr(value);
                    Attribute::Regular(RegularAttribute(key, encoded_value.into()))
                } else {
                    Attribute::Regular(RegularAttribute("", "".into()))
                }
            }
        };

        attribute
    }

    /// Creates a new boolean attribute
    pub fn new_boolean(key: &'static str) -> Self {
        Attribute::Bool(BooleanAttribute(key))
    }
}

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Attribute::Bool(BooleanAttribute(b)) => {
                if b.is_empty() {
                    return Ok(());
                }

                if b.contains("_") {
                    write!(f, " {}", b.replace("_", "-"))?;
                } else {
                    write!(f, " {}", b)?;
                }
                Ok(())
            }
            Attribute::Regular(RegularAttribute(k, v)) => {
                if k.is_empty() {
                    return Ok(());
                }

                if k.contains("_") {
                    write!(f, " {}", k.replace("_", "-"))?;
                } else {
                    write!(f, " {}", k)?;
                }
                write!(f, "=\"{}\"", v)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// Creates a `Vec` with the given [attributes](crate::Attribute)
///
/// `__!` allows `Vec<Attribute>` to be defined using similar syntax as you would to define a list
/// of attributes in HTML.
///
/// The macro has a single form:
/// ```
/// use toph::__;
/// let list = __![class="my-class", async, checked, id="my-id"];
/// ```
///
/// You may have a trailing comma:
///
/// ```
/// use toph::__;
/// let list = __![hidden,];
/// ```
///
/// The attribute key must be a valid rust identifier. This means that `data-*` attributes must be
/// written as `data_*` because the dash (`-`) cannot be part of a rust identifier.
///
/// ```
/// use toph::__;
/// // let list = __![data-custom="true"]; // Syntax error
/// let list = __![data_custom="true"];
/// ```
///
/// The attribute value can be either:
/// - A string slice with `'static` lifetime
/// - An owned `String`
///
/// Using a non  non `'static` string slice will cause a borrow-checker error:
///
/// ```
/// use toph::__;
///
/// let string_slice = "hello";
/// let heap_allocated_string = "world".to_string();
/// let reference_to_heap_allocated_string = &heap_allocated_string;
///
/// let list = __![class=string_slice]; // OK
/// let list = __![class=heap_allocated_string]; // OK
/// // let list = __![class=reference_to_heap_allocated_string];
/// // Error: borrowed value does not live long enough
/// ```
#[macro_export]
macro_rules! __ {
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
// __![async, class = "hidden", checked]
// ```
// This is the trace:
//
// ```
// expanding `__! { async, class = "hidden", checked }`
// to `{ $crate :: __! ([] -> async, class = "hidden", checked) }`
//
// expanding `__! { [] -> async, class = "hidden", checked }`
// to `$crate :: __! ([$crate :: Attribute :: new_boolean(stringify! (async))] -> class = "hidden", checked)`
//
// expanding `__! { [$crate :: Attribute :: new_boolean(stringify! (async))] -> class = "hidden", checked }`
// to `$crate :: __! ([crate::Attribute::new_boolean(stringify!(async)), $crate :: Attribute :: new(stringify! (class), "hidden")] -> checked)`
//
// expanding `__! { [crate::Attribute::new_boolean(stringify!(async)), $crate :: Attribute :: new(stringify! (class), "hidden")] -> checked }`
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
    // Match the @css key
    ([$($attr:expr),*] -> @css = $value:expr, $($rest:tt)*) => {
        $crate::attr_impl!([$($attr,)* $crate::Attribute::Css($value)] -> $($rest)*)
    };
    ([$($attr:expr),*] -> @css = $value:expr) => {
        $crate::attr_impl!([$($attr,)* $crate::Attribute::Css($value)] ->)
    };

    // Match the @js key
    ([$($attr:expr),*] -> @js = $value:expr, $($rest:tt)*) => {
        $crate::attr_impl!([$($attr,)* $crate::Attribute::Js($value)] -> $($rest)*)
    };
    ([$($attr:expr),*] -> @js = $value:expr) => {
        $crate::attr_impl!([$($attr,)* $crate::Attribute::Js($value)] ->)
    };

    // Match regular key/value attributes
    ([$($attr:expr),*] -> $name:ident = $value:expr , $($rest:tt)*) => {
        $crate::attr_impl!([$($attr,)* $crate::Attribute::new(stringify!($name), $value)] -> $($rest)*)
    };
    ([$($attr:expr),*] -> $name:ident = $value:expr) => {
        $crate::attr_impl!([$($attr,)* $crate::Attribute::new(stringify!($name), $value)] ->)
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
