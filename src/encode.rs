use std::fmt::Write;
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

// The crate ensures that attribute values are double quoted, so the only character that needs to
// be encoded inside a value is the double quote
pub fn attr(value: &str) -> String {
    if !value.contains('"') {
        return value.into();
    }
    value.replace('"', "&quot;")
}

// "Percent encoding a url" is not straightforward because the url has to be parsed then eeach part
// percent-encoded according to specification.
//
// Thankfully, the `url` crate handles this.
pub fn url(input: &str) -> Option<String> {
    let url = match Url::parse(input) {
        Ok(u) => u.to_string(),
        Err(ParseError::RelativeUrlWithoutBase) => {
            // Relative URLs like `/about` or `../contact` can appear in HTML attributes. However,
            // the `url` crate does not parse partial URLs.
            //
            // The workaround is to
            // a) Use a fake base to parse the relative url,
            // b) Extract the now percent-encoded path, query and fragment
            // c) Re-ad any prefix the relative url had

            // For relative urls like `../../about.html`, extract that `../../` prefix
            let mut prefix = String::new();
            if input.starts_with('.') {
                for c in input.chars() {
                    if c == '.' {
                        prefix += ".";
                    } else if c == '/' {
                        prefix += "/";
                    } else {
                        break;
                    }
                }
            }
            // Disgard any leading `/`
            prefix = prefix.trim_end_matches('/').to_string();

            // Use a fake base to parse the relative url
            let fake = Url::parse("http://example.org").expect("valid url");
            let parsed = fake.join(input).ok()?;

            // Parsing the url would have stripped any `.../`-like prefixes. Re-add them
            let mut relative = prefix + parsed.path();

            if let Some(query) = parsed.query() {
                write!(relative, "?{}", query).ok()?;
            }

            if let Some(fragment) = parsed.fragment() {
                write!(relative, "#{}", fragment).ok()?;
            }

            relative
        }
        Err(_) => return None,
    };

    Some(url)
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
    fn percent_encoding() {
        // partial urls
        assert_eq!(url("/ test/ path"), Some("/%20test/%20path".into()),);
        assert_eq!(
            url("../../ test/ path"),
            Some("../../%20test/%20path".into()),
        );
        assert_eq!(url("/test?key= val"), Some("/test?key=%20val".into()));
        assert_eq!(url("/test#fragment"), Some("/test#fragment".into()));
        assert_eq!(
            url("/test?key= val#fragment"),
            Some("/test?key=%20val#fragment".into())
        );
        assert_eq!(url("?query=val"), Some("/?query=val".into()));
        assert_eq!(url("#fragment space"), Some("/#fragment%20space".into()));
    }
}
