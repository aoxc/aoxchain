#![allow(non_snake_case)]
mod app;
mod components;
mod hooks;
mod i18n;
mod layouts;
mod modules;
mod route;
mod services;
mod state;
mod types;
mod views;

use app::App;

fn main() {
    dioxus::launch(App);
}
