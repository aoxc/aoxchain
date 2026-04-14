use crate::domain::{BinaryCandidate, BinarySourceKind, TrustLevel};
use std::{collections::HashSet, env, fs, path::PathBuf};

fn read_version(path: &str) -> Option<String> {
    let output = std::process::Command::new(path)
        .arg("version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|s| s.trim().to_owned())
}

pub fn discover() -> Vec<BinaryCandidate> {
    let mut out = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();
    let mut seq: u64 = 0;
    let mut push_candidate = |id: &str, kind: BinarySourceKind, path: String, trust: TrustLevel| {
        if !PathBuf::from(&path).is_file() {
            return;
        }
        if !seen_paths.insert(path.clone()) {
            return;
        }
        seq = seq.saturating_add(1);
        out.push(BinaryCandidate {
            id: format!("{id}-{seq}"),
            kind,
            path: path.clone(),
            version: read_version(&path),
            trust,
            checksum_verified: None,
        });
    };

    let home = env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
    let installed_legacy = format!("{home}/.AOXCData/bin/aoxc");
    push_candidate(
        "installed-release-legacy",
        BinarySourceKind::InstalledRelease,
        installed_legacy,
        TrustLevel::Trusted,
    );
    let installed_current = format!("{home}/.aoxc/bin/current/aoxc");
    push_candidate(
        "installed-release-current",
        BinarySourceKind::InstalledRelease,
        installed_current,
        TrustLevel::Trusted,
    );
    push_candidate(
        "installed-release-mnt",
        BinarySourceKind::InstalledRelease,
        String::from("/mnt/xdbx/aoxc/bin/current/aoxc"),
        TrustLevel::Trusted,
    );

    let releases_root = PathBuf::from(format!("{home}/.AOXCData/releases"));
    if releases_root.is_dir()
        && let Ok(entries) = fs::read_dir(&releases_root)
    {
        let mut dirs: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        if let Some(bundle) = dirs.last() {
            let path = bundle.join("bin/aoxc");
            if path.is_file() {
                let p = path.display().to_string();
                push_candidate(
                    "versioned-bundle-legacy",
                    BinarySourceKind::VersionedBundle,
                    p,
                    TrustLevel::Trusted,
                );
            }
        }
    }

    let releases_root_current = PathBuf::from(format!("{home}/.aoxc/releases"));
    if releases_root_current.is_dir()
        && let Ok(entries) = fs::read_dir(&releases_root_current)
    {
        let mut dirs: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        if let Some(bundle) = dirs.last() {
            let path = bundle.join("bin/aoxc");
            if path.is_file() {
                let p = path.display().to_string();
                push_candidate(
                    "versioned-bundle-current",
                    BinarySourceKind::VersionedBundle,
                    p,
                    TrustLevel::Trusted,
                );
            }
        }
    }

    if let Ok(workspace_root) = env::var("AOXCHUB_WORKSPACE_ROOT") {
        let local_release = PathBuf::from(workspace_root).join("target/release/aoxc");
        push_candidate(
            "local-release-build-env-root",
            BinarySourceKind::LocalReleaseBuild,
            local_release.display().to_string(),
            TrustLevel::Experimental,
        );
    }

    if let Ok(cwd) = env::current_dir() {
        let local_release = cwd.join("target/release/aoxc");
        push_candidate(
            "local-release-build-cwd",
            BinarySourceKind::LocalReleaseBuild,
            local_release.display().to_string(),
            TrustLevel::Experimental,
        );

        if let Some(parent) = cwd.parent() {
            let parent_release = parent.join("target/release/aoxc");
            push_candidate(
                "local-release-build-parent",
                BinarySourceKind::LocalReleaseBuild,
                parent_release.display().to_string(),
                TrustLevel::Experimental,
            );
        }
    }

    if let Ok(exe) = env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        let sibling_release = exe_dir.join("aoxc");
        push_candidate(
            "local-release-build-sibling",
            BinarySourceKind::LocalReleaseBuild,
            sibling_release.display().to_string(),
            TrustLevel::Experimental,
        );
    }

    if let Some(path_var) = env::var_os("PATH") {
        for dir in env::split_paths(&path_var) {
            let from_path = dir.join("aoxc");
            push_candidate(
                "installed-from-path",
                BinarySourceKind::InstalledRelease,
                from_path.display().to_string(),
                TrustLevel::Trusted,
            );
        }
    }

    out
}
