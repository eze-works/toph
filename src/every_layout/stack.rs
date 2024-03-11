use super::*;
use crate::{attr, tag::*, Node};

pub struct Stack;

impl Stack {
    pub fn create<const S: u8>(space: u8, child: impl Into<Node>) -> Node {
        let margin = spacing(space);
        let base = include_str!("stack.css");
        custom_("el-stack").set(child)
    }
}
