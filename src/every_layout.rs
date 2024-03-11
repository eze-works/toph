//! Css Layout primitives
//!
//! Source: <https://every-layout.dev>

mod stack;
use crate::{attr, tag::*, Node};

fn spacing(level: u8) -> f64 {
    if level == 0 {
        return 0.0;
    }
    0.325 * 1.5f64.powi(level as i32)
}

fn sizing(level: u8) -> f64 {
    if level == 0 {
        return 0.0;
    }
    3.0 * 1.5f64.powi(level as i32)
}

pub struct Sizing(u8);
