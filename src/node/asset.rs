// Element assets refer to CSS and Javascript associated with a given element

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Asset {
    Css(&'static str),
    JavaScript(&'static str),
}
