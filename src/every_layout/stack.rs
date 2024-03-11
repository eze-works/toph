use super::*;
use crate::{attr, tag::*, Node};

/// A container with children that are evenly spaced out vertically
pub fn stack(gap: u8, child: impl Into<Node>) -> Node {
    let space = spacing(gap);
    custom_("el-stack")
        .set(child)
        .css(include_str!("stack.css"))
        .var("el-stack-space", &space)
}
