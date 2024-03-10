//! Build HTML documents in Rust
//!
//! ```
//! use toph::{attr, tag::*};
//!
//! let navigation = [("Home", "/"), ("Posts", "/posts")];
//! let doc = [
//!     doctype_(),
//!     html_((attr![lang="en"],
//!         [
//!             head_(title_("My Webpage")),
//!             body_([
//!                 ul_((attr![id="navigation"],
//!                     navigation.into_iter().map(|(caption, url)| {
//!                         li_(a_((attr![href=url], caption)))
//!                     })
//!                     .collect::<Vec<_>>()
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
