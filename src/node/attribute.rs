use crate::{allowlist, encode};
use std::borrow::Cow;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Attribute {
    Bool(BooleanAttribute),
    Regular(RegularAttribute),
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BooleanAttribute(&'static str);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct RegularAttribute(&'static str, Cow<'static, str>);

impl Attribute {
    pub fn new(key: &'static str, value: impl Into<Cow<'static, str>>) -> Self {
        let value = value.into();
        let attribute = match value {
            v @ Cow::Borrowed(_) => Attribute::Regular(RegularAttribute(key, v)),
            v @ Cow::Owned(_) => {
                let value = v.into_owned();

                if allowlist::URL_ATTRIBUTES.contains(&key) {
                    if let Some(url) = encode::url(&value) {
                        Attribute::Regular(RegularAttribute(key, url.into()))
                    } else {
                        Attribute::Empty
                    }
                } else if allowlist::ALLOWED_ATTR_NAMES.contains(&key) || key.starts_with("data_") {
                    let encoded_value = encode::attr(&value);
                    Attribute::Regular(RegularAttribute(key, encoded_value.into()))
                } else {
                    Attribute::Empty
                }
            }
        };

        attribute
    }

    pub fn new_boolean(key: &'static str) -> Self {
        Attribute::Bool(BooleanAttribute(key))
    }
}

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Attribute::Empty => Ok(()),
            Attribute::Bool(BooleanAttribute(b)) => {
                if b.contains("_") {
                    write!(f, "{}", b.replace("_", "-"))?;
                } else {
                    write!(f, "{}", b)?;
                }
                Ok(())
            }
            Attribute::Regular(RegularAttribute(k, v)) => {
                if k.contains("_") {
                    write!(f, "{}", k.replace("_", "-"))?;
                } else {
                    write!(f, "{}", k)?;
                }
                write!(f, "=\"{}\"", v)?;
                Ok(())
            }
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
/// use html_string::__;
/// let list = __![class="my-class", async, checked, id="my-id"];
/// ```
///
/// You may have a trailing comma:
///
/// ```
/// use html_string::__;
/// let list = __![hidden,];
/// ```
///
/// The attribute key must be a valid rust identifier. This means that `data-*` attributes must be
/// written as `data_*` because the dash (`-`) cannot be part of a rust identifier.
///
/// ```
/// use html_string::__;
/// // let list = __![data-custom="true"]; // Syntax error
/// let list = __![data_custom="true"];
/// ```
///
/// The attribute value can be either:
/// - A string slice with `'static` lifetime (i.e. known at compile-time)
/// - An owned `String`
///
/// Using a non  non `'static` string slice will cause a borrow-checker error:
///
/// ```
/// use html_string::__;
///
/// let compile_time_known_string = "hello";
/// let heap_allocated_string = "world".to_string();
/// let reference_to_heap_allocated_string = &heap_allocated_string;
///
/// let list = __![class=compile_time_known_string]; // OK
/// let list = __![class=heap_allocated_string]; // OK
/// // let list = __![class=reference_to_heap_allocated_string];
/// // Error: borrowed value does not live long enough
/// ```
#[macro_export]
macro_rules! __ {
    ([$($attr:expr),*] -> $name:ident = $value:expr) => {
        [$($attr,)* $crate::Attribute::new(stringify!($name), $value)].to_vec()
    };
    ([$($attr:expr),*] -> $name:ident) => {
        [$($attr,)* $crate::Attribute::new_boolean(stringify!($name))].to_vec()
    };
    ([$($attr:expr),*] ->) => {
        <Vec<$crate::Attribute>>::from([$($attr,)*])
    };
    ([$($attr:expr),*] -> $name:ident = $value:expr , $($rest:tt)*) => {
        $crate::__!([$($attr,)* $crate::Attribute::new(stringify!($name), $value)] -> $($rest)*)
    };
    ([$($attr:expr),*] -> $name:ident , $($rest:tt)*) => {
        $crate::__!([$($attr,)* $crate::Attribute::new_boolean(stringify!($name))] -> $($rest)*)
    };
    ($($input:tt)*) => {{
        $crate::__!([] -> $($input)*)
    }}
}
