pub mod attribute;
pub mod tag;
pub mod visitor;

use crate::encode;
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
                | "!DOCTYPE"
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
    pub fn write_to_string(&mut self) -> String {
        let mut buf = String::new();
        let writer = visitor::HtmlStringWriter::new(&mut buf);
        visitor::visit_nodes(self, writer).expect("printing to a string should not fail");
        buf
    }

    /// Writes the HTML for the tree rooted at this node to anything that implements [`io::Write`]
    pub fn write<W: io::Write>(&mut self, w: W) -> Result<(), io::Error> {
        let writer = visitor::HtmlWriter::new(w);
        visitor::visit_nodes(self, writer)
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
    use crate::node::tag::*;
    use crate::__;

    #[track_caller]
    fn assert_html(node: impl Into<Node>, expected: &str) {
        assert_eq!((&mut node.into()).write_to_string(), expected);
    }

    #[test]
    fn testing() {
        let huh = __![];
        println!("{:?}", huh);
    }

    #[test]
    fn html_fragments() {
        // including strings
        assert_html(
            [span_("literal"), span_(String::from("string"))],
            "<span>literal</span><span>string</span>",
        );

        // nesting nodes
        assert_html(
            [div_(span_([])), div_([span_([])]), div_([])],
            "<div><span></span></div><div><span></span></div><div></div>",
        );

        // literal attribute values can be used with unsafe sinks
        assert_html(
            span_(__![onclick = "something"]),
            r#"<span onclick="something"></span>"#,
        );
        // non-literal attribute values cannot be used with unsafe sinks
        assert_html(span_(__![onclick = String::new()]), "<span></span>");

        // literal urls can include any scheme
        assert_html(
            span_(__![src = "javascript:boom"]),
            r#"<span src="javascript:boom"></span>"#,
        );

        // non-literal urls may only use safe schemes
        assert_html(
            span_(__![src = String::from("javascript:")]),
            "<span></span>",
        );
        assert_html(
            span_(__![src = String::from("mailto:a.com")]),
            r#"<span src="mailto:a.com"></span>"#,
        );

        // boolean attributes are supported
        assert_html(span_(__![async]), "<span async></span>");

        // mix of regular & boolean attributes
        assert_html(
            span_(__![async, class = "hidden", checked]),
            r#"<span async class="hidden" checked></span>"#,
        );
        assert_html(
            span_(__![class = "hidden", async, id = "id"]),
            r#"<span class="hidden" async id="id"></span>"#,
        );

        // optional comma at the end of attribute list
        assert_html(span_(__![async,]), "<span async></span>");
        assert_html(
            span_(__![class = "class",]),
            r#"<span class="class"></span>"#,
        );

        // data-* attributes are supported
        assert_html(
            span_(__![data_custom = "hello"]),
            r#"<span data-custom="hello"></span>"#,
        );
    }

    #[test]
    fn including_assets() {
        // css is included if there is a head element
        assert_html(
            [html_([head_(title_([])), body_(__![@css="some css"])])],
            r#"<html><head><style>some css</style><title></title></head><body></body></html>"#,
        );
        assert_html(
            [html_(body_(__![@css="some css"]))],
            "<html><body></body></html>",
        );

        // js is included if there is a body element
        assert_html(
            [html_(body_((__![@js="some js"], span_([]))))],
            "<html><body><span></span><script>some js</script></body></html>",
        );
        assert_html(
            [html_(span_(__![@js="some js"]))],
            "<html><span></span></html>",
        );
    }
}
