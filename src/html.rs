#[doc(hidden)]
#[macro_export]
macro_rules! html_impl {
    (($parent:expr) ) => { };

    // div [<attributes>] { <children> }
    (($parent:expr) $tag:ident [$($attributes:tt)*] {$($children:tt)*} $($rest:tt)*) => {
        let tag = stringify!($tag);
        let attributes = $crate::attributes!($($attributes)*);
        #[allow(unused_mut)]
        let mut element = $crate::Node::new_element(tag, attributes.to_vec());
        $crate::html_impl!((&mut element) $($children)*);
        $parent.append_child(element);
        $crate::html_impl!(($parent) $($rest)*);
    };

    // div { <children> }
    (($parent:expr) $tag:ident {$($children:tt)*} $($rest:tt)*) => {
        let tag = stringify!($tag);
        let mut element = $crate::Node::new_element(tag, vec![]);
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
        let mut fragment = $crate::Node::new_fragment();
        $crate::html_impl!((&mut fragment) $($input)*);
        fragment
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testing() {
        //trace_macros!(true);
        let html = html! {
            html {
                head {
                    title {
                        "Hello world";
                    }
                }
                body {
                    div {
                        button {
                            i[class: "fa fa-facebook"] {}
                            "Submit";
                        }
                    }
                }
            }
        };
        //trace_macros!(false);

        println!("{:#?}", html);
    }
}
