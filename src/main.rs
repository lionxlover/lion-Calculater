// Lion Calculator — main.rs
// Build: cargo build --release   (requires rustc ≥ 1.82, slint 1.17)
#![allow(clippy::too_many_arguments)]

mod app;
mod calculator;
mod history;
mod memory;
mod modes;
mod parser;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::run().map_err(Into::into)
}
