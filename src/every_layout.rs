//! Css Layout primitives
//!
//! Source: <https://every-layout.dev>

mod stack;

pub use stack::stack;

fn spacing(level: u8) -> String {
    if level == 0 {
        return String::new();
    }
    format!("{}rem", 0.325 * 1.5f64.powi(level as i32))
}

fn sizing(level: u8) -> String {
    if level == 0 {
        return String::new();
    }
    format!("{}rem", 3.0 * 1.5f64.powi(level as i32))
}
