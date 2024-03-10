//! HTML document building through function composition
//!
//! Functions in this module are named after the html tag they generate. Nest function calls to
//! generate a tree:
//!
//! ```
//! use toph::tag::*;
//!
//! let tree = html_([
//!     head_(
//!         title_("My Website")
//!     ),
//!     body_(
//!         h1_("My header")
//!     )
//! ]);
//! ```
//!
//! With the exception of [`custom_()`] and [`doctype_()`] functions, all functions in this module
//! accept exactly __one__ argument.
//!
//! You can pass in anything that can be converted to a [`Node`](crate::Node):
//!
//! ```
//! use toph::tag::*;
//!
//! // A single 'static string slice
//! span_("hello");
//!
//! // An owned string
//! span_(String::from("hello"));
//!
//! // An empty array (implies no child nodes)
//! span_([]);
//!
//! // An array of nodes
//! span_([div_([]), span_([])]);
//!
//! // 'static string slices and owned strings can be
//! // converted to nodes, so this also works
//! span_([
//!     div_([]),
//!     "bare string".into(),
//!     span_([])
//! ]);
//!
//! ```
//!
//! You can pass in a [list of attributes](crate::__):
//!
//! ```
//! use toph::{__, tag::*};
//! span_(__![class="card", hidden]);
//! ```
//!
//! You can pass both of the above if you place them in a tuple
//!
//! ```
//! use toph::{__, tag::*};
//!
//! span_((
//!     __![class="card",hidden],
//!     "hello"
//! ));
//!
//! span_((
//!     __![class="card",hidden],
//!     [
//!         div_([]),
//!         span_([]),
//!     ]
//! ));
//! ```
//!
//! ## XSS Prevention
//!
//! `'static` string slices in Rust generally[^1] correspond to string literals. This crate takes
//! advantage of this by allowing `'static` string slices to appear anywhere in the HTML un-encoded
//! since by their nature they cannot contain user input, and so are safe.
//!
//! For owned strings, a handful of measures are taken to protect against Cross-site scripting
//! attacks (XSS):
//!
//! - Owned strings are appropriately encoded in HTML, attribute and URL contexts:
//! ```
//! use toph::{__, tag::*};
//!
//! let xss_attr_attempt = String::from(r#"" onclick="alert(1)""#);
//! let xss_attempt = String::from(r#"><script>alert(1)"#);
//! let url = String::from("/path with space");
//!
//! let mut span = span_((
//!     __![class=xss_attr_attempt],
//!     xss_attempt
//! ));
//!
//! let mut anchor = a_((
//!     __![href=url],
//!     "A link"
//! ));
//!
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
//! use toph::{__, tag::*};
//!
//! let user_input = String::from("alert(1)");
//! let mut html = button_(__![onclick=user_input]);
//! assert_eq!(
//!     html.write_to_string(),
//!     r#"<button></button>"# // the attribute is ignored
//! );
//!
//! // You can still set any atribute you want using `'static` string slices
//! let mut html = button_(__![onclick="alert(1)"]);
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
//! use toph::{__, tag::*};
//!
//!
//! let mut html = a_(__![href=String::from("mailto:a.com")]);
//! assert_eq!(
//!     html.write_to_string(),
//!     r#"<a href="mailto:a.com"></a>"#
//! );
//!
//! let mut html = a_(__![href=String::from("javascript:alert(1)")]);
//! assert_eq!(
//!     html.write_to_string(),
//!     "<a></a>"
//! );
//! ```
//!
//! [^1]: [`String::leak`] does allow you to get a `&'static mut str`. However even that won't work
//! with this crate because the APIs expect a shared reference.

use super::attribute::Attribute;
use super::*;

/// The type used to by tag functions to create HTML [`Node`]s
pub struct TagArgs {
    child: Option<Box<Node>>,
    attributes: Vec<Attribute>,
}

impl<I> From<I> for TagArgs
where
    I: Into<Node>,
{
    fn from(value: I) -> Self {
        let child = value.into();
        Self {
            child: Some(Box::new(child)),
            attributes: vec![],
        }
    }
}

impl From<Vec<Attribute>> for TagArgs {
    fn from(value: Vec<Attribute>) -> Self {
        Self {
            child: None,
            attributes: value,
        }
    }
}

impl From<(Vec<Attribute>,)> for TagArgs {
    fn from(value: (Vec<Attribute>,)) -> Self {
        TagArgs::from(value.0)
    }
}

impl<I> From<(Vec<Attribute>, I)> for TagArgs
where
    I: Into<Node>,
{
    fn from(value: (Vec<Attribute>, I)) -> Self {
        let child = value.1.into();
        Self {
            child: Some(Box::new(child)),
            attributes: value.0,
        }
    }
}

/// Creates an HTML Node with a custom tag name.
pub fn custom_(tag: &'static str, args: impl Into<TagArgs>) -> Node {
    let args = args.into();
    Node::Element(Element {
        tag,
        child: args.child,
        attributes: args.attributes,
    })
}

/// Creates a `<!DOCTYPE html>` node
pub fn doctype_() -> Node {
    Node::Element(Element {
        tag: "!DOCTYPE",
        child: None,
        attributes: vec![Attribute::new_boolean("html")],
    })
}

/// Creates a `<html>` node
pub fn html_(args: impl Into<TagArgs>) -> Node {
    let args = args.into();
    let mut node = Node::Element(Element {
        tag: "html",
        child: args.child,
        attributes: args.attributes,
    });

    include_assets(&mut node);

    node
}

// Extracts all css & javascript assets from the subtrees and places them in <style> & <script>
// nodes
fn include_assets(node: &mut Node) {
    // Get assets
    let mut collector = visitor::AssetCollector::new();
    visitor::visit_nodes(node, &mut collector).expect("collecting assets does not fail");

    let mut style = None;
    let mut script = None;

    let script_fragments = collector.js.into_iter().map(script_).collect::<Vec<_>>();
    let style_fragments = collector.css.into_iter().map(style_).collect::<Vec<_>>();

    if script_fragments.len() > 0 {
        script = Some(Node::Fragment(Fragment(script_fragments)));
    }

    if style_fragments.len() > 0 {
        style = Some(Node::Fragment(Fragment(style_fragments)));
    }

    // Insert them into the tree
    let inserter = visitor::AssetInserter::new(style, script);
    visitor::visit_nodes(node, inserter).expect("inserting nodes does not fail");
}

macro_rules! impl_tag {
    ($($tag:ident),+) => {
        $(
            impl_tag!(@withdoc $tag, concat!("Creates a `", stringify!($tag), "` HTML element"));
        )+
    };
    (@withdoc $tag:ident, $doc:expr) => {
        paste::paste!{
            #[doc = $doc]
            #[inline]
            pub fn [<$tag _>](args: impl Into<TagArgs>) -> Node {
                let args = args.into();
                Node::Element(Element {
                    tag: stringify!($tag),
                    child: args.child,
                    attributes: args.attributes,
                })
            }
        }
    }
}

#[rustfmt::skip]
impl_tag![
    // main root
    // html_
    // document metadata
    base, head, link, meta, style, title,
    // sectioning root
    body,
    // content sectioning
    address, article, aside, footer, header, h1, h2, h3, h4, h5, h6, main, nav, section,
    // text content
    blockquote, dd, div, dl, dt, figcaption, figure, hr, li, menu, ol, p, pre, ul,
    // inline text semantics
    a, abbr, b, bdi, bdo, br, cite, code, data, dfn, em, i, kbd, mark, q, rp, rt, ruby, s, samp,
    small, span, strong, sub, sup, time, u, var, wbr,
    // image and multimedia
    area, audio, img, map, track, video,
    // embedded content
    embed, iframe, object, picture, portal, source,
    // svg and mathml
    svg, math,
    // scripting
    canvas, noscript, script,
    // demarcating edits
    del, ins,
    // table content
    caption, col, colgroup, table, tbody, td, tfoot, th, thead, tr,
    // forms
    button, datalist, fieldset, form, input, label, legend, meter, optgroup, option, output,
    progress, select, textarea,
    // interactive elements
    details, dialog, summary,
    // web components
    slot, template
];
