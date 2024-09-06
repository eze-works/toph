use crate::encode;

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum Attribute {
    /// A regular HTML attribute
    Regular(&'static str, String),
    /// Encodes the presence or absence of an HTML boolean attribute
    Bool(&'static str, bool),
}

impl From<(&'static str, String)> for Attribute {
    fn from(value: (&'static str, String)) -> Self {
        Attribute::Regular(value.0, value.1)
    }
}

impl From<(&'static str, &str)> for Attribute {
    fn from(value: (&'static str, &str)) -> Self {
        Attribute::Regular(value.0, value.1.to_string())
    }
}

impl From<(&'static str, bool)> for Attribute {
    fn from(value: (&'static str, bool)) -> Self {
        Attribute::Bool(value.0, value.1)
    }
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Attribute::Regular(k, v) => {
                write!(f, " {}=\"{}\"", k.replace('_', "-"), encode::attr(v))
            }
            Attribute::Bool(k, present) if *present => write!(f, " {}", k.replace('_', "-")),
            Attribute::Bool(_, _) => Ok(()),
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! attributes {
    ($($name:ident : $value:expr),*) => {{
        let mut attrs = vec![];
        $(
            let key = stringify!($name);
            let attribute = $crate::Attribute::from((key, $value));
            attrs.push(attribute);
        )*
        attrs
    }};
}
