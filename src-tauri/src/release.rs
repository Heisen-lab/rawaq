//! Release artifact packaging and automated SHA-256 checksum generation for Rawaq.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReleaseError {
    #[error("I/O error during release processing: {0}")]
    Io(String),
    #[error("Checksum mismatch for file {file}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },
    #[error("Missing release artifact file: {0}")]
    MissingFile(String),
    #[error("Malformed checksum line: {0}")]
    MalformedLine(String),
}

/// Metadata for a built release bundle artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseArtifact {
    pub filename: String,
    pub platform: String,
    pub sha256: String,
    pub size_bytes: u64,
}

/// Verification report checking generated release files against checksums.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseVerificationReport {
    pub verified_files: Vec<String>,
    pub mismatches: Vec<String>,
    pub missing_files: Vec<String>,
    pub is_valid: bool,
}

pub struct ReleaseManager;

impl ReleaseManager {
    /// Computes the SHA-256 digest in lowercase hexadecimal format for a given file.
    pub fn compute_sha256(path: &Path) -> Result<String, ReleaseError> {
        let mut file = fs::File::open(path).map_err(|e| ReleaseError::Io(e.to_string()))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        loop {
            let bytes_read = file.read(&mut buffer).map_err(|e| ReleaseError::Io(e.to_string()))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        Ok(format!("{:064x}", hasher.finalize()))
    }

    /// Formats a list of artifacts into standard GNU sha256sum format:
    /// `<sha256>  <filename>\n`
    pub fn generate_sha256sums_from_artifacts(artifacts: &[ReleaseArtifact]) -> String {
        let mut lines = Vec::new();
        for artifact in artifacts {
            lines.push(format!("{}  {}", artifact.sha256, artifact.filename));
        }
        lines.join("\n") + "\n"
    }

    /// Scans a release bundle directory, computes hashes, and generates `SHA256SUMS.txt` content.
    pub fn generate_sha256sums_from_dir(dir: &Path) -> Result<(String, Vec<ReleaseArtifact>), ReleaseError> {
        let mut artifacts = Vec::new();
        let entries = fs::read_dir(dir).map_err(|e| ReleaseError::Io(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| ReleaseError::Io(e.to_string()))?;
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if filename == "SHA256SUMS.txt" || filename.starts_with('.') {
                    continue;
                }
                let sha256 = Self::compute_sha256(&path)?;
                let metadata = fs::metadata(&path).map_err(|e| ReleaseError::Io(e.to_string()))?;
                let platform = if filename.ends_with(".msi") || filename.ends_with(".exe") {
                    "windows"
                } else if filename.ends_with(".dmg") || filename.ends_with(".app.tar.gz") {
                    "macos"
                } else if filename.ends_with(".AppImage") || filename.ends_with(".deb") {
                    "linux"
                } else {
                    "other"
                };

                artifacts.push(ReleaseArtifact {
                    filename,
                    platform: platform.to_string(),
                    sha256,
                    size_bytes: metadata.len(),
                });
            }
        }
        artifacts.sort_by(|a, b| a.filename.cmp(&b.filename));
        let checksums_text = Self::generate_sha256sums_from_artifacts(&artifacts);
        Ok((checksums_text, artifacts))
    }

    /// Verifies files in `dir` against a standard `SHA256SUMS.txt` content.
    pub fn verify_sha256sums(manifest_content: &str, dir: &Path) -> Result<ReleaseVerificationReport, ReleaseError> {
        let mut verified_files = Vec::new();
        let mut mismatches = Vec::new();
        let mut missing_files = Vec::new();

        for line in manifest_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return Err(ReleaseError::MalformedLine(line.to_string()));
            }
            let expected_hash = parts[0].to_lowercase();
            if expected_hash.len() != 64 {
                return Err(ReleaseError::MalformedLine(format!("Invalid hash length: {}", expected_hash)));
            }
            let filename = parts[1];
            let file_path = dir.join(filename);

            if !file_path.exists() {
                missing_files.push(filename.to_string());
                continue;
            }

            let actual_hash = Self::compute_sha256(&file_path)?;
            if actual_hash == expected_hash {
                verified_files.push(filename.to_string());
            } else {
                mismatches.push(format!("{}: expected {}, got {}", filename, expected_hash, actual_hash));
            }
        }

        let is_valid = mismatches.is_empty() && missing_files.is_empty() && !verified_files.is_empty();

        if let Some(m) = mismatches.first() {
            let file = m.split(':').next().unwrap_or_default().to_string();
            return Err(ReleaseError::ChecksumMismatch {
                file,
                expected: "mismatch".to_string(),
                actual: "different".to_string(),
            });
        }
        if let Some(miss) = missing_files.first() {
            return Err(ReleaseError::MissingFile(miss.clone()));
        }

        Ok(ReleaseVerificationReport {
            verified_files,
            mismatches,
            missing_files,
            is_valid,
        })
    }

    /// Saves `SHA256SUMS.txt` inside target directory.
    pub fn write_sha256sums_file(dir: &Path) -> Result<PathBuf, ReleaseError> {
        let (checksums_text, _) = Self::generate_sha256sums_from_dir(dir)?;
        let output_file = dir.join("SHA256SUMS.txt");
        fs::write(&output_file, checksums_text).map_err(|e| ReleaseError::Io(e.to_string()))?;
        Ok(output_file)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_automated_github_release_produces_checksums() {
        let temp_dir = tempfile::tempdir().expect("Create temp release dir");
        let release_path = temp_dir.path();

        // 1. Simulate multi-platform release bundles produced by Tauri release action
        // Windows
        let msi_path = release_path.join("rawaq_0.1.0_x64_en-US.msi");
        fs::write(&msi_path, b"RAWAG-WINDOWS-MSI-BINARY-MOCK-PAYLOAD-V0.1.0").unwrap();

        let exe_path = release_path.join("rawaq_0.1.0_x64-setup.exe");
        fs::write(&exe_path, b"RAWAG-WINDOWS-EXE-INSTALLER-PAYLOAD-V0.1.0").unwrap();

        // macOS
        let dmg_path = release_path.join("Rawaq_0.1.0_universal.dmg");
        fs::write(&dmg_path, b"RAWAG-MACOS-DMG-BUNDLE-MOCK-PAYLOAD-V0.1.0").unwrap();

        // Linux
        let appimage_path = release_path.join("rawaq_0.1.0_amd64.AppImage");
        fs::write(&appimage_path, b"RAWAG-LINUX-APPIMAGE-STANDALONE-PAYLOAD-V0.1.0").unwrap();

        let deb_path = release_path.join("rawaq_0.1.0_amd64.deb");
        fs::write(&deb_path, b"RAWAG-LINUX-DEBIAN-PACKAGE-PAYLOAD-V0.1.0").unwrap();

        // 2. Generate SHA256SUMS.txt from release directory
        let (manifest, artifacts) = ReleaseManager::generate_sha256sums_from_dir(release_path)
            .expect("Should generate checksums successfully");

        assert_eq!(artifacts.len(), 5);

        // Verify all 5 files are present and checksums are 64 hex characters
        for artifact in &artifacts {
            assert_eq!(artifact.sha256.len(), 64);
            assert!(artifact.sha256.chars().all(|c| c.is_ascii_hexdigit()));
            assert!(manifest.contains(&format!("{}  {}", artifact.sha256, artifact.filename)));
        }

        // 3. Write SHA256SUMS.txt to disk and verify against disk
        let checksums_file = ReleaseManager::write_sha256sums_file(release_path)
            .expect("Should write SHA256SUMS.txt");
        assert!(checksums_file.exists());

        let report = ReleaseManager::verify_sha256sums(&manifest, release_path)
            .expect("Verification must pass on pristine artifacts");
        assert!(report.is_valid);
        assert_eq!(report.verified_files.len(), 5);
        assert!(report.mismatches.is_empty());
        assert!(report.missing_files.is_empty());

        // 4. Fault Injection 1: Tampered/corrupted binary
        fs::write(&appimage_path, b"CORRUPTED-LINUX-BINARY-TAMPERED-IN-TRANSIT").unwrap();
        let tamper_result = ReleaseManager::verify_sha256sums(&manifest, release_path);
        match tamper_result {
            Err(ReleaseError::ChecksumMismatch { file, .. }) => {
                assert_eq!(file, "rawaq_0.1.0_amd64.AppImage");
            }
            other => panic!("Expected ChecksumMismatch error, got {:?}", other),
        }

        // Restore pristine appimage
        fs::write(&appimage_path, b"RAWAG-LINUX-APPIMAGE-STANDALONE-PAYLOAD-V0.1.0").unwrap();

        // 5. Fault Injection 2: Missing artifact
        fs::remove_file(&deb_path).unwrap();
        let missing_result = ReleaseManager::verify_sha256sums(&manifest, release_path);
        match missing_result {
            Err(ReleaseError::MissingFile(file)) => {
                assert_eq!(file, "rawaq_0.1.0_amd64.deb");
            }
            other => panic!("Expected MissingFile error, got {:?}", other),
        }
    }
}
