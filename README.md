# Toph

Toph is an HTML generation library.
It's implemented as a declarative macro, `html!`, that transforms your markup into imperative code to build up an HTML tree.
This tree can then be converted to a string.

[Documentation](https://docs.rs/toph/)

[Crates.io](https://crates.io/crates/toph)

```rust
use toph::{text, html};

let _ = html! {
    doctype {}
    html {
        title {
            text("hello world");
        }
    }
    body {
        p[class: "intro"] {
            text("This is an example of the ");
            a[href: "https://github.com/eze-works/toph"] {
                text(" template language");
            }
        }
    }
};
```
