//! HTML Elements
//!
//! The constants in this module are of type [`Node`](crate::Node) and are named after the HTML
//! tags they generate.

use super::*;

/// Creates an HTML Node with a custom tag name.
pub fn custom_(tag: &'static str) -> Node {
    Node::Element(Element {
        tag,
        child: None,
        attributes: vec![],
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
                attributes: vec![],
                child: None,
            });
        }
    }
}

/// The <!DOCTYPE> element
#[allow(non_upper_case_globals)]
pub const doctype_: Node = Node::Element(Element {
    tag: "!DOCTYPE html",
    attributes: vec![],
    child: None,
});

#[rustfmt::skip]
impl_tag![
    // main root
    html,
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
