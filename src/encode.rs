use crate::allowlist::*;
use url::{ParseError, Url};

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

// The crate ensures that attribute values are doulbe quoted, so the only character that needs to
// be encoded inside a value is the double quote
pub fn attr(value: &str) -> String {
    if !value.contains('"') {
        return value.into();
    }
    value.replace("\"", "&quot;")
}

// Url attribute values are encoded in the same way as regular attribute values except that they
// need to be percent encoded first.
//
// "Percent encoding a url" is not straightforward. One has to parse it, the percent encode each
// part according to specification rules about what characters may appear un-encoded.
// Thankfully, the `url` crate handles this.
pub fn url(input: &str) -> Option<String> {
    let url = match Url::parse(input) {
        Ok(u) => u,
        Err(ParseError::RelativeUrlWithoutBase) => {
            println!("huh");
            let fake = Url::parse("http://example.org").expect("valid url");
            fake.join(input).ok()?
        }
        Err(err) => {
            dbg!(err);
            return None;
        }
    };

    if !ALLOWED_URL_SCHEMES.contains(&url.scheme()) {
        return None;
    }

    let percent_encoded = url.to_string();

    Some(percent_encoded.replace("\"", "&quot;"))
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

    #[test]
    fn encoding_url() {
        assert_eq!(
            url("mailto:matt@example\".com"),
            Some("mailto:matt@example&quot;.com".into())
        );

        assert_eq!(url("javascript:alert(1)"), None);
    }
}
