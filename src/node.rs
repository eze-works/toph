pub mod attribute;
pub mod tag;
mod variable;
pub mod visitor;

use crate::encode;
use attribute::AttributeMap;
use variable::CSSVariableMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Asset {
    StyleSheet(&'static str),
    JavaScript(&'static str),
}

/// An HTML Node. All [tag functions](crate::tag) are instances of this type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    // When `tag` is not empty, this is an element node
    tag: &'static str,
    // When `tag` is empty, this isa a text node
    text: String,
    attributes: AttributeMap,
    variables: CSSVariableMap,
    children: Vec<Node>,
    assets: Vec<Asset>,
}

impl Node {
    const fn element(tag: &'static str) -> Self {
        Node {
            tag,
            text: String::new(),
            attributes: AttributeMap::new(),
            variables: CSSVariableMap::new(),
            children: vec![],
            assets: vec![],
        }
    }

    // NOTE: For consistency, the API should NEVER return a Node with no tag.
    // Text nodes can't have attributes, variables, or children when printed out
    const fn text(text: String) -> Self {
        Node {
            tag: "",
            text,
            attributes: AttributeMap::new(),
            variables: CSSVariableMap::new(),
            children: vec![],
            assets: vec![],
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

    /// Links an inline css stylesheet to the Node
    ///
    /// The stylesheet will be included verbatim in a `<style>` element when this Node is in a tree
    /// with both `<html>` & `<head>` tags
    ///
    /// # Example
    ///
    /// ```
    /// use toph::{tag::*, Node};
    ///
    /// let html = div_.stylesheet("div { border: 1px solid black; }");
    /// assert_eq!(Node::render([html]), "<div></div>");
    ///
    /// let html = html_.set([
    ///     head_,
    ///     div_.stylesheet("div { border: 1px solid black; }")
    /// ]);
    /// assert_eq!(
    ///     Node::render([html]),
    ///     "<html><head><style>div { border: 1px solid black; }</style></head><div></div></html>"
    /// );
    /// ```
    /// # Note
    ///
    /// CSS snippets are de-duplicated; Including the same snippet multiple times  will
    /// still result in a single `<style>` element
    pub fn stylesheet(mut self, css: &'static str) -> Self {
        self.assets.push(Asset::StyleSheet(css));
        self
    }

    /// Links a JavaScript snippet to the Node
    ///
    /// The javascript snippet will be included verbatim as a `<script>` element when this Node is
    /// in a tree with both `<html>` & `<body>` tags
    ///
    /// # Example
    ///
    /// ```
    /// use toph::{tag::*, Node};
    ///
    /// let html = div_.js("console.log()");
    /// assert_eq!(Node::render([html]), "<div></div>");
    ///
    /// let html = html_.set([
    ///     body_.set([div_.js("console.log()")])
    /// ]);
    ///
    /// assert_eq!(
    ///     Node::render([html]),
    ///     "<html><body><div></div><script>console.log()</script></body></html>"
    /// );
    /// ```
    ///
    /// # Note:
    ///
    /// JavaScript snippets are de-duplicated; Including the same snippet multiple times  will
    /// still result in a single `<script>` element
    pub fn js(mut self, js: &'static str) -> Self {
        self.assets.push(Asset::JavaScript(js));
        self
    }

    /// Define a CSS variable for this Node
    ///
    /// This is useful for "parameterizing" styles. You can call this method multiple times to
    /// define additional variables.
    ///
    /// If you are writing a component with declarations affecting descendant rules like ...
    ///
    /// ```text
    /// your-component > * {
    ///     blah-blah: var(--your-variable);
    /// }
    /// ```
    ///
    /// ... then you _probably_ intended to call this method on the child nodes.
    ///
    /// Because CSS ... uhh, cascades, calling this method on the parent instead of the child nodes
    /// can cause odd interactions to happen when you nest one `your-component` inside another.
    ///
    /// # Example
    ///
    /// ```
    /// use toph::{tag::*, Node};
    ///
    /// let css = "div { color: var(--text-color); border: 1px solid var(--div-color); }";
    /// let html = html_.set([
    ///     head_,
    ///     body_.set([
    ///         div_.stylesheet(css)
    ///             .var("text-color", "white")
    ///             .var("div-color", "black"),
    ///
    ///         div_.stylesheet(css)
    ///             .var("text-color", "brown")
    ///             .var("div-color", "pink"),
    ///     ])
    /// ]);
    ///
    /// assert_eq!(
    ///     Node::render_pretty([html]),
    /// r#"<html>
    ///   <head>
    ///     <style>
    ///       div { color: var(--text-color); border: 1px solid var(--div-color); }
    ///     </style>
    ///   </head>
    ///   <body>
    ///     <div style="--div-color: black;--text-color: white;">
    ///     </div>
    ///     <div style="--div-color: pink;--text-color: brown;">
    ///     </div>
    ///   </body>
    /// </html>
    /// "#)
    /// ```
    ///
    /// # Notes:
    /// - Double dashes are automatically prepended to the name when displayed
    /// - The value is always attribute encoded
    pub fn var(mut self, name: &'static str, value: &str) -> Self {
        self.variables.insert(name, value);
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

    /// Append `html` verbatim as a child of this element. This skip html encoding.
    ///
    /// ```
    /// use toph::{Node, tag::*};
    /// let html = span_.dangerously_set_html("<script>alert(1)</script>");
    ///
    /// assert_eq!(
    ///     Node::render([html]),
    ///     "<span><script>alert(1)</script></span>"
    /// );
    /// ```
    pub fn dangerously_set_html(mut self, html: &str) -> Self {
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
            if node.tag == "html" {
                visitor::include_assets(&mut node);
            }

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

    #[test]
    fn including_assets() {
        // css is appended to the head element
        assert_html(
            [html_.set([head_.set([title_]), body_.stylesheet("some css")])],
            r#"<html><head><title></title><style>some css</style></head><body></body></html>"#,
        );
        // css is added if when head element is empty
        assert_html(
            [html_.set([head_, body_.stylesheet("some css")])],
            r#"<html><head><style>some css</style></head><body></body></html>"#,
        );
        // no css is included when head is absent
        assert_html(
            [html_.set([body_.stylesheet("some css")])],
            "<html><body></body></html>",
        );
        // no css is included when html is absent
        assert_html([body_.stylesheet("some css")], "<body></body>");

        // css is deduplicated
        assert_html(
            [html_.stylesheet("a").set([head_, body_.stylesheet("a")])],
            "<html><head><style>a</style></head><body></body></html>",
        );

        // js is appended to the body element
        assert_html(
            [html_.set([body_.js("some js").set([span_])])],
            "<html><body><span></span><script>some js</script></body></html>",
        );

        // js is added when body element is empty
        assert_html(
            [html_.set([body_.js("some js")])],
            "<html><body><script>some js</script></body></html>",
        );

        // no js is added when body is absent
        assert_html(
            [html_.set([span_.js("some js")])],
            "<html><span></span></html>",
        );

        // no js is added when html is absent
        assert_html([body_.js("some js")], "<body></body>");

        // js is deduplicated
        assert_html(
            [html_.js("js").set([body_.js("js")])],
            "<html><body><script>js</script></body></html>",
        );

        // order in which assets are appended does not matter
        assert_html(
            [html_.set([head_, body_]).js("js").stylesheet("css")],
            "<html><head><style>css</style></head><body><script>js</script></body></html>",
        );
    }
}
