//! An API for building HTML documents in Rust.
//!
//! - It's Just Rust Code.
//! - [Safely](#xss-prevention) set attributes and content on HTML elements.
//!
//! ## Example
//!
//! ```
//! use toph::{attr, Node, tag::*};
//!
//! let navigation = [("Home", "/about me"), ("Posts", "/posts")];
//! let mut doc = [
//!     doctype_,
//!     html_.with(attr![lang="en"])
//!         .set([
//!             head_.set([title_.set(["My Webpage"])]),
//!             body_.set([
//!                 ul_.with(attr![id="navigation"])
//!                     .set(
//!                         navigation.into_iter().map(|(caption, url)| {
//!                             li_.set([a_.with(attr![href=url]).set([caption])])
//!                         }).collect::<Vec<_>>()
//!                     ),
//!                 h1_.set(["My Webpage"])
//!             ])
//!         ])
//! ];
//!
//! assert_eq!(
//!     Node::render_pretty(doc),
//!     r#"<!DOCTYPE html>
//! <html lang="en">
//!   <head>
//!     <title>
//!       My Webpage
//!     </title>
//!   </head>
//!   <body>
//!     <ul id="navigation">
//!       <li>
//!         <a href="/about%20me">
//!           Home
//!         </a>
//!       </li>
//!       <li>
//!         <a href="/posts">
//!           Posts
//!         </a>
//!       </li>
//!     </ul>
//!     <h1>
//!       My Webpage
//!     </h1>
//!   </body>
//! </html>
//! "#);
//! ```
//!
//! ## XSS Prevention
//!
//! Strings are appropriately encoded in HTML, attribute and URL contexts:
//!
//! ```
//! use toph::{attr, tag::*, Node};
//!
//! let xss_attr_attempt = r#"" onclick="alert(1)""#;
//! let xss_attempt = r#"<script>alert(1)"#;
//! let url = "/path with space";
//!
//! let mut span = span_
//!     .with(attr![class=xss_attr_attempt])
//!     .set([xss_attempt]);
//!
//! let mut anchor = a_
//!     .with(attr![href=url])
//!     .set(["A link"]);
//!
//! assert_eq!(
//!     Node::render([span]),
//!     r#"<span class="&quot; onclick=&quot;alert(1)&quot;">&lt;script&gt;alert(1)</span>"#
//! );
//!
//! assert_eq!(
//!     Node::render([anchor]),
//!     r#"<a href="/path%20with%20space">A link</a>"#
//! );
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod encode;
mod node;

pub use node::{tag, Node};

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDocTests;
