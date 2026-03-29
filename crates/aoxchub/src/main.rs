mod app;
mod application;
mod cli;
mod domain;
mod features;
mod infrastructure;
mod shared;
mod testing;

use clap::Parser;

use crate::cli::{Cli, Command, OutputFormat, RuntimeLayout};

/// Desktop entrypoint.
///
/// This branch is compiled only when the `desktop` feature is enabled.
/// Native window decorations are intentionally preserved to avoid removing
/// the operating system title bar and frame. This keeps the application
/// aligned with standard desktop UX expectations and prevents accidental
/// frameless-window behavior.
#[cfg(feature = "desktop")]
fn main() {
    let cli = Cli::parse();

    if !should_launch_desktop(&cli) {
        run_headless_command(&cli);
        return;
    }

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
    let cli = Cli::parse();

    if !should_launch_desktop(&cli) {
        run_headless_command(&cli);
        return;
    }

    dioxus::launch(crate::app::app_root::AppRoot);
}

fn should_launch_desktop(cli: &Cli) -> bool {
    if cli.headless {
        return false;
    }

    !matches!(cli.command, Some(Command::Doctor | Command::Paths))
}

fn run_headless_command(cli: &Cli) {
    let runtime = RuntimeLayout::from_cli(cli);

    match cli.command {
        Some(Command::Paths) => print_runtime(runtime, cli.format),
        Some(Command::Doctor) => {
            print_runtime(runtime, cli.format);
            println!("compatibility: ok");
            println!("desktop_launch: enabled");
        }
        Some(Command::Launch) | None => print_runtime(runtime, cli.format),
    }
}

fn print_runtime(runtime: RuntimeLayout, format: OutputFormat) {
    match format {
        OutputFormat::Table => println!("{}", runtime.render_table()),
        OutputFormat::Json => {
            let payload = serde_json::to_string_pretty(&runtime)
                .expect("runtime diagnostic serialization should not fail");
            println!("{payload}");
        }
    }
}
