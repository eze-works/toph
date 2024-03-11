mod asset;
pub mod attribute;
pub mod tag;
pub mod visitor;

use crate::encode;
use attribute::Attribute;
use std::borrow::Cow;
use std::io;

/// An HTML Node
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    /// See [`Element`]
    Element(Element),
    /// See [`Text`]
    Text(Text),
    // See [`Fragment`]
    #[doc(hidden)]
    Fragment(Fragment),
}

/// An HTML element. All [tag functions](crate::tag) create an HTML node with this variant
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Element {
    tag: &'static str,
    attributes: Vec<attribute::Attribute>,
    child: Option<Box<Node>>,
    assets: Vec<asset::Asset>,
}

impl Element {
    fn is_void(&self) -> bool {
        matches!(
            self.tag,
            "area"
                | "base"
                | "br"
                | "col"
                | "embed"
                | "hr"
                | "img"
                | "input"
                | "link"
                | "meta"
                | "source"
                | "track"
                | "wbr"
                | "!DOCTYPE html"
        )
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::Text(Text("".into()))
    }
}

/// A text element. This is the variant created when a string is given as an argument to a tag
/// function
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Text(Cow<'static, str>);

// Fragment is a container for multiple `Node`s. It's the variant created with an array of
// nodes is converted to a single node. It is an implementation detail.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Fragment(Vec<Node>);

impl Node {
    /// Converts the tree rooted at this node to an HTML string
    pub fn write_to_string(&mut self, indent: bool) -> String {
        let mut buf = String::new();
        let writer = visitor::HtmlStringWriter::new(&mut buf, indent);
        visitor::visit_nodes(self, writer).expect("printing to a string should not fail");
        buf
    }

    /// Writes the HTML for the tree rooted at this node to anything that implements [`io::Write`]
    pub fn write<W: io::Write>(&mut self, w: W) -> Result<(), io::Error> {
        let writer = visitor::HtmlWriter::new(w);
        visitor::visit_nodes(self, writer)
    }

    /// Sets attributes for the element.
    ///
    /// A mix of boolean & regular attributes can be set
    ///
    /// ```
    /// use toph::{attr, tag::*};
    /// span_.with(attr![class="card", hidden]);
    /// ```
    ///
    /// See the [attr](crate::attr) macro docs for details.
    pub fn with(mut self, attributes: Vec<Attribute>) -> Node {
        if let Self::Element(ref mut el) = self {
            el.attributes = attributes;
        }
        self
    }

    /// Links a css snippet to the Node
    ///
    /// The CSS snippet will be included as a `<style>` element when this Node is in a tree with
    /// both `<html>` & `<head>` tags
    ///
    /// The contents of the snippet are included verbatim.
    ///
    /// # Example
    ///
    /// ```
    /// use toph::{tag::*};
    ///
    /// let mut html = div_.css("div { border: 1px solid black; }");
    /// assert_eq!(html.write_to_string(false), "<div></div>");
    ///
    /// let mut html = html_.set([
    ///     head_,
    ///     div_.css("div { border: 1px solid black; }")
    /// ]);
    /// assert_eq!(
    ///     html.write_to_string(false),
    ///     "<html><head><style>div { border: 1px solid black; }</style></head><div></div></html>"
    /// );
    /// ```
    /// # Note
    ///
    /// CSS snippets are de-duplicated; Including the same snippet multiple times  will
    /// still result in a single `<style>` element
    pub fn css(mut self, css: impl Into<Cow<'static, str>>) -> Node {
        if let Self::Element(ref mut el) = self {
            if let Cow::Borrowed(s) = css.into() {
                el.assets.push(asset::Asset::Css(s));
            }
        }
        self
    }

    /// Links a JavaScript snippet to the Node
    ///
    /// The javascript snippet will be included as a `<script>` element when this Node is in a tree
    /// with both `<html>` & `<body>` tags
    ///
    /// The contents of the snippet are included verbatim
    ///
    /// # Example
    ///
    /// ```
    /// use toph::{tag::*};
    ///
    /// let mut html = div_.js("console.log()");
    /// assert_eq!(html.write_to_string(false), "<div></div>");
    ///
    /// let mut html = html_.set([
    ///     body_.set(div_.js("console.log()"))
    /// ]);
    ///
    /// assert_eq!(
    ///     html.write_to_string(false),
    ///     "<html><body><div></div><script>console.log()</script></body></html>"
    /// );
    /// ```
    ///
    /// # Note:
    ///
    /// JavaScript snippets are de-duplicated; Including the same snippet multiple times  will
    /// still result in a single `<script>` element
    pub fn js(mut self, js: &'static str) -> Node {
        if let Self::Element(ref mut el) = self {
            el.assets.push(asset::Asset::JavaScript(js));
        }
        self
    }

    /// Define a CSS variable for this Node
    ///
    /// This is useful for "parameterizing" styles. You can call this method multiple times to
    /// define additional variables.
    ///
    /// # Example
    /// ```
    /// use toph::{tag::*};
    ///
    /// let css = "div { color: var(--text-color); border: 1px solid var(--div-color); }";
    /// let mut html = html_.set([
    ///     head_,
    ///     body_.set([
    ///         div_.css(css)
    ///             .var("text-color", "white")
    ///             .var("div-color", "black"),
    ///
    ///         div_.css(css)
    ///             .var("text-color", "brown")
    ///             .var("div-color", "pink"),
    ///     ])
    /// ]);
    ///
    /// assert_eq!(
    ///     html.write_to_string(true),
    /// r#"<html>
    ///   <head>
    ///     <style>
    ///       div { color: var(--text-color); border: 1px solid var(--div-color); }
    ///     </style>
    ///   </head>
    ///   <body>
    ///     <div style="--text-color: white;--div-color: black;">
    ///     </div>
    ///     <div style="--text-color: brown;--div-color: pink;">
    ///     </div>
    ///   </body>
    /// </html>
    /// "#)
    /// ```
    ///
    /// # Notes:
    /// - Double dashes are automatically prepended to the name when displayed
    /// - The value is always attribute encoded
    pub fn var(mut self, name: &'static str, value: &str) -> Node {
        if let Self::Element(ref mut el) = self {
            el.attributes.push(Attribute::new_variable(name, value));
        }
        self
    }

    /// Sets this Element's children
    ///
    /// You can pass in anything that can be converted to a [`Node`](crate::Node):
    ///
    /// ```
    /// use toph::tag::*;
    ///
    /// // A single 'static string slice
    /// span_.set("hello");
    ///
    /// // An owned string
    /// span_.set(String::from("hello"));
    ///
    /// // These are equivalent
    /// span_;
    /// span_.set([]);
    ///
    /// // An array of nodes
    /// span_.set([div_, span_]);
    ///
    /// // 'static string slices and owned strings can be
    /// // converted to nodes, so this also works
    /// span_.set([
    ///     div_,
    ///     "bare string".into(),
    ///     span_,
    /// ]);
    /// ```
    pub fn set(mut self, child: impl Into<Node>) -> Node {
        if let Self::Element(ref mut el) = self {
            el.child = Some(Box::new(child.into()));
            if el.tag == "html" {
                visitor::include_assets(&mut self);
            }
        }
        self
    }
}

impl From<&'static str> for Node {
    fn from(value: &'static str) -> Self {
        Node::Text(Text(Cow::Borrowed(value)))
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        let encoded = encode::html(&value);
        Node::Text(Text(encoded.into()))
    }
}

macro_rules! impl_node_for_array_of_nodes {
    ($($n:expr),+) => {
        $(
            impl From<[Node; $n]> for Node {
                fn from(value: [Node; $n]) -> Self {
                    Node::Fragment(Fragment(value.to_vec()))
                }
            }
        )+
    };
}

impl From<Vec<Node>> for Node {
    fn from(value: Vec<Node>) -> Self {
        Self::Fragment(Fragment(value))
    }
}
impl From<[Node; 0]> for Node {
    fn from(_value: [Node; 0]) -> Self {
        Self::Text(Text("".into()))
    }
}

#[rustfmt::skip]
impl_node_for_array_of_nodes!(
    1, 2, 3, 4, 5, 6, 7, 8, 9,
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    20
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr;
    use crate::node::tag::*;

    #[track_caller]
    fn assert_html(node: impl Into<Node>, expected: &str) {
        assert_eq!((&mut node.into()).write_to_string(false), expected);
    }

    #[test]
    fn testing() {
        let huh = attr![];
        println!("{:?}", huh);
    }

    #[test]
    fn html_fragments() {
        // including strings
        assert_html(
            [span_.set("literal"), span_.set(String::from("string"))],
            "<span>literal</span><span>string</span>",
        );

        // nesting nodes
        assert_html(
            [div_.set([span_, div_.set(div_)])],
            "<div><span></span><div><div></div></div></div>",
        );

        // literal attribute values can be used with unsafe sinks
        assert_html(
            span_.with(attr![onclick = "something"]),
            r#"<span onclick="something"></span>"#,
        );
        // non-literal attribute values cannot be used with unsafe sinks
        assert_html(span_.with(attr![onclick = String::new()]), "<span></span>");

        // literal urls can include any scheme
        assert_html(
            span_.with(attr![src = "javascript:boom"]),
            r#"<span src="javascript:boom"></span>"#,
        );

        // non-literal urls may only use safe schemes
        assert_html(
            span_.with(attr![src = String::from("javascript:")]),
            "<span></span>",
        );
        assert_html(
            span_.with(attr![src = String::from("mailto:a.com")]),
            r#"<span src="mailto:a.com"></span>"#,
        );

        // boolean attributes are supported
        assert_html(span_.with(attr![async]), "<span async></span>");

        // mix of regular & boolean attributes
        assert_html(
            span_.with(attr![async, class = "hidden", checked]),
            r#"<span async class="hidden" checked></span>"#,
        );
        assert_html(
            span_.with(attr![class = "hidden", async, id = "id"]),
            r#"<span class="hidden" async id="id"></span>"#,
        );

        // optional comma at the end of attribute list
        assert_html(span_.with(attr![async,]), "<span async></span>");
        assert_html(
            span_.with(attr![class = "class",]),
            r#"<span class="class"></span>"#,
        );

        // data-* attributes are supported
        assert_html(
            span_.with(attr![data_custom = "hello"]),
            r#"<span data-custom="hello"></span>"#,
        );
    }

    #[test]
    fn including_assets() {
        // css is prepended to the head element
        assert_html(
            [html_.set([head_.set(title_), body_.css("some css")])],
            r#"<html><head><style>some css</style><title></title></head><body></body></html>"#,
        );
        // css is added if when head element is empty
        assert_html(
            [html_.set([head_, body_.css("some css")])],
            r#"<html><head><style>some css</style></head><body></body></html>"#,
        );
        // no css is included when head is absent
        assert_html(
            [html_.set(body_.css("some css"))],
            "<html><body></body></html>",
        );
        // no css is included when html is absent
        assert_html([body_.css("some css")], "<body></body>");

        // js is appended to the body element
        assert_html(
            [html_.set(body_.js("some js").set(span_))],
            "<html><body><span></span><script>some js</script></body></html>",
        );

        // js is added when body element is empty
        assert_html(
            [html_.set(body_.js("some js"))],
            "<html><body><script>some js</script></body></html>",
        );

        // no js is added when body is absent
        assert_html(
            [html_.set(span_.js("some js"))],
            "<html><span></span></html>",
        );

        // no js is added when html is absent
        assert_html([body_.js("some js")], "<body></body>");
    }
}
