# Toph

Toph is an HTML generation library.
It's implemented as a declarative macro, `html!`, that transforms your markup into imperative code to build up an HTML tree.
This tree can then be converted to a string.

[Documentation](https://docs.rs/toph/)

[Crates.io](https://crates.io/crates/toph)

```rust
let _ = toph::html! {
    doctype {}
    html {
        title {
            toph::text("hello world");
        }
    }
    body {
        p[class: "intro"] {
            toph::text("This is an example of the ");
            a[href: "https://github.com/eze-works/toph"] {
                toph::text(" template language");
            }
        }
    }
};
```
