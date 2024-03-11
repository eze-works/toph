use std::fs;
use toph::{attr, every_layout::*, tag::*, Node};

fn container() -> Node {
    let css = ".box { border: 1px solid black; width: 100px; height: 10px; }";
    div_.css(css).with(attr![class = "box"])
}

fn main() {
    let mut html: Node = [
        doctype_,
        html_.set([
            head_.set(title_.set("Every Layout")),
            body_.set([
                h1_.set("Stack"),
                stack(
                    5,
                    [
                        stack(1, [container(), container(), container()]),
                        stack(2, [container(), container(), container()]),
                        stack(3, [container(), container(), container()]),
                        stack(4, [container(), container(), container()]),
                    ],
                ),
            ]),
        ]),
    ]
    .into();

    fs::write("every-layout.html", html.write_to_string(true)).unwrap();
}
