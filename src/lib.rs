//! An API for building HTML documents in Rust.
//!
//! - Macro use kept to a minimum (there is just one macro for setting attributes). It's Just Rust
//! Code.
//! - [Safely](#xss-prevention) set attributes and content on HTML elements.
//! - Link [css](crate::Node::stylesheet) & [javascript](crate::Node::js) snippets to HTML
//! elements, such that those snippets appear when the linked element is displayed.
//!
//! The crate also implements [a set of layout primitives](crate::layout) so you don't have to
//!
//! ## Example
//!
//! ```
//! use toph::{attr, Node, tag::*};
//!
//! let navigation = [("Home", "/about me"), ("Posts", "/posts")];
//! let mut doc = Node::from([
//!     doctype_,
//!     html_.with(attr![lang="en"])
//!         .set([
//!             head_.set(title_.set("My Webpage")),
//!             body_.set([
//!                 ul_.with(attr![id="navigation"])
//!                     .set(
//!                         navigation.into_iter().map(|(caption, url)| {
//!                             li_.set(a_.with(attr![href=url]).set(caption))
//!                         }).collect::<Vec<_>>()
//!                     ),
//!                 h1_.stylesheet("h1 { text-decoration: underline; }")
//!                     .set("My Webpage")
//!             ])
//!         ])
//! ]);
//!
//! assert_eq!(
//!     doc.write_to_string(true),
//!     r#"<!DOCTYPE html>
//! <html lang="en">
//!   <head>
//!     <style>
//!       h1 { text-decoration: underline; }
//!     </style>
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
//! A couple measures are taken to protect against Cross-site scripting attacks (XSS):
//!
//! - Strings are appropriately encoded in HTML, attribute and URL contexts:
//!
//! ```
//! use toph::{attr, tag::*};
//!
//! let xss_attr_attempt = r#"" onclick="alert(1)""#;
//! let xss_attempt = r#"<script>alert(1)"#;
//! let url = "/path with space";
//!
//! let mut span = span_
//!     .with(attr![class=xss_attr_attempt])
//!     .set(xss_attempt);
//!
//! let mut anchor = a_
//!     .with(attr![href=url])
//!     .set("A link");
//!
//! assert_eq!(
//!     span.write_to_string(false),
//!     r#"<span class="&quot; onclick=&quot;alert(1)&quot;">&lt;script&gt;alert(1)</span>"#
//! );
//!
//! assert_eq!(
//!     anchor.write_to_string(false),
//!     r#"<a href="/path%20with%20space">A link</a>"#
//! );
//! ```
//!
//! - JavaScript & CSS snippets can only be set using literal (i.e. `'static`) string slices.
//!   - It is frequently useful to parameterize styles though, so the [`var`](crate::Node::var)
//!   method is provided.
//!   - To include files, use [`include_str`]
//!
//! ```
//! use toph::{tag::*};
//!
//! let user_input = "1rem";
//! let css = format!("p {{ font-size: {}; }}", user_input);
//!
//! // This does not compile
//! // let mut html = html_.set([ head_, p_.stylesheet(css)]);
//! // Neither does this
//! // let mut html = html_.set([ head_, p_.stylesheet(&css)]);
//!
//! // Technically, you _could_ leak the string...
//! // Why you would want to leak memory for this purpose is beyond me.
//! // You are on your own.
//! // let mut html = html_.set([ head_, p_.stylesheet(css.leak())]);
//!
//! // Set snippets using string literals
//! // Parameterize with css custom variables & `var()`
//! let css = "p { font-size: var(--font-size); }";
//! let mut html = html_.set([
//!     head_,
//!     p_.stylesheet(css).var("font-size", user_input),
//! ]);
//! assert_eq!(
//!   html.write_to_string(true),
//!   r#"<html>
//!   <head>
//!     <style>
//!       p { font-size: var(--font-size); }
//!     </style>
//!   </head>
//!   <p style="--font-size: 1rem;">
//!   </p>
//! </html>
//!"#);
//!
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod encode;
pub mod layout;
mod node;

pub use node::{tag, Element, Fragment, Node, Text};
