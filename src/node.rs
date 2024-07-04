pub mod attribute;
pub mod tag;
pub mod visitor;

use crate::encode;
use attribute::AttributeMap;

/// An HTML Node. All [tag functions](crate::tag) are instances of this type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    // When `tag` is not empty, this is an element node
    tag: &'static str,
    // When `tag` is empty, this is a text node
    text: String,
    attributes: AttributeMap,
    children: Vec<Node>,
}

impl Node {
    const fn element(tag: &'static str) -> Self {
        Node {
            tag,
            text: String::new(),
            attributes: AttributeMap::new(),
            children: vec![],
        }
    }

    // NOTE: For consistency, the API should avoid returning a Node with no tag.
    // Text nodes can't have attributes, variables, or children when printed out
    const fn text(text: String) -> Self {
        Node {
            tag: "",
            text,
            attributes: AttributeMap::new(),
            children: vec![],
        }
    }

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

    /// Sets HTML attributes
    ///
    /// A mix of boolean & regular attributes can be set. You can call this method multiple times
    ///
    /// ```
    /// use toph::{attr, tag::*, Node};
    /// let mut s = span_
    ///     .with(attr![class="card", hidden])
    ///     .with(attr![id="hello"]);
    ///
    /// assert_eq!(
    ///     Node::render([s]),
    ///     r#"<span class="card" id="hello" hidden></span>"#
    /// );
    /// ```
    ///
    ///
    /// ## Duplicates
    ///
    /// Generally, if an attribute appears twice, the last occurence wins
    ///
    /// ```
    /// use toph::{attr, tag::*, Node};
    /// assert_eq!(
    ///     Node::render([span_.with(attr![id="one", id="two"])]),
    ///     r#"<span id="two"></span>"#
    /// );
    /// ```
    ///
    /// For space-separated attributes (e..g `class`), occurences are combined with a space;
    ///
    /// ```
    /// use toph::{attr, tag::*, Node};
    /// assert_eq!(
    ///     Node::render([span_.with(attr![class="one", class="two"])]),
    ///     r#"<span class="one two"></span>"#
    /// );
    /// ```
    ///
    /// For comma-separated attributes (e.g. `accept`), occurences are combined with a comma;
    ///
    /// ```
    /// use toph::{attr, tag::*, Node};
    ///
    /// assert_eq!(
    ///     Node::render([span_.with(attr![accept="audio/*", accept="video/*"])]),
    ///     r#"<span accept="audio/*,video/*"></span>"#
    /// );
    /// ```
    /// See the [attr](crate::attr) macro docs for details.
    pub fn with<I>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = (&'static str, String, bool)>,
    {
        for attr in attributes {
            self.attributes.insert(attr.0, &attr.1, attr.2)
        }
        self
    }

    /// Sets this Element's children
    ///
    /// You can pass anything that can be converted into an iterator of `Nodes`
    ///
    /// This includes things such as `Option<Node>`, `Result<Node>` and arrays & `Vec`s of `Node`s
    ///
    /// # Examples
    ///
    /// ```
    /// use toph::tag::*;
    ///
    /// // Strings can be converted to nodes. So an array of strings works
    /// span_.set(["hello", "world"]);
    ///
    /// // An array of nodes
    /// span_.set([div_, span_]);
    ///
    /// // if you want to mix them, you have to explicitely use `.into()` on the string
    /// span_.set([div_, "hey".into(), div_]);
    ///
    /// // A node wrapped in an  option or result
    /// span_.set(Some(div_));
    /// ```
    ///
    /// Calling this multiple times appends children
    pub fn set<I, E>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<Node>,
    {
        let mut children = children.into_iter().map(|c| c.into()).collect::<Vec<_>>();
        self.children.append(&mut children);
        self
    }

    /// Append `html` verbatim as a child of this element. This skips html encoding.
    ///
    /// ```
    /// use toph::{Node, tag::*};
    /// let html = span_.set_unescaped("<script>alert(1)</script>");
    ///
    /// assert_eq!(
    ///     Node::render([html]),
    ///     "<span><script>alert(1)</script></span>"
    /// );
    /// ```
    pub fn set_unescaped(mut self, html: impl Into<String>) -> Self {
        let child_text_element = Node::text(html.into());
        self.children.push(child_text_element);
        self
    }
}

impl Node {
    /// Converts the list of nodes to an HTML string
    ///
    /// If one of the nodes is an `<html>` element, css & javascript will be extracted from its
    /// children and appended to the `<head>` & `<body>` elements respectively , if they
    /// exist
    pub fn render<I, N>(nodes: I) -> String
    where
        I: IntoIterator<Item = N>,
        N: Into<Node>,
    {
        Node::render_to_string(nodes, false)
    }

    /// Converts the list of nodes to an HTML string with indentation
    ///
    /// See [`render`](`crate::Node::render`)
    pub fn render_pretty<I, N>(nodes: I) -> String
    where
        I: IntoIterator<Item = N>,
        N: Into<Node>,
    {
        Node::render_to_string(nodes, true)
    }

    fn render_to_string<I, N>(nodes: I, indent: bool) -> String
    where
        I: IntoIterator<Item = N>,
        N: Into<Node>,
    {
        let mut buf = String::new();
        for node in nodes {
            let mut node = node.into();
            let writer = visitor::HtmlStringWriter::new(&mut buf, indent);
            visitor::visit_nodes(&mut node, writer).expect("printing to a string should not fail");
        }
        buf
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Node::text(encode::html(value))
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        Node::from(value.as_str())
    }
}

impl<I: Into<Node>, E> From<Result<I, E>> for Node {
    fn from(value: Result<I, E>) -> Self {
        value
            .map(|e| e.into())
            .unwrap_or_else(|_| Node::text(String::new()))
    }
}

impl<I: Into<Node>> From<Option<I>> for Node {
    fn from(value: Option<I>) -> Self {
        value
            .map(|e| e.into())
            .unwrap_or_else(|| Node::text(String::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr;
    use crate::node::tag::*;

    #[track_caller]
    fn assert_html<I, N>(nodes: I, expected: &str)
    where
        I: IntoIterator<Item = N>,
        N: Into<Node>,
    {
        let actual = Node::render(nodes);
        assert_eq!(actual, expected);
    }

    #[test]
    fn html_fragments() {
        // including strings
        assert_html(
            [span_.set(["literal"]), span_.set([String::from("string")])],
            "<span>literal</span><span>string</span>",
        );

        // strings are html encoded
        assert_html([span_.set(["<script>"])], "<span>&lt;script&gt;</span>");

        // nesting nodes
        assert_html(
            [div_.set([span_, div_.set([div_])])],
            "<div><span></span><div><div></div></div></div>",
        );

        // regular attributes
        assert_html(
            [span_.with(attr![onclick = "something"])],
            r#"<span onclick="something"></span>"#,
        );
        assert_html(
            [span_.with(attr![onclick = String::from("something")])],
            r#"<span onclick="something"></span>"#,
        );

        // boolean attributes are supported
        assert_html([span_.with(attr![async])], "<span async></span>");

        // mix of regular & boolean attributes
        assert_html(
            [span_.with(attr![async, class = "hidden", checked])],
            r#"<span class="hidden" async checked></span>"#,
        );
        assert_html(
            [span_.with(attr![class = "hidden", async, id = "id"])],
            r#"<span class="hidden" id="id" async></span>"#,
        );

        // optional comma at the end of attribute list
        assert_html([span_.with(attr![async,])], "<span async></span>");
        assert_html(
            [span_.with(attr![class = "class",])],
            r#"<span class="class"></span>"#,
        );

        // data-* attributes are supported
        assert_html(
            [span_.with(attr![data_custom = "hello"])],
            r#"<span data-custom="hello"></span>"#,
        );
    }
}
