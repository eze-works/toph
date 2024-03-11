//! An API for building  HTML documents.
//!
//! - [Safely](#xss-prevention) set attributes and content on HTML elements.
//! - Link [css](crate::Node::css) & [javascript](crate::Node::with_js) snippets to HTML
//! elements, such that those snippets only appear if the linked element is displayed.
//!
//!
//! ## Example
//!
//! ```
//! use toph::{attr, Node, tag::*};
//!
//! let navigation = [("Home", "/"), ("Posts", "/posts")];
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
//!                 h1_.css("h1 { text-decoration: underline; }")
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
//!         <a href="/">
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
//! `'static` string slices in Rust generally[^1] correspond to string literals. This crate takes
//! advantage of this by allowing `'static` string slices to appear anywhere in the HTML un-encoded
//! since by their nature they cannot contain user input.
//!
//! A handful of additional measures are taken to protect against Cross-site scripting attacks
//! (XSS):
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
//!     span.write_to_string(false),
//!     r#"<span class="&quot; onclick=&quot;alert(1)&quot;">&gt;&lt;script&gt;alert(1)</span>"#
//! );
//!
//! assert_eq!(
//!     anchor.write_to_string(false),
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
//!     html.write_to_string(false),
//!     r#"<button></button>"# // the attribute is ignored
//! );
//!
//! // You can still set any atribute you want using `'static` string slices
//! let mut html = button_.with(attr![onclick="alert(1)"]);
//! assert_eq!(
//!     html.write_to_string(false),
//!     r#"<button onclick="alert(1)"></button>"#
//! );
//! ```
//!
//! - For owned strings, A subset of the safe attributes are recognized as URL attributes. For
//! these, there is a whitelist of allowed schemes. Notably, this excludes `javascript:`.
//!
//! ```
//! use toph::{attr, tag::*};
//!
//!
//! let mut html = a_.with(attr![href=String::from("mailto:a.com")]);
//! assert_eq!(
//!     html.write_to_string(false),
//!     r#"<a href="mailto:a.com"></a>"#
//! );
//!
//! let mut html = a_.with(attr![href=String::from("javascript:alert(1)")]);
//! assert_eq!(
//!     html.write_to_string(false),
//!     "<a></a>"
//! );
//! ```
//!
//! - JavaScript & CSS snippets can only be set using `'static` string slices.
//!   - It is frequently useful to parameterize styles though, so the [`var`](crate::Node::var)
//!   method is provided.
//!
//! ```
//! use toph::{tag::*};
//!
//! let user_input = "1rem";
//! let css = format!("p {{ font-size: {}; }}", user_input);
//!
//! // This does not compile
//! // let mut html = html_.set([ head_, p_.css(&css)]);
//!
//! // Neither does this. good try though
//! // let mut html = html_.set([ head_, p_.css(css.leak())]);
//!
//! // this compiles .. but won't actually do anything
//! let mut html = html_.set([ head_, p_.css(css)]);
//! assert_eq!(
//!   html.write_to_string(false),
//!   "<html><head></head><p></p></html>"
//! );
//!
//! // The only way to set snippets is through literal strings.
//! let css = "p { font-size: var(--font-size); }";
//! let mut html = html_.set([
//!     head_,
//!     p_.css(css).var("font-size", user_input),
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
//!
//!
//! [^1]: [`String::leak`] does allow you to get a `&'static mut str`. However even that won't work
//! with this crate because the APIs expect a shared reference.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod allowlist;
mod encode;
pub mod every_layout;
mod node;

pub use node::{attribute::Attribute, tag, Element, Node, Text};
