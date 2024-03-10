//! Build HTML documents in Rust
//!
//! ```
//! use toph::{attr, tag::*};
//!
//! let navigation = [("Home", "/"), ("Posts", "/posts")];
//! let doc = [
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
//!                 h1_.set("My Webpage")
//!             ])
//!         ])
//! ];
//! ```
//!
//! ## XSS Prevention
//!
//! `'static` string slices in Rust generally[^1] correspond to string literals. This crate takes
//! advantage of this by allowing `'static` string slices to appear anywhere in the HTML un-encoded
//! since by their nature they cannot contain user input.
//!
//! For owned strings, a handful of measures are taken to protect against Cross-site scripting
//! attacks (XSS):
//!
//! - Owned strings are appropriately encoded in HTML, attribute and URL contexts:
//! ```
//! use toph::{attr, tag::*};
//!
//! let xss_attr_attempt = String::from(r#"" onclick="alert(1)""#);
//! let xss_attempt = String::from(r#"><script>alert(1)"#);
//! let url = String::from("/path with space");
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
//!     span.write_to_string(),
//!     r#"<span class="&quot; onclick=&quot;alert(1)&quot;">&gt;&lt;script&gt;alert(1)</span>"#
//! );
//!
//! assert_eq!(
//!     anchor.write_to_string(),
//!     r#"<a href="/path%20with%20space">A link</a>"#
//! );
//! ```
//!
//! - Owned strings may only be used to set HTML attributes that are considered [safe
//! sinks](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html#output-encoding-for-html-attribute-contexts).
//! Notably, none of the event attributes (`on*`) are on this list.
//!
//! ```
//! use toph::{attr, tag::*};
//!
//! let user_input = String::from("alert(1)");
//! let mut html = button_.with(attr![onclick=user_input]);
//! assert_eq!(
//!     html.write_to_string(),
//!     r#"<button></button>"# // the attribute is ignored
//! );
//!
//! // You can still set any atribute you want using `'static` string slices
//! let mut html = button_.with(attr![onclick="alert(1)"]);
//! assert_eq!(
//!     html.write_to_string(),
//!     r#"<button onclick="alert(1)"></button>"#
//! );
//! ```
//!
//! - A subset of the safe attributes are recognized as URL attributes. For these, there is a
//! whitelist of allowed schemes. Notably, this excludes `javascript:`
//!
//! ```
//! use toph::{attr, tag::*};
//!
//!
//! let mut html = a_.with(attr![href=String::from("mailto:a.com")]);
//! assert_eq!(
//!     html.write_to_string(),
//!     r#"<a href="mailto:a.com"></a>"#
//! );
//!
//! let mut html = a_.with(attr![href=String::from("javascript:alert(1)")]);
//! assert_eq!(
//!     html.write_to_string(),
//!     "<a></a>"
//! );
//! ```
//!
//! [^1]: [`String::leak`] does allow you to get a `&'static mut str`. However even that won't work
//! with this crate because the APIs expect a shared reference.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod allowlist;
mod encode;
mod node;

pub use node::{attribute::Attribute, tag, Element, Node, Text};
