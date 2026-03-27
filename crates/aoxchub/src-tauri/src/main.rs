// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

// Prevents an additional console window on Windows release builds.
// This attribute must remain in place for production desktop packaging.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    aoxchub_lib::run();
}
