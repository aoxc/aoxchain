mod app;
mod application;
mod domain;
mod features;
mod infrastructure;
mod shared;
mod testing;

/// Desktop entrypoint.
///
/// This branch is compiled only when the `desktop` feature is enabled.
/// Native window decorations are intentionally preserved to avoid removing
/// the operating system title bar and frame. This keeps the application
/// aligned with standard desktop UX expectations and prevents accidental
/// frameless-window behavior.
#[cfg(feature = "desktop")]
fn main() {
    use dioxus::desktop::tao::dpi::LogicalSize;
    use dioxus::desktop::{Config, WindowBuilder};

    let window = WindowBuilder::new()
        .with_title("AOXC Hub")
        .with_inner_size(LogicalSize::new(1440.0, 900.0))
        .with_resizable(true);

    let config = Config::new().with_window(window);

    dioxus::LaunchBuilder::new()
        .with_cfg(config)
        .launch(crate::app::app_root::AppRoot);
}

/// Non-desktop fallback entrypoint.
///
/// This fallback remains intentionally minimal so auxiliary build flows
/// such as server-side helper compilation can complete without depending
/// on desktop-only APIs.
#[cfg(not(feature = "desktop"))]
fn main() {
    dioxus::launch(crate::app::app_root::AppRoot);
}
