mod app;
mod application;
mod domain;
mod features;
mod infrastructure;
mod shared;
mod testing;

#[cfg(all(feature = "web", not(target_arch = "wasm32")))]
fn main() {
    eprintln!(
        "aoxchub(web): non-wasm target detected. Build for wasm32-unknown-unknown (dx serve) or enable desktop feature."
    );
}

#[cfg(any(
    all(feature = "desktop", not(target_arch = "wasm32")),
    all(feature = "web", target_arch = "wasm32")
))]
fn main() {
    dioxus::launch(crate::app::app_root::AppRoot);
}
