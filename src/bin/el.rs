use std::fs;
use toph::{attr, layout::*, tag::*, Node};

fn stub() -> Node {
    let css = ".stub { width: 50px; height: 50px; background-color: black }";
    div_.stylesheet(css).with(attr![class = "stub"])
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
                        container(1, stack(1, [stub(), stub(), stub()])),
                        container(1, stack(4, [stub(), stub(), stub()])),
                        container(1, stack(6, [stub(), stub(), stub()])),
                    ],
                ),
                h1_.set("Center"),
                center([stub()]),
            ]),
        ]),
    ]
    .into();

    fs::write("every-layout.html", html.write_to_string(true)).unwrap();
}
