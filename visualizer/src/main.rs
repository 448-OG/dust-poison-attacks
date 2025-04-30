#![allow(non_snake_case)]

use dioxus::prelude::*;

mod views;
use views::*;

mod header;
use header::*;

mod utils;

// mod fetch_parser;
// use fetch_parser::*;

mod fetch_util;
pub(crate) use fetch_util::*;

mod svg_assets;

mod app;
pub(crate) use app::*;

fn main() {
    launch(App);
}
