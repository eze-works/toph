use crate::encode;

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum Attribute {
    /// A regular HTML attribute
    Regular(&'static str, String),
    /// Encodes the presence
    Bool(&'static str),
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Attribute::Regular(k, v) => {
                write!(f, " {}=\"{}\"", k.replace('_', "-"), encode::attr(v))
            }
            Attribute::Bool(k) => write!(f, " {}", k.replace('_', "-")),
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! attributes {
    // A regular attribute
    (($container:expr) $name:ident : $value:expr, $($rest:tt)*) => {
        let name = stringify!($name);
        let value = $value.to_string();
        $container.push($crate::Attribute::Regular(name, value));
        $crate::attributes!(($container) $($rest)*);
    };

    (($container:expr) $name:ident : $value:expr) => {
        let name = stringify!($name);
        let value = $value.to_string();
        $container.push($crate::Attribute::Regular(name, value));
    };

    // A boolean attribute
    (($container:expr) $name:expr, $($rest:tt)*) => {
        if !$name.is_empty() {
            $container.push($crate::Attribute::Bool($name));
        }
        $crate::attributes!(($container) $($rest)*);
    };

    (($container:expr) $name:expr) => {
        if !$name.is_empty() {
            $container.push($crate::Attribute::Bool($name));
        }
    };

}
