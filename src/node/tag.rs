//! HTML Document generation through function composition
//!
//! Functions in this module are named after the html node they generate. Nest function calls to
//! generate a tree:
//!
//! TODO
//!
//!
//! use tag::*
//!
//! div_
//!     .class_("myclass")
//!     .id_("my-id")
//!     
//!
//! )

use super::*;
use super::attribute::Attribute;

pub struct TagProps {
    child: Option<Box<Node>>,
    attributes: Vec<Attribute>,
}

impl<I> From<I> for TagProps
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

impl From<Vec<Attribute>> for TagProps {
    fn from(value: Vec<Attribute>) -> Self {
        Self {
            child: None,
            attributes: value,
        }
    }
}

impl From<(Vec<Attribute>,)> for TagProps {
    fn from(value: (Vec<Attribute>,)) -> Self {
        TagProps::from(value.0)
    }
}

impl<I> From<(Vec<Attribute>, I)> for TagProps
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

pub fn empty_() -> Node {
    Node::Empty
}

pub fn custom_(tag: &'static str, props: impl Into<TagProps>) -> Node {
    let props = props.into();
    Node::Element(Element {
        tag,
        child: props.child,
        attributes: props.attributes,
        css: Css::default(),
        js: Js::default(),
    })
}

pub fn doctype_() -> Node {
    Node::Element(Element {
        tag: "!DOCTYPE",
        child: Some(Box::new(Node::Empty)),
        attributes: vec![Attribute::new_boolean("html")],
        css: Css::default(),
        js: Js::default(),
    })
}

pub fn html_(props: impl Into<TagProps>) -> Node {
    let props = props.into();
    let mut node = Node::Element(Element {
        tag: "html",
        child: props.child,
        attributes: props.attributes,
        css: Css::default(),
        js: Js::default(),
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
    ($($tag:ident), +) => {
        $(
            #[inline]
            pub fn $tag(props: impl Into<TagProps>) -> Node {
                let props = props.into();
                let tag = stringify!($tag).strip_suffix("_").expect("tag names must end in an underscore");
                Node::Element(Element {
                    tag,
                    child: props.child,
                    attributes: props.attributes,
                    css: Css::default(),
                    js: Js::default(),
                })

            }
        )+
    }
}

#[rustfmt::skip]
impl_tag![
    // main root
    // html_
    // document metadata
    base_, head_, link_, meta_, style_, title_,
    // sectioning root
    body_,
    // content sectioning
    address_, article_, aside_, footer_, header_, h1_, h2_, h3_, h4_, h5_, h6_, main_, nav_, section_,
    // text content
    blockquote_, dd_, div_, dl_, dt_, figcaption_, figure_, hr_, li_, menu_, ol_, p_, pre_, ul_,
    // inline text semantics
    a_, abbr_, b_, bdi_, bdo_, br_, cite_, code_, data_, dfn_, em_, i_, kbd_, mark_, q_, rp_, rt_, ruby_, s_, samp_,
    small_, span_, strong_, sub_, sup_, time_, u_, var_, wbr_,
    // image and multimedia
    area_, audio_, img_, map_, track_, video_,
    // embedded content
    embed_, iframe_, object_, picture_, portal_, source_,
    // svg and mathml
    svg_, math_,
    // scripting
    canvas_, noscript_, script_,
    // demarcating edits
    del_, ins_,
    // table content
    caption, col_, colgroup_, table_, tbody_, td_, tfoot_, th_, thead_, tr_,
    // forms
    button_, datalist_, fieldset_, form_, input_, label_, legend_, meter_, optgroup_, option_, output_,
    progress_, select_, textarea_,
    // interactive elements
    details_, dialog_, summary_,
    // web components
    slot_, template_
];
