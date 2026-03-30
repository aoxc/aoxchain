// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Reads a binary file from disk.
///
/// Validation policy:
/// - the supplied path must not be blank,
/// - the path is normalized before filesystem access,
/// - filesystem failures are mapped into stable CLI-facing error strings.
pub fn read_file(path: &str) -> Result<Vec<u8>, String> {
    let normalized_path = normalize_required_path(path)?;
    fs::read(&normalized_path).map_err(|error| format!("FILE_READ_ERROR: {}", error))
}

/// Reads a UTF-8 text file from disk.
///
/// Validation policy:
/// - the supplied path must not be blank,
/// - the path is normalized before filesystem access,
/// - invalid UTF-8 content is surfaced as a read failure.
pub fn read_text_file(path: &str) -> Result<String, String> {
    let normalized_path = normalize_required_path(path)?;
    fs::read_to_string(&normalized_path).map_err(|error| format!("FILE_READ_ERROR: {}", error))
}

/// Writes binary data to disk using an atomic replace strategy.
///
/// Security and integrity policy:
/// - parent directories are created when absent,
/// - file content is written to a temporary sibling file first,
/// - the temporary file is flushed and then atomically renamed into place,
/// - file permissions are hardened on supported Unix platforms.
pub fn write_file(path: &str, data: &[u8]) -> Result<(), String> {
    let normalized_path = normalize_required_path(path)?;
    ensure_parent_dir_exists(&normalized_path)?;
    atomic_write_bytes(&normalized_path, data)
}

/// Writes UTF-8 text content to disk using the same atomic replace strategy as [`write_file`].
pub fn write_text_file(path: &str, data: &str) -> Result<(), String> {
    write_file(path, data.as_bytes())
}

/// Reads a JSON file from disk and deserializes it into the requested type.
///
/// Validation policy:
/// - the file must be readable as UTF-8,
/// - blank JSON payloads are rejected,
/// - malformed JSON is surfaced through a stable parse error.
pub fn read_json_file<T>(path: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let body = read_text_file(path)?;

    if body.trim().is_empty() {
        return Err("JSON_PARSE_ERROR: input JSON document is empty".to_string());
    }

    serde_json::from_str::<T>(&body).map_err(|error| format!("JSON_PARSE_ERROR: {}", error))
}

/// Serializes a value into canonical pretty JSON and writes it atomically to disk.
pub fn write_json_file<T>(path: &str, value: &T) -> Result<(), String>
where
    T: Serialize,
{
    let body = serde_json::to_string_pretty(value)
        .map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))?;

    write_text_file(path, &body)
}

/// Normalizes and validates a required filesystem path.
///
/// Policy:
/// - trims leading and trailing whitespace,
/// - rejects whitespace-only input,
/// - returns an owned normalized path string.
fn normalize_required_path(path: &str) -> Result<String, String> {
    let normalized = path.trim();

    if normalized.is_empty() {
        return Err("INVALID_ARGUMENT: path must not be blank".to_string());
    }

    Ok(normalized.to_string())
}

/// Ensures the parent directory of the target path exists.
///
/// Behavior:
/// - creates the full parent directory tree when needed,
/// - treats paths without a parent as valid and leaves them unchanged.
fn ensure_parent_dir_exists(path: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|error| format!("DIR_CREATE_ERROR: {}", error))?;
    }

    Ok(())
}

/// Writes bytes to a temporary sibling file and atomically renames it into place.
///
/// Integrity rationale:
/// - reduces partial-write risk,
/// - avoids leaving a truncated target file on write interruption,
/// - preserves a single final commit point through rename semantics.
fn atomic_write_bytes(path: &str, data: &[u8]) -> Result<(), String> {
    let target = Path::new(path);
    let temporary = temporary_sibling_path(target)?;

    let write_result = (|| -> Result<(), String> {
        let mut file = create_restricted_temp_file(&temporary)?;
        file.write_all(data)
            .map_err(|error| format!("FILE_WRITE_ERROR: {}", error))?;
        file.sync_all()
            .map_err(|error| format!("FILE_SYNC_ERROR: {}", error))?;
        drop(file);

        harden_file_permissions(&temporary)?;
        fs::rename(&temporary, target).map_err(|error| format!("FILE_RENAME_ERROR: {}", error))?;
        harden_file_permissions(target)?;

        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }

    write_result
}

/// Builds a temporary sibling path for atomic file replacement.
///
/// The temporary path is intentionally placed in the same directory as the
/// target path so that rename remains atomic on common filesystems.
fn temporary_sibling_path(target: &Path) -> Result<PathBuf, String> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            "INVALID_ARGUMENT: target path must reference a valid file name".to_string()
        })?;

    let process_id = std::process::id();
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("TIME_ERROR: {}", error))?
        .as_nanos();

    Ok(parent.join(format!(
        ".{}.tmp.{}.{}",
        file_name, process_id, timestamp_nanos
    )))
}

/// Creates a new temporary file with restricted permissions where supported.
///
/// Safety policy:
/// - uses create_new(true) to avoid accidental reuse,
/// - hardens the file mode immediately on Unix platforms.
fn create_restricted_temp_file(path: &Path) -> Result<File, String> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("FILE_CREATE_ERROR: {}", error))?;

    harden_file_permissions(path)?;

    Ok(file)
}

/// Hardens file permissions for sensitive AOXC artifacts on supported Unix platforms.
///
/// Current Unix policy:
/// - file mode is restricted to `0o600`.
fn harden_file_permissions(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|error| format!("FILE_PERMISSION_ERROR: {}", error))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    struct SampleJson {
        name: String,
        value: u32,
    }

    fn unique_path(label: &str) -> String {
        std::env::temp_dir()
            .join(format!("aoxckit-util-{}-{}.tmp", label, std::process::id()))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn normalize_required_path_rejects_blank_input() {
        let result = normalize_required_path("   ");
        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: path must not be blank".to_string())
        );
    }

    #[test]
    fn read_file_rejects_blank_path() {
        let result = read_file("   ");
        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: path must not be blank".to_string())
        );
    }

    #[test]
    fn read_text_file_rejects_blank_path() {
        let result = read_text_file("   ");
        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: path must not be blank".to_string())
        );
    }

    #[test]
    fn write_file_creates_parent_directories_and_persists_binary_content() {
        let path = unique_path("write-binary");
        let nested = format!("{}/nested/file.bin", path);
        let _ = fs::remove_dir_all(&path);

        write_file(&nested, b"abc123").expect("binary write must succeed");

        let content = fs::read(&nested).expect("written binary file must be readable");
        assert_eq!(content, b"abc123");

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn write_text_file_persists_utf8_content() {
        let path = unique_path("write-text");
        let _ = fs::remove_file(&path);

        write_text_file(&path, "hello world").expect("text write must succeed");

        let content = fs::read_to_string(&path).expect("written text file must be readable");
        assert_eq!(content, "hello world");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn write_file_atomically_replaces_existing_content() {
        let path = unique_path("atomic-replace");
        let _ = fs::remove_file(&path);

        write_text_file(&path, "old").expect("initial write must succeed");
        write_text_file(&path, "new").expect("replacement write must succeed");

        let content = fs::read_to_string(&path).expect("updated file must be readable");
        assert_eq!(content, "new");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn read_json_file_roundtrips_valid_document() {
        let path = unique_path("json-roundtrip");
        let value = SampleJson {
            name: "alpha".to_string(),
            value: 7,
        };

        write_json_file(&path, &value).expect("JSON write must succeed");
        let restored: SampleJson = read_json_file(&path).expect("JSON read must succeed");

        assert_eq!(restored, value);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn read_json_file_rejects_empty_document() {
        let path = unique_path("json-empty");
        fs::write(&path, "").expect("fixture file must be written");

        let result = read_json_file::<SampleJson>(&path);
        assert_eq!(
            result,
            Err("JSON_PARSE_ERROR: input JSON document is empty".to_string())
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn read_json_file_rejects_invalid_document() {
        let path = unique_path("json-invalid");
        fs::write(&path, "{not-json").expect("fixture file must be written");

        let result = read_json_file::<SampleJson>(&path);
        assert!(matches!(result, Err(error) if error.starts_with("JSON_PARSE_ERROR:")));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn write_json_file_emits_pretty_json() {
        let path = unique_path("json-pretty");
        let value = SampleJson {
            name: "beta".to_string(),
            value: 42,
        };

        write_json_file(&path, &value).expect("JSON write must succeed");

        let content = fs::read_to_string(&path).expect("JSON file must be readable");
        assert!(content.contains('\n'));
        assert!(content.contains("\"name\""));
        assert!(content.contains("\"value\""));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn write_file_rejects_blank_path() {
        let result = write_file("   ", b"abc");
        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: path must not be blank".to_string())
        );
    }

    #[test]
    fn write_text_file_rejects_blank_path() {
        let result = write_text_file("   ", "abc");
        assert_eq!(
            result,
            Err("INVALID_ARGUMENT: path must not be blank".to_string())
        );
    }

    #[cfg(unix)]
    #[test]
    fn write_file_hardens_permissions_on_unix() {
        use std::os::unix::fs::PermissionsExt;

        let path = unique_path("permissions");
        let _ = fs::remove_file(&path);

        write_text_file(&path, "secret").expect("secure write must succeed");

        let metadata = fs::metadata(&path).expect("metadata must be readable");
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        let _ = fs::remove_file(path);
    }
}
