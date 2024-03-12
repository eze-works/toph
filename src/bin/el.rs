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
                        padded(1, stack(1, [stub(), stub(), stub()])),
                        padded(1, stack(4, [stub(), stub(), stub()])),
                        padded(1, stack(6, [stub(), stub(), stub()])),
                    ],
                ),
                h1_.set("Center"),
                center([stub()]),
                h1_.set("Cluster"),
                cluster(
                    5,
                    [
                        stub(),
                        stub(),
                        stub(),
                        stub(),
                        stub(),
                        stub(),
                        stub(),
                        stub(),
                    ],
                ),
                h1_.set("Switcher"),
                switcher(4, 60, [stub(), stub(), stub(), stub()]),
                h1_.set("Cover"),
                cover(stub(), None, None, Some(50)),
                cover(stub(), Some(stub()), None, None),
                cover(stub(), Some(stub()), Some(stub()), None),
                h1_.set("Fluid Grid"),
                fluid_grid(
                    10,
                    1,
                    [
                        stub(),
                        stub(),
                        stub(),
                        stub(),
                        stub(),
                        stub(),
                        stub(),
                        stub(),
                        stub(),
                    ],
                ),
                h1_.set("Frame"),
                frame(
                    (3, 4),
                    img_.with(
                        attr![src="https://img.freepik.com/free-photo/painting-mountain-lake-with-mountain-background_188544-9126.jpg"]
                        )
                    ).with(attr![style="width: 400px;"]),
                h1_.set("Manual SVG"),
                svg_.with(attr![width="32", height="32", viewBox="0 0 32 32"])
                    .set(custom_("path")
                         .with(attr![
                               fill="none",
                               stroke="currentColor",
                               stroke_linecap="round",
                               stroke_linejoin="round",
                               stroke_width="2",
                               d="M2 6v24h28V6Zm0 9h28M7 3v6m6-6v6m6-6v6m6-6v6"
                         ]))


            ]),
        ]),
    ]
    .into();

    fs::write("every-layout.html", html.write_to_string(true)).unwrap();
}
