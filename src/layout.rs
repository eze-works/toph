//! Css Layout primitives
//!
//! Source: <https://every-layout.dev>

use crate::{attr, tag::*, Node};

fn spacing(level: u8) -> String {
    if level == 0 {
        return String::new();
    }
    format!("{}rem", 0.325 * 1.5f64.powi(level as i32))
}

impl From<u8> for ModularSpacing {
    fn from(value: u8) -> Self {
        if value == 0 {
            return ModularSpacing(String::new());
        }
        ModularSpacing(format!("{}rem", 0.325 * 1.5f64.powi(value as i32)))
    }
}

/// Expresses the spacing between elements as modular scale based on a line height of 1.5
pub struct ModularSpacing(String);

/// A container with children that are evenly spaced out vertically
pub fn stack(gap: impl Into<ModularSpacing>, child: impl Into<Node>) -> Node {
    custom_("t-stack")
        .set(child)
        .stylesheet(include_str!("css/stack.css"))
        .var("t-stack-space", &gap.into().0)
}

/// A simple padded box
pub fn container(padding: impl Into<ModularSpacing>, child: impl Into<Node>) -> Node {
    custom_("t-container")
        .set(child)
        .stylesheet(include_str!("css/container.css"))
        .var("t-container-padding", &padding.into().0)
}

/// A container whose child elements are horizontally centered
pub fn center(child: impl Into<Node>) -> Node {
    custom_("t-center")
        .set(child)
        .stylesheet(include_str!("css/center.css"))
}

/// A group of elements evenly spaced out and laid out horizontally
///
/// The elements may wrap
pub fn cluster(gap: impl Into<ModularSpacing>, child: impl Into<Node>) -> Node {
    custom_("t-cluster")
        .set(child)
        .stylesheet(include_str!("css/cluster.css"))
        .var("t-cluster-gap", &gap.into().0)
}
