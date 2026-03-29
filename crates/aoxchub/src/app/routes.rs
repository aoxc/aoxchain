#![allow(dead_code)]

use dioxus::prelude::*;

/// Temporary single-page routing stub kept only to satisfy the existing
/// module graph while AOXC Hub runs in desktop single-page mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {}

#[component]
pub fn RouterStub() -> Element {
    rsx! {
        div {}
    }
}
