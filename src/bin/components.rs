#![allow(dead_code)]
use std::fs;
use toph::{tag::*, Node, attr};

#[derive(strum::Display)]
#[allow(non_camel_case_types)]
enum ButtonModifier {
    none,
    primary,
    secondary,
    warning,
}

fn button(text: &'static str, modifier: ButtonModifier) -> Node {
    button_((
        attr![@css=include_str!("./button.css"), data_modifier = modifier.to_string()],
        text,
    ))
}

struct TextInputProps<'v, 'e> {
    id: &'static str,
    label: Option<&'static str>,
    hint: Option<&'static str>,
    placeholder: Option<&'static str>,
    value: Option<&'v str>,
    error: Option<&'e str>,
    submit_button: Node,
}
fn text_input(
    TextInputProps {
        id,
        label,
        hint,
        placeholder,
        value,
        error,
        submit_button,
    }: TextInputProps,
) -> Node {
    custom_(
        "text-input",
        (
            attr![@css=include_str!("./textinput.css")],
            [
                custom_(
                    "text-input-element",
                    [
                        label.map(|s| label_((attr![for=id], s))).unwrap_or_default(),
                        hint.map(|s| custom_("text-input-hint", s))
                            .unwrap_or_default(),
                        error
                            .map(|s| custom_("text-input-error", s.to_string()))
                            .unwrap_or_default(),
                        div_(input_(attr![
                            id=id,
                            name=id,
                            type="text",
                            placeholder=placeholder.unwrap_or_default(),
                            value=value.unwrap_or_default().to_string()
                        ])),
                    ],
                ),
                submit_button,
            ],
        ),
    )
}

fn layout(child: impl Into<Node>) -> Node {
    [
        doctype_(),
        html_([
            head_([title_("My test site with html-string")]),
            body_([css_reset(), child.into()]),
        ]),
    ]
    .into()
}

fn css_reset() -> Node {
    custom_("css-reset", attr![@css=include_str!("./reset.css")])
}

fn main() {
    let mut view = layout([
        text_input(TextInputProps {
            id: "my-text-input",
            label: Some("label"),
            hint: Some("input text here"),
            placeholder: None,
            value: None,
            error: None,
            submit_button: button("Submit form", ButtonModifier::secondary),
        }),
        button("Hello world", ButtonModifier::none),
    ]);
    fs::write("./button.html", view.write_to_string()).unwrap();
}
