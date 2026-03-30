use crate::domain::{BinaryCandidate, BinarySourceKind, TrustLevel};
use std::{env, fs, path::PathBuf};

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
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
    let installed = format!("{home}/.AOXCData/bin/aoxc");
    if PathBuf::from(&installed).is_file() {
        out.push(BinaryCandidate {
            id: "installed-release".into(),
            kind: BinarySourceKind::InstalledRelease,
            path: installed.clone(),
            version: read_version(&installed),
            trust: TrustLevel::Trusted,
            checksum_verified: None,
        });
    }

    let releases_root = PathBuf::from(format!("{home}/.AOXCData/releases"));
    if releases_root.is_dir() {
        if let Ok(entries) = fs::read_dir(&releases_root) {
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
                    out.push(BinaryCandidate {
                        id: "versioned-bundle".into(),
                        kind: BinarySourceKind::VersionedBundle,
                        path: p.clone(),
                        version: read_version(&p),
                        trust: TrustLevel::Trusted,
                        checksum_verified: None,
                    });
                }
            }
        }
    }

    let local_release = "/workspace/aoxchain/target/release/aoxc".to_string();
    if PathBuf::from(&local_release).is_file() {
        out.push(BinaryCandidate {
            id: "local-release-build".into(),
            kind: BinarySourceKind::LocalReleaseBuild,
            path: local_release.clone(),
            version: read_version(&local_release),
            trust: TrustLevel::Experimental,
            checksum_verified: None,
        });
    }

    out
}
