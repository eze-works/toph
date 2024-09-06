use crate::encode;
use crate::Attribute;
use std::fmt::Write;

/// See [`Node`]
#[derive(Debug, Clone)]
pub struct Element {
    tag: &'static str,
    attributes: Vec<Attribute>,
    children: Vec<Node>,
}

impl Element {
    fn is_void(&self) -> bool {
        [
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
            "track", "wbr", "!doctype",
        ]
        .contains(&self.tag.to_lowercase().as_str())
    }
}

/// See [`Node`]
#[derive(Debug, Clone)]
pub struct Text(String);

/// See [`Node`]
#[derive(Debug, Clone)]
pub struct Fragment(Vec<Node>);

/// A node in an HTML tree structure
///
/// The [`html`](crate::html!) macro creates instances of this type
#[derive(Debug, Clone)]
pub enum Node {
    /// An HTML element like `<p>` or `<div>`
    Element(Element),
    /// Text within an HTML element. This is automatically html-escaped
    Text(Text),
    /// Similar to the `Text` variant, except it is included in the final HTML as-is, without
    /// escaping.
    RawText(Text),
    /// A list of HTML nodes.
    Fragment(Fragment),
}

/// Returns a text [`Node`] whose contents are not HTML escaped
///
/// See the [`html`](crate::html!) macro for more details
pub fn raw_text(text: impl AsRef<str>) -> Node {
    Node::RawText(Text(text.as_ref().to_string()))
}

enum Tag<'n> {
    Open(&'n Node),
    Close(&'static str),
}

impl Node {
    #[doc(hidden)]
    pub fn element(tag: &'static str, attributes: Vec<Attribute>) -> Node {
        let tag = if tag.to_lowercase() == "doctype" {
            "!doctype"
        } else {
            tag
        };

        Node::Element(Element {
            tag,
            attributes,
            children: vec![],
        })
    }

    #[doc(hidden)]
    pub fn fragment() -> Node {
        Node::Fragment(Fragment(vec![]))
    }

    #[doc(hidden)]
    pub fn append_child(&mut self, child: Node) {
        match self {
            Node::Fragment(Fragment(nodes)) => nodes.push(child),
            Node::Element(Element { children, .. }) => children.push(child),
            Node::Text(_) | Node::RawText(_) => panic!("cannot add child to text node"),
        }
    }

    /// Converts the Node to an HTML string
    pub fn serialize(&self) -> String {
        let mut buffer = String::new();
        let mut visit_later = vec![Tag::Open(self)];

        while let Some(t) = visit_later.pop() {
            match t {
                Tag::Open(node) => match node {
                    Node::Text(Text(s)) => {
                        write!(buffer, "{}", encode::html(s)).unwrap();
                    }
                    Node::RawText(Text(s)) => {
                        write!(buffer, "{s}").unwrap();
                    }
                    Node::Element(
                        el @ Element {
                            tag,
                            attributes,
                            children,
                        },
                    ) => {
                        let attributes = attributes
                            .iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<_>>()
                            .join("");

                        write!(buffer, "<{}{}>", tag.replace('_', "-"), attributes).unwrap();

                        if el.is_void() {
                            continue;
                        }

                        // re-visit this node after its children have been visited
                        visit_later.push(Tag::Close(el.tag));

                        for child in children.iter().rev() {
                            visit_later.push(Tag::Open(child));
                        }
                    }
                    Node::Fragment(Fragment(nodes)) => {
                        for child in nodes.iter().rev() {
                            visit_later.push(Tag::Open(child));
                        }
                    }
                },
                Tag::Close(tag) => {
                    write!(buffer, "</{}>", tag.replace('_', "-")).unwrap();
                }
            }
        }

        buffer
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Node::Text(Text(value.to_string()))
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        Node::from(value.as_str())
    }
}

impl From<Vec<Node>> for Node {
    fn from(value: Vec<Node>) -> Self {
        Node::Fragment(Fragment(value))
    }
}

macro_rules! impl_from_array {
    ($($count:literal)*) => {
        $(
            impl From<[Node; $count]> for Node {
                fn from(value: [Node; $count]) -> Self {
                    Node::Fragment(Fragment(value.to_vec()))
                }
            }
        )*
    }
}

impl_from_array! { 0 1 2 3 4 5 6 7 8 9 10 11 12 }
