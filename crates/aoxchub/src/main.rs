mod app;
mod application;
mod domain;
mod features;
mod infrastructure;
mod shared;
mod testing;

fn main() {
    #[cfg(all(feature = "web", not(target_arch = "wasm32"), not(feature = "desktop")))]
    {
        eprintln!(
            "aoxchub(web): non-wasm target detected. Build for wasm32-unknown-unknown (dx serve) or enable desktop feature."
        );
    }

    #[cfg(not(all(feature = "web", not(target_arch = "wasm32"), not(feature = "desktop"))))]
    dioxus::launch(crate::app::app_root::AppRoot);
}
