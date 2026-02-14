use leptos::prelude::*;

mod app;
use app::App;
mod components;
mod pages;

fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    mount_to_body(App);
}
