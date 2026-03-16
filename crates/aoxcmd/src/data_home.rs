use std::env;
use std::path::{Path, PathBuf};

pub const DEFAULT_HOME_DIR_NAME: &str = ".AOXC-Data";

#[must_use]
pub fn default_data_home() -> PathBuf {
    if let Ok(path) = env::var("AOXC_HOME") {
        return PathBuf::from(path);
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(DEFAULT_HOME_DIR_NAME);
    }

    PathBuf::from(DEFAULT_HOME_DIR_NAME)
}

#[must_use]
pub fn resolve_data_home(args: &[String]) -> PathBuf {
    arg_value(args, "--home")
        .map(PathBuf::from)
        .unwrap_or_else(default_data_home)
}

#[must_use]
pub fn join(home: &Path, section: &str) -> String {
    home.join(section).to_string_lossy().into_owned()
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find_map(|window| {
        if window[0] == key {
            Some(window[1].clone())
        } else {
            None
        }
    })
}
