//! Composable CSS Layout primitives
//!
//! Sources: <https://every-layout.dev>

use crate::{tag::*, Node};

impl From<u8> for ModularSpacing {
    fn from(value: u8) -> Self {
        if value == 0 {
            return ModularSpacing(String::new());
        }
        ModularSpacing(format!("{}rem", 0.325 * 1.5f64.powi(value as i32)))
    }
}

impl From<u16> for Measure {
    fn from(value: u16) -> Self {
        Measure(format!("{}ch", value))
    }
}

/// Expresses the spacing between elements as modular scale based on a line height of 1.5
pub struct ModularSpacing(String);

/// Expresses the measure (or width) of elements as a multiple of character width in current font.
pub struct Measure(String);

/// A container with children that are evenly spaced out vertically
///
/// ```text
///   x no gap
/// +---+
/// |   |
/// +---+
///   ^
///   | gap
///   v
/// +---+
/// |   |
/// +---+
///   ^
///   | gap
///   v
/// +---+
/// |   |
/// +---+
///   x no gap
/// ```
pub fn stack(gap: impl Into<ModularSpacing>, child: impl Into<Node>) -> Node {
    custom_("t-stack")
        .set(child)
        .stylesheet(include_str!("css/stack.css"))
        .var("t-stack-space", &gap.into().0)
}

/// A container with children that are evenly spaced out horizontally
///
///
/// ```text
/// +---+           +---+           +---+
/// |   | <- gap -> |   | <- gap -> |   |
/// +---+           +---+           +---+
/// ```
///
/// The elements may wrap
///
/// ```text
/// +---+           +---+           +---+
/// |   | <- gap -> |   | <- gap -> |   |
/// +---+           +---+           +---+
///   ^
///   | gap
///   v
/// +---+
/// |   | ...
/// +---+
/// ```
pub fn cluster(gap: impl Into<ModularSpacing>, child: impl Into<Node>) -> Node {
    custom_("t-cluster")
        .set(child)
        .stylesheet(include_str!("css/cluster.css"))
        .var("t-cluster-gap", &gap.into().0)
}

/// A simple padded box
///
/// ```text
/// +-----------+
/// |///////////|
/// |//content//|
/// |///////////|
/// +-----------+
/// ```
pub fn padded(padding: impl Into<ModularSpacing>, child: impl Into<Node>) -> Node {
    custom_("t-container")
        .set(child)
        .stylesheet(include_str!("css/padded.css"))
        .var("t-padded-padding", &padding.into().0)
}

/// A container whose child elements are horizontally centered
///
/// ```text
///             +---+
///             |   |
///             +---+
///             +---+
/// <---------->|   |<----------->
///             +---+
///             +---+
///             |   |
///             +---+
/// ```
pub fn center(child: impl Into<Node>) -> Node {
    custom_("t-center")
        .set(child)
        .stylesheet(include_str!("css/center.css"))
}

/// A container whose elements switch from a horizontal layout to a vertical one at the given width
/// threshold
///
///
/// The layout goes from this:
///  
/// ```text
/// +---+  +---+  +---+
/// |   |  |   |  |   |
/// +---+  +---+  +---+
/// ```
///
/// To this;
///
/// ```text
/// +---+
/// |   |
/// +---+
/// +---+
/// |   |
/// +---+
/// +---+
/// |   |
/// +---+
/// ```
pub fn switcher(
    gap: impl Into<ModularSpacing>,
    threshold: impl Into<Measure>,
    child: impl Into<Node>,
) -> Node {
    custom_("t-switcher")
        .set(child)
        .stylesheet(include_str!("css/switcher.css"))
        .var("t-switcher-gap", &gap.into().0)
        .var("t-switcher-threshold", &threshold.into().0)
}
