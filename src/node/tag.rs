//! HTML Elements
//!
//! The constants in this module are of type [`Node`](crate::Node) and are named after the HTML
//! tags they generate.
//!
//! You can also create an HTML element [with a custom tag name](crate::tag::custom_).
//!
//! Missing from this module are constants for the `_script` & `_style` elements. JavaScript & CSS
//! snippets are set using [`Node::with`](crate::Node::with) and the
//! [`attr`](crate::attr#js--css-snippets) macro

use super::*;
use attribute::AttributeMap;
use variable::CSSVariableMap;

/// Creates an HTML Node with a custom tag name.
pub fn custom_(tag: &'static str) -> Node {
    Node::Element(Element {
        tag,
        child: None,
        attributes: AttributeMap::new(),
        assets: vec![],
        variables: CSSVariableMap::new(),
    })
}

macro_rules! impl_tag {
    ($($tag:ident),+) => {
        $(
            impl_tag!(@withdoc $tag, concat!("The `", stringify!($tag), "` HTML element"));
        )+
    };
    (@withdoc $tag:ident, $doc:expr) => {
        paste::paste!{
            #[allow(non_upper_case_globals)]
            #[doc = $doc]
            pub const [<$tag _>]: Node = Node::Element(Element {
                tag: stringify!($tag),
                attributes: AttributeMap::new(),
                assets: vec![],
                child: None,
                variables: CSSVariableMap::new()
            });
        }
    }
}

/// The <!DOCTYPE> element
#[allow(non_upper_case_globals)]
pub const doctype_: Node = Node::Element(Element {
    tag: "!DOCTYPE html",
    attributes: AttributeMap::new(),
    child: None,
    assets: vec![],
    variables: CSSVariableMap::new(),
});

// script_ & style_ tag constants are omitted from the public API
#[allow(non_upper_case_globals)]
pub(crate) const script_: Node = Node::Element(Element {
    tag: "script",
    attributes: AttributeMap::new(),
    assets: vec![],
    child: None,
    variables: CSSVariableMap::new(),
});

/// An empty Node that does not render as anything
#[allow(non_upper_case_globals)]
pub const empty_: Node = Node::Text(Text(String::new()));

#[allow(non_upper_case_globals)]
pub(crate) const style_: Node = Node::Element(Element {
    tag: "style",
    attributes: AttributeMap::new(),
    child: None,
    assets: vec![],
    variables: CSSVariableMap::new(),
});

#[rustfmt::skip]
impl_tag![
    // main root
    html,
    // document metadata
    base, head, link, meta, /*style,*/ title,
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
    canvas, /* script, */ noscript, 
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
