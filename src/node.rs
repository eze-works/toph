use crate::encode;
use crate::Attribute;
use std::fmt::Display;

/// See [`Node`]
#[derive(Debug, Clone)]
pub struct Element {
    tag: String,
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

/// Returns a text [`Node`] whose contents are HTML escaped
///
/// See the [`html`](crate::html!) macro for more details
pub fn text(text: impl Display) -> Node {
    Node::Text(Text(text.to_string()))
}

/// Returns a text [`Node`] whose contents are not HTML escaped
///
/// See the [`html`](crate::html!) macro for more details
pub fn raw_text(text: impl Display) -> Node {
    Node::RawText(Text(text.to_string()))
}

enum Tag<'n> {
    Open(&'n Node),
    Close(&'n Element),
}

impl Node {
    #[doc(hidden)]
    pub fn element(tag: String, attributes: Vec<Attribute>) -> Node {
        let tag = tag.to_ascii_lowercase();
        let tag = if tag == "doctype" {
            String::from("!doctype")
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
}

impl Display for Node {
    /// Converts the Node to an HTML string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Serialization is done by traversing the tree in a depth-first manner.
        // Open tags are serialized on the way down, closing tags are serialized on the way up
        let mut visit_later = vec![Tag::Open(self)];

        while let Some(t) = visit_later.pop() {
            match t {
                Tag::Open(Node::Text(Text(s))) => {
                    write!(f, "{}", encode::html(s))?;
                }
                Tag::Open(Node::RawText(Text(s))) => {
                    write!(f, "{s}")?;
                }
                Tag::Open(Node::Element(el)) => {
                    let attributes = el
                        .attributes
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join("");

                    write!(f, "<{}{}>", el.tag.replace('_', "-"), attributes)?;

                    if el.is_void() {
                        continue;
                    }

                    // re-visit this node after its children have been visited
                    visit_later.push(Tag::Close(el));

                    for child in el.children.iter().rev() {
                        visit_later.push(Tag::Open(child));
                    }
                }
                Tag::Open(Node::Fragment(fragment)) => {
                    for child in fragment.0.iter().rev() {
                        visit_later.push(Tag::Open(child));
                    }
                }
                Tag::Close(el) => {
                    write!(f, "</{}>", el.tag.replace('_', "-"))?;
                }
            }
        }

        Ok(())
    }
}

impl From<Node> for String {
    fn from(value: Node) -> Self {
        value.to_string()
    }
}

impl<T> From<T> for Node
where
    T: IntoIterator<Item = Node>,
{
    fn from(value: T) -> Self {
        Node::Fragment(Fragment(value.into_iter().collect::<Vec<Node>>()))
    }
}
