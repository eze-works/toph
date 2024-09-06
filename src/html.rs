#[doc(hidden)]
#[macro_export]
macro_rules! html_impl {
    (($parent:expr) ) => { };

    // div [<attributes>] { <children> }
    (($parent:expr) $tag:ident [$($attributes:tt)*] {$($children:tt)*} $($rest:tt)*) => {
        let tag = stringify!($tag);
        let attributes = $crate::attributes!($($attributes)*);
        #[allow(unused_mut)]
        let mut element = $crate::Node::element(tag, attributes.to_vec());
        $crate::html_impl!((&mut element) $($children)*);
        $parent.append_child(element);
        $crate::html_impl!(($parent) $($rest)*);
    };

    // div { <children> }
    (($parent:expr) $tag:ident {$($children:tt)*} $($rest:tt)*) => {
        let tag = stringify!($tag);
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
    #[test]
    fn empty_element() {
        assert_eq!(
            html! {
                div {}
            }
            .serialize(),
            "<div></div>"
        );

        // snake_case elements get converted to kebab-case
        assert_eq!(
            html! {
                custom_element {}
            }.serialize(),
            "<custom-element></custom-element>"
        );
    }

    #[test]
    fn element_with_attributes() {
        assert_eq!(
            html! {
                div [class: "container", readonly: true, async: false] {}
            }
            .serialize(),
            "<div class=\"container\" readonly></div>"
        );

        // snake_case keys get converted to kebab-case
        assert_eq!(
            html! {
                div [data_one: "two"] {}
            }
            .serialize(),
            "<div data-one=\"two\"></div>"
        );

        // quotes are encoded in attributes
        assert_eq!(
            html! {
                div [key: "a \"templating\" engine"] {}
            }.serialize(),
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
            .serialize(),
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
            }.serialize(),
            "<img>"
        );

        // void elements are case-insensitively recognized
        assert_eq! (
            html! {
                IMG { p {} }
            }.serialize(),
            "<IMG>"
        );
    }

    #[test]
    fn doctype_element_is_recognized() {
        assert_eq!(
            html! {
                DOCtype {}
            }.serialize(),
            "<!doctype>"
        );
    }

    #[test]
    fn escaping_strings() {
        use crate::raw_text;
        assert_eq! (
            html! {
                "foo";
                "<span>";
                raw_text("<span>");
            }.serialize(),
            "foo&lt;span&gt;<span>"
        )
    }

    #[test]
    fn interpolating_expressions() {
        // interpolating strings
        assert_eq! (
            html! {
                div {}
                "hello";
                span {}
            }.serialize(),
            "<div></div>hello<span></span>"
        );

        // interpolating another node
        let node = html! {
            button {
                "submit";
            }
        };

        assert_eq!(
            html!{
                form {
                    node;
                }
            }.serialize(),
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
            }.serialize(),
            "<form><input><button type=\"submit\"></button><select></select></form>"
        );
    }
}
