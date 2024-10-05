#[doc(hidden)]
#[macro_export]
macro_rules! html_impl {
    (($parent:expr) ) => { };

    // div [<attributes>] { <children> }
    (($parent:expr) $tag:ident [$($attributes:tt)*] {$($children:tt)*} $($rest:tt)*) => {
        let tag = String::from(stringify!($tag));
        let attributes = $crate::attributes!($($attributes)*);
        #[allow(unused_mut)]
        let mut element = $crate::Node::element(tag, attributes.to_vec());
        $crate::html_impl!((&mut element) $($children)*);
        $parent.append_child(element);
        $crate::html_impl!(($parent) $($rest)*);
    };

    // div { <children> }
    (($parent:expr) $tag:ident {$($children:tt)*} $($rest:tt)*) => {
        let tag = String::from(stringify!($tag));
        #[allow(unused_mut)]
        let mut element = $crate::Node::element(tag, vec![]);
        $crate::html_impl!((&mut element) $($children)*);
        $parent.append_child(element);
        $crate::html_impl!(($parent) $($rest)*);
    };

    // <expression>;
    (($parent:expr) $expression:expr ; $($rest:tt)*) => {
        $parent.append_child($crate::Node::from($expression));
        $crate::html_impl!(($parent) $($rest)*);
    };
}

/// This macro implements a syntax for creating HTML [`Node`](crate::Node)s.
///
/// # Syntax
///
/// An identifier followed by curly braces represents an HTML element:
///
/// ```
/// assert_eq!(
///     toph::html! {
///         div {}
///     }.to_string(),
///     "<div></div>",
/// );
/// ```
///
/// Elements can have attributes.
/// These are expressed as a key-value list seperated by commas.
/// Underscores in attribute names are converted to dashes.
///
/// Double quotes in attribute values are escaped.
///
/// ```
/// assert_eq!(
///     toph::html! {
///         div [data_count: "y\"all", id: "main"] {}
///     }.to_string(),
///     "<div data-count=\"y&quot;all\" id=\"main\"></div>"
/// );
/// ```
///
/// An attribute with a boolean value is treated as an [HTML boolean attribute](https://developer.mozilla.org/en-US/docs/Glossary/Boolean/HTML)
///
/// ```
/// use toph::html;
/// assert_eq!(
///     toph::html! {
///         div [async: false, readonly: true] {}
///     }.to_string(),
///     "<div readonly></div>"
/// );
/// ```
///
/// As a special case, the `data-tagname` attribute can be used to override the name of the html tag.
/// This is useful when the tag name is determined at runtime:
///
/// ```
/// let tagname = "div";
/// assert_eq!(
///     toph::html! {
///         thiscanbeanything[data_tagname: tagname] { }
///     }.to_string(),
///     "<div data-tagname=\"div\"></div>"
/// )
/// ```
///
/// Elements can have children:
///
/// ```
/// assert_eq!(
///     toph::html! {
///         div {
///             span {
///                 p {
///                 }
///             }
///         }
///     }.to_string(),
///     "<div><span><p></p></span></div>"
/// );
/// ```
///
/// Insert HTML-escaped text with [`text()`](crate::text).
/// Insert raw, unescaped text with [`raw_text()`](crate::text).
///
/// There is no other way to insert text as a child.
///
/// Anything that implements [`Display`](std::fmt::Display) can be passed as an argument.
///
/// Text must be terminated with a semicolon.
///
/// ```
/// assert_eq!(
///     toph::html! {
///         div[class: "\""] {
///             toph::text("<span>");
///             toph::raw_text("<span>");
///         }
///     }.to_string(),
///     "<div class=\"&quot;\">&lt;span&gt;<span></div>"
/// );
/// ```
///
/// Last, but by no means least, a child may also be any Rust expression that returns one or more `Node`s.
///
/// More specifically, in addition to expressions returning a single `Node`, any expression whose return value implements `IntoIterator<Item = Node>` qualifies.
/// Among other things, this means you can use:
/// - `Result<Node, E>`
/// - `Option<Node>`,
/// - Any iterator that yields `Node`s
/// - An array or `Vec` of `Node`s.
///
/// Expressions must also be terminated with a semicolon.
///
/// ```
/// let option = Some(toph::html!{ toph::text("option"); });
/// let iterator = (0..=2).into_iter().map(|n| toph::html! { toph::text(n); });
/// assert_eq!(
///     toph::html! {
///         div {
///             option;
///             iterator;
///         }
///     }.to_string(),
///     "<div>option012</div>"
/// );
/// ```
///
#[macro_export]
macro_rules! html {
    ($($input:tt)*) => {{
        let mut fragment = $crate::Node::fragment();
        $crate::html_impl!((&mut fragment) $($input)*);
        fragment
    }};
}

#[cfg(test)]
mod tests {
    use crate::{raw_text, text, Node};
    #[test]
    fn empty_element() {
        assert_eq!(
            html! {
                div {}
            }
            .to_string(),
            "<div></div>"
        );

        // snake_case elements get converted to kebab-case
        assert_eq!(
            html! {
                custom_element {}
            }
            .to_string(),
            "<custom-element></custom-element>"
        );
    }

    #[test]
    fn element_with_attributes() {
        assert_eq!(
            html! {
                div [class: "container", readonly: true, async: false] {}
            }
            .to_string(),
            "<div class=\"container\" readonly></div>"
        );

        // snake_case keys get converted to kebab-case
        assert_eq!(
            html! {
                div [data_one: "two"] {}
            }
            .to_string(),
            "<div data-one=\"two\"></div>"
        );

        // quotes are encoded in attributes
        assert_eq!(
            html! {
                div [key: "a \"templating\" engine"] {}
            }
            .to_string(),
            "<div key=\"a &quot;templating&quot; engine\"></div>"
        );
    }

    #[test]
    fn element_with_attributes_and_children() {
        assert_eq!(
            html! {
                div[class: "container"] {
                    p {}
                    span {}
                }
            }
            .to_string(),
            "<div class=\"container\"><p></p><span></span></div>"
        )
    }

    #[test]
    fn void_elements() {
        assert_eq!(
            html! {
                img {
                    p {}
                }
            }
            .to_string(),
            "<img>"
        );

        // void elements are case-insensitively recognized
        assert_eq!(
            html! {
                IMG { p {} }
            }
            .to_string(),
            "<img>"
        );
    }

    #[test]
    fn doctype_element_is_recognized() {
        assert_eq!(
            html! {
                DOCtype {}
            }
            .to_string(),
            "<!doctype>"
        );
    }

    #[test]
    fn escaping_strings() {
        assert_eq!(
            html! {
                text("foo");
                text("<span>");
                raw_text("<span>");
            }
            .to_string(),
            "foo&lt;span&gt;<span>"
        )
    }

    #[test]
    fn interpolating_expressions() {
        // interpolating strings
        assert_eq!(
            html! {
                div {}
                text("hello");
                span {}
            }
            .to_string(),
            "<div></div>hello<span></span>"
        );

        // interpolating another node
        let node = html! {
            button {
                text("submit");
            }
        };

        assert_eq!(
            html! {
                form {
                    node;
                }
            }
            .to_string(),
            "<form><button>submit</button></form>"
        );

        // interpolating a list of nodes
        let form = [
            html! { input {} },
            html! { button[type: "submit"] {} },
            html! { select {} },
        ];

        assert_eq!(
            html! {
                form {
                    form;
                }
            }
            .to_string(),
            "<form><input><button type=\"submit\"></button><select></select></form>"
        );

        // interpolating other iterator-like structures
        let no: Option<Node> = None;
        let yes = Some(html! { text("yes"); });
        let success: Result<Node, ()> = Ok(html! { text("success"); });

        assert_eq!(
            html! {
                no;
                yes;
                success;
            }
            .to_string(),
            "yessuccess"
        );
    }

    #[test]
    fn overriding_the_tagname() {
        assert_eq!(
            html! {
                custom[data_tagname: "h1"] { }
            }
            .to_string(),
            "<h1 data-tagname=\"h1\"></h1>"
        );
    }
}
