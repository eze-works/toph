use super::{tag::*, Asset, Node};
use std::borrow::Cow;
use std::collections::btree_map::Entry;
use std::collections::BTreeSet;
use std::fmt;

enum Tag<'n> {
    Open(&'n mut Node),
    Close(&'static str),
}

// Extracts all css & javascript assets from the subtrees and places them in <style> & <script>
// nodes
pub fn include_assets(node: &mut Node) {
    // Get assets
    let mut collector = SnippetCollector::new();
    visit_nodes(node, &mut collector).expect("collecting assets does not fail");

    let script_fragments = collector
        .js
        .into_iter()
        .map(|j| script_.dangerously_set_html(j))
        .collect::<Vec<_>>();
    let style_fragments = collector
        .css
        .into_iter()
        .map(|c| style_.dangerously_set_html(&c))
        .collect::<Vec<_>>();

    // Insert them into the tree
    let inserter = AssetInserter::new(style_fragments, script_fragments);
    visit_nodes(node, inserter).expect("inserting nodes does not fail");
}

// The visitor pattern[1] is used for traversing a Node tree.
// [1]: https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html
pub trait NodeVisitor {
    type Error;
    fn visit_open_tag(&mut self, _el: &mut Node) -> Result<(), Self::Error> {
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
    visit_later.push(Tag::Open(start));

    while let Some(t) = visit_later.pop() {
        match t {
            Tag::Open(el) => {
                if el.tag.is_empty() {
                    visitor.visit_text(&el.text)?;
                    continue;
                }

                visitor.visit_open_tag(el)?;

                if el.is_void() {
                    continue;
                }

                // re-visit this node after its children have been visited
                visit_later.push(Tag::Close(el.tag));

                for child in el.children.iter_mut().rev() {
                    visit_later.push(Tag::Open(child));
                }
            }
            Tag::Close(tag_name) => {
                visitor.visit_close_tag(tag_name)?;
            }
        }
    }

    visitor.finish()?;

    Ok(())
}

// A visitor that transforms a Node tree to an html string
pub struct HtmlStringWriter<W> {
    html: W,
    indent_level: usize,
    indent: bool,
}

impl<W: fmt::Write> HtmlStringWriter<W> {
    pub fn new(inner: W, indent: bool) -> Self {
        Self {
            html: inner,
            indent_level: 0,
            indent,
        }
    }

    fn increment_indent(&mut self) {
        if self.indent {
            self.indent_level += 1;
        }
    }

    fn decrement_indent(&mut self) {
        if self.indent {
            self.indent_level -= 1;
        }
    }

    fn current_indent(&self) -> String {
        if self.indent {
            "  ".repeat(self.indent_level)
        } else {
            String::new()
        }
    }

    fn newline(&self) -> &'static str {
        if self.indent {
            "\n"
        } else {
            ""
        }
    }

    fn indent_text<'s>(&self, text: &'s str) -> Cow<'s, str> {
        if !self.indent {
            return Cow::Borrowed(text);
        }

        let replacement = format!("\n{}", self.current_indent());
        Cow::Owned(text.trim_end().replace('\n', &replacement))
    }
}

impl<W: fmt::Write> NodeVisitor for HtmlStringWriter<W> {
    type Error = fmt::Error;

    fn visit_open_tag(&mut self, el: &mut Node) -> Result<(), Self::Error> {
        write!(self.html, "{}<{}", self.current_indent(), el.tag)?;
        // css variables are set using the `style` attribute
        // merge them with any existing style attribute
        if !el.variables.is_empty() {
            match el.attributes.entry("style") {
                Entry::Vacant(v) => {
                    v.insert(el.variables.to_string());
                }
                Entry::Occupied(mut o) => {
                    let existing = o.get_mut();
                    *existing += &el.variables.to_string();
                }
            }
        }
        write!(self.html, "{}", el.attributes)?;
        write!(self.html, ">{}", self.newline())?;
        if !el.is_void() {
            self.increment_indent();
        }
        Ok(())
    }

    fn visit_close_tag(&mut self, tag: &'static str) -> Result<(), Self::Error> {
        self.decrement_indent();
        write!(
            self.html,
            "{}</{}>{}",
            self.current_indent(),
            tag,
            self.newline()
        )?;
        Ok(())
    }

    fn visit_text(&mut self, text: &str) -> Result<(), Self::Error> {
        let text = self.indent_text(text);
        write!(
            self.html,
            "{}{}{}",
            self.current_indent(),
            text,
            self.newline()
        )?;
        Ok(())
    }
}

// A visitor that inserts style & script nodes into a node tree
pub struct AssetInserter {
    style: Vec<Node>,
    script: Vec<Node>,
}

impl AssetInserter {
    pub fn new(style: Vec<Node>, script: Vec<Node>) -> Self {
        Self { style, script }
    }
}

impl NodeVisitor for AssetInserter {
    type Error = ();

    fn visit_open_tag(&mut self, el: &mut Node) -> Result<(), Self::Error> {
        if el.tag == "head" {
            el.children.append(&mut self.style);
        } else if el.tag == "body" {
            el.children.append(&mut self.script);
        }

        Ok(())
    }
}

// A visitor that collects all css & js snippets from the Node tree
// Using btreeset because the iteration order is defined, which makes it possible to test
pub struct SnippetCollector {
    pub css: BTreeSet<String>,
    pub js: BTreeSet<&'static str>,
}

impl SnippetCollector {
    pub fn new() -> Self {
        Self {
            css: BTreeSet::new(),
            js: BTreeSet::new(),
        }
    }
}

impl NodeVisitor for &mut SnippetCollector {
    type Error = ();

    fn visit_open_tag(&mut self, el: &mut Node) -> Result<(), Self::Error> {
        for asset in el.assets.iter_mut() {
            match asset {
                Asset::StyleSheet(css) => {
                    let mut localized_css = String::from(*css);
                    for (k, v) in el.variables.into_iter() {
                        let pattern = format!("var(--{})", k);
                        let replacement = format!("var(--{}-{})", k, v.suffix);
                        localized_css = localized_css.replace(&pattern, &replacement);
                    }
                    self.css.insert(localized_css);
                }
                Asset::JavaScript(js) => {
                    self.js.insert(js);
                }
            }
        }
        Ok(())
    }
}
