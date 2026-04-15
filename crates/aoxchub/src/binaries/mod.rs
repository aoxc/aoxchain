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
    let mut push_candidate = |id: &str, kind: BinarySourceKind, path: String, trust: TrustLevel| {
        if !PathBuf::from(&path).is_file() {
            return;
        }
        if !seen_paths.insert(path.clone()) {
            return;
        }
        out.push(BinaryCandidate {
            id: id.into(),
            kind,
            path: path.clone(),
            version: read_version(&path),
            trust,
            checksum_verified: None,
        });
    };

    let home = env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
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

    let local_release = "/workspace/aoxchain/target/release/aoxc".to_string();
    push_candidate(
        "local-release-build",
        BinarySourceKind::LocalReleaseBuild,
        local_release,
        TrustLevel::Experimental,
    );

    out
}
