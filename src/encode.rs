// Context-dependent encoding for html
// There is a decent overview here:
// https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html#output-encoding

pub fn html(input: &str) -> String {
    if !input.contains(['"', '\'', '<', '>', '&']) {
        return String::from(input);
    }

    let mut escaped = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            c => {
                let mut buf = [0; 4];
                escaped.push_str(c.encode_utf8(&mut buf));
            }
        }
    }
    escaped
}

// The crate ensures that attribute values are double quoted, so the only character that needs to
// be encoded inside a value is the double quote
pub fn attr(value: &str) -> String {
    if !value.contains('"') {
        return value.into();
    }
    value.replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoding_html() {
        assert_eq!(
            html("Hello<>\"'&world;"),
            "Hello&lt;&gt;&quot;&#39;&amp;world;"
        );
    }

    #[test]
    fn encoding_attributes() {
        assert_eq!(attr("some\"attribute"), "some&quot;attribute");
    }
}
