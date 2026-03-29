mod app;
mod application;
mod domain;
mod features;
mod infrastructure;
mod shared;
mod testing;

use crate::app::app_root::AppRoot;

fn main() {
    dioxus::launch(AppRoot);
}
