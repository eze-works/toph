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
    pub fn new_element(tag: &'static str, attributes: Vec<Attribute>) -> Node {
        Node::Element(Element {
            tag,
            attributes,
            children: vec![],
        })
    }

    pub fn new_fragment() -> Node {
        Node::Fragment(Fragment(vec![]))
    }

    pub fn new_text(text: String) -> Node {
        Node::Text(Text(text))
    }

    pub fn append_child(&mut self, child: Node) {
        match self {
            Node::Fragment(Fragment(nodes)) => nodes.push(child),
            Node::Element(Element { children, .. }) => children.push(child),
            Node::Text(_) | Node::RawText(_) => panic!("cannot add child to text node"),
        }
    }
}

impl<T: std::fmt::Display> From<T> for Node {
    fn from(value: T) -> Self {
        Node::new_text(value.to_string())
    }
}
