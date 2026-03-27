// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

pub fn read_file(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("FILE_READ_ERROR: {}", e))
}

pub fn read_text_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("FILE_READ_ERROR: {}", e))
}

pub fn write_file(path: &str, data: &[u8]) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("DIR_CREATE_ERROR: {}", e))?;
    }

    fs::write(path, data).map_err(|e| format!("FILE_WRITE_ERROR: {}", e))
}

pub fn write_text_file(path: &str, data: &str) -> Result<(), String> {
    write_file(path, data.as_bytes())
}

pub fn read_json_file<T>(path: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let body = read_text_file(path)?;
    serde_json::from_str::<T>(&body).map_err(|error| format!("JSON_PARSE_ERROR: {}", error))
}

pub fn write_json_file<T>(path: &str, value: &T) -> Result<(), String>
where
    T: Serialize,
{
    let body = serde_json::to_string_pretty(value)
        .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))?;
    write_text_file(path, &body)
}
