#![allow(non_snake_case)]

use wasm_bindgen::prelude::*;

mod fmt;

#[wasm_bindgen]
pub fn format_size(size: f64) -> String {
  fmt::format_size(size)
}

#[wasm_bindgen]
pub fn format_time(time_secs: u64) -> String {
  fmt::format_time(time_secs)
}

