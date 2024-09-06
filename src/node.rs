use crate::Attribute;

/// See [`Node`]
#[derive(Debug, Clone)]
pub struct Element {
    tag: &'static str,
    attributes: Vec<Attribute>,
    children: Vec<Node>,
}

/// See [`Node`]
#[derive(Debug, Clone)]
pub struct Text(String);

/// See [`Node`]
#[derive(Debug, Clone)]
pub struct Fragment(Vec<Node>);

#[derive(Debug, Clone)]
pub enum Node {
    /// An HTML element like `<p>` or `<div>`
    Element(Element),
    /// The actual text inside of an HTML element. The text will be HTML-escaped upon serialization
    Text(Text),
    /// Similar to the [`Node::Text`] variant, except the text is emitted as-is, without escaping.
    RawText(Text),
    /// A list of HTML nodes.
    Fragment(Fragment),
}

impl Node {
    pub fn element(tag: &'static str, attributes: Vec<Attribute>) -> Node {
        Node::Element(Element {
            tag,
            attributes,
            children: vec![],
        })
    }

    pub fn fragment() -> Node {
        Node::Fragment(Fragment(vec![]))
    }

    pub fn text(text: impl AsRef<str>) -> Node {
        Node::Text(Text(text.as_ref().to_string()))
    }

    pub fn append_child(&mut self, child: Node) {
        match self {
            Node::Fragment(Fragment(nodes)) => nodes.push(child),
            Node::Element(Element { children, .. }) => children.push(child),
            Node::Text(_) | Node::RawText(_) => panic!("cannot add child to text node"),
        }
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Node::Text(Text(value.to_string()))
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        Node::Text(Text(value))
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
