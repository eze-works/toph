use std::error::Error;
use std::fs;
use toph::{component::*, tag::*, Node};

fn button(text: &str) -> Node {
    let css = r#"
        button {
            padding: 0.5rem 1.25rem;
            background-color: #ffffff;
            border-radius: 0.25rem;
        }
    "#;
    button_.set(t_(text)).stylesheet(css)
}
fn header() -> Node {
    let nav_elements = [
        "Features",
        "Screenshots",
        "Our pledge",
        "Pricing",
        "Questions?",
    ];

    let li_items = nav_elements.into_iter().map(|e| li_.set(a_.set(t_(e))));
    let nav = ul_.set(li_items);
    let login = button("Login");
    let cta = button("Get Started");
    header_.set([nav, div_.set([login, cta])])
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut html: Node = [
        doctype_,
        html_.set([head_, body_.set([css_reset(), header()])]),
    ]
    .into();

    fs::write("posthaven.html", html.write_to_string(true))?;
    Ok(())
}
