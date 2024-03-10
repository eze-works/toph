//! Build HTML documents in Rust
//!
//! ```
//! use toph::{__, tag::*};
//!
//! let navigation = [("Home", "/"), ("Posts", "/posts")];
//! let doc = [
//!     doctype_(),
//!     html_((
//!         __![lang="en"],
//!         [
//!             head_(title_("My Webpage")),
//!             body_([
//!                 ul_((
//!                     __![id="navigation"],
//!                     navigation.into_iter().map(|(caption, url)| {
//!                         li_(a_((__![href=url], caption)))
//!                     }).collect::<Vec<_>>()
//!                 )),
//!                 h1_("My Webpage")
//!             ])
//!         ]
//!     )),
//! ];
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod allowlist;
mod encode;
mod node;

pub use node::{attribute::Attribute, tag, Element, Node, Text};
