use super::{attribute::Attribute, tag::*, Element, Node, Text};
use std::fmt;
use std::io;
use std::mem;

enum Tag<'n> {
    Open(Option<&'n mut Node>),
    Close(&'static str),
}

// Extracts all css & javascript assets from the subtrees and places them in <style> & <script>
// nodes
pub fn include_assets(node: &mut Node) {
    // Get assets
    let mut collector = AssetCollector::new();
    visit_nodes(node, &mut collector).expect("collecting assets does not fail");

    let mut style = None;
    let mut script = None;

    let script_fragments = collector
        .js
        .into_iter()
        .map(|j| script_.set(j))
        .collect::<Vec<_>>();
    let style_fragments = collector
        .css
        .into_iter()
        .map(|c| style_.set(c))
        .collect::<Vec<_>>();

    if script_fragments.len() > 0 {
        script = Some(script_fragments.into());
    }

    if style_fragments.len() > 0 {
        style = Some(style_fragments.into());
    }

    // Insert them into the tree
    let inserter = AssetInserter::new(style, script);
    visit_nodes(node, inserter).expect("inserting nodes does not fail");
}

// The visitor pattern[1] is used for traversing a Node tree.
// [1]: https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html
pub trait NodeVisitor {
    type Error;
    fn visit_open_tag(&mut self, _el: &mut Element) -> Result<(), Self::Error> {
        Ok(())
    }
    fn visit_close_tag(&mut self, _tag: &'static str) -> Result<(), Self::Error> {
        Ok(())
    }
    fn visit_text(&mut self, _text: &str) -> Result<(), Self::Error> {
        Ok(())
    }
    fn finish(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// Core traversal code:
// Visits the nodes in the tree in the order they would appear in html
//
// Element nodes nodes are visited twice; for the start & end tags.
// Text nodes are visited once
// Fragment nodes are skipped, but the nodes they contain are visited
pub fn visit_nodes<V: NodeVisitor>(
    start: &mut Node,
    mut visitor: V,
) -> Result<(), <V as NodeVisitor>::Error> {
    let mut visit_later: Vec<Tag> = vec![];
    visit_later.push(Tag::Open(Some(start)));

    while let Some(t) = visit_later.pop() {
        match t {
            Tag::Open(Some(Node::Element(el))) => {
                visitor.visit_open_tag(el)?;

                if el.is_void() {
                    continue;
                }

                // re-visit this node after its children have been visited
                visit_later.push(Tag::Close(el.tag));

                visit_later.push(Tag::Open(el.child.as_deref_mut()));
            }
            Tag::Close(tag_name) => {
                visitor.visit_close_tag(tag_name)?;
            }
            Tag::Open(Some(Node::Fragment(f))) => {
                for child in f.0.iter_mut().rev() {
                    visit_later.push(Tag::Open(Some(child)));
                }
            }
            Tag::Open(Some(Node::Text(Text(ref t)))) => {
                visitor.visit_text(t)?;
            }
            _ => {}
        }
    }

    visitor.finish()?;

    Ok(())
}

// A visitor that transforms a Node tree to an html string
pub struct HtmlStringWriter<W> {
    html: W,
}

impl<W: fmt::Write> HtmlStringWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { html: inner }
    }
}

impl<W: fmt::Write> NodeVisitor for HtmlStringWriter<W> {
    type Error = fmt::Error;

    fn visit_open_tag(&mut self, el: &mut Element) -> Result<(), Self::Error> {
        write!(self.html, "<{}", el.tag)?;
        if el.attributes.len() > 0 {
            for attr in &el.attributes {
                write!(self.html, "{}", attr)?;
            }
        }
        write!(self.html, ">")?;
        Ok(())
    }

    fn visit_close_tag(&mut self, tag: &'static str) -> Result<(), Self::Error> {
        write!(self.html, "</{}>", tag)?;
        Ok(())
    }

    fn visit_text(&mut self, text: &str) -> Result<(), Self::Error> {
        write!(self.html, "{}", text)?;
        Ok(())
    }
}

// A visitor that transforms a Node tree to an html byte stream.
pub struct HtmlWriter<W> {
    html: W,
}
impl<W: io::Write> HtmlWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { html: inner }
    }
}

impl<W: io::Write> NodeVisitor for HtmlWriter<W> {
    type Error = io::Error;

    fn visit_open_tag(&mut self, el: &mut Element) -> Result<(), Self::Error> {
        write!(self.html, "<{}", el.tag)?;
        if el.attributes.len() > 0 {
            for attr in &el.attributes {
                write!(self.html, "{}", attr)?;
            }
        }
        write!(self.html, ">")?;
        Ok(())
    }

    fn visit_close_tag(&mut self, tag: &'static str) -> Result<(), Self::Error> {
        write!(self.html, "</{}>", tag)?;
        Ok(())
    }

    fn visit_text(&mut self, text: &str) -> Result<(), Self::Error> {
        write!(self.html, "{}", text)?;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        self.html.flush()?;
        Ok(())
    }
}

// A visitor that inserts style & script nodes into a node tree
pub struct AssetInserter {
    style: Option<Node>,
    script: Option<Node>,
}

impl AssetInserter {
    pub fn new(style: Option<Node>, script: Option<Node>) -> Self {
        Self { style, script }
    }
}

impl<'s> NodeVisitor for AssetInserter {
    type Error = ();

    fn visit_open_tag(&mut self, el: &mut Element) -> Result<(), Self::Error> {
        if el.tag == "head" {
            if let Some(node) = self.style.take() {
                if let Some(mut old) = el.child.take() {
                    *old = [node, mem::take(&mut old)].into();
                    el.child = Some(old);
                }
            }
        } else if el.tag == "body" {
            if let Some(node) = self.script.take() {
                if let Some(mut old) = el.child.take() {
                    *old = [mem::take::<Node>(&mut old), node].into();
                    el.child = Some(old);
                }
            }
        }

        Ok(())
    }
}

// A visitor that collects all css & js declarations from the Node tree
pub struct AssetCollector {
    pub css: Vec<&'static str>,
    pub js: Vec<&'static str>,
}

impl AssetCollector {
    pub fn new() -> Self {
        Self {
            css: Vec::default(),
            js: Vec::default(),
        }
    }
}

impl NodeVisitor for &mut AssetCollector {
    type Error = ();

    fn visit_open_tag(&mut self, el: &mut Element) -> Result<(), Self::Error> {
        for attr in &el.attributes {
            match attr {
                Attribute::Js(js) => {
                    self.js.push(js);
                }
                Attribute::Css(css) => {
                    self.css.push(css);
                }
                _ => {}
            }
        }
        Ok(())
    }
}
