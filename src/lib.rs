mod attribute;
mod encode;
mod html;
mod node;

#[doc(hidden)]
pub use attribute::Attribute;

pub use node::{raw_text, text, Element, Fragment, Node, Text};

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDocTests;
