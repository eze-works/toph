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
        let mut element = $crate::Node::element(tag, vec![]);
        $crate::html_impl!((&mut element) $($children)*);
        $parent.append_child(element);
        $crate::html_impl!(($parent) $($rest)*);
    };

    // (<expression>)
    (($parent:expr) ($expression:expr) $($rest:tt)*) => {
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
    use super::*;

    fn navigation() -> [crate::Node; 2] {
        let navigation = [("Home", "/about me"), ("Posts", "/posts")];
        navigation
            .map(|(caption, url)| {
                html! {
                    li [href: url] {
                        (caption)
                    }
                }
            })
    }

    #[test]
    fn testing() {
        let component = navigation();
        let html = html! {
            html {
                head {
                    title {
                        ("Hello world")
                    }
                }
                body {
                    (component)
                    div {
                        button {
                            i[class: "fa fa-facebook"] {}
                            ("Submit")
                        }
                    }
                }
            }
        };

        println!("{:#?}", html);
    }
}
