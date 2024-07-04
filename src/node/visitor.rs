use super::{tag::*, Node};
use std::borrow::Cow;
use std::collections::btree_map::Entry;
use std::collections::BTreeSet;
use std::fmt;

enum Tag<'n> {
    Open(&'n mut Node),
    Close(&'static str),
}

// The visitor pattern[1] is used for traversing a Node tree.
//
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
