use crate::storage::{OrganizeRule, Storage, StorageError, UndoOperation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrganizerError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Source folder does not exist or is not a directory: {0}")]
    InvalidSourceFolder(String),

    #[error("Checksum mismatch: expected {expected}, actual {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Target file already exists and overwrite is forbidden: {0}")]
    CollisionForbidden(String),

    #[error("Undo failed: {0}")]
    UndoFailed(String),
}

/// A planned move action calculated during dry-run preview.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannedMove {
    pub source_path: String,
    pub original_destination_path: String,
    pub resolved_destination_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_hash: String,
    pub rule_name: String,
    pub collision_detected: bool,
}

/// An item skipped during dry-run calculation (e.g. no matching rule or directory).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkippedItem {
    pub path: String,
    pub file_name: String,
    pub reason: String,
}

/// The complete dry-run preview plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DryRunPlan {
    pub plan_id: String,
    pub source_folder: String,
    pub moves: Vec<PlannedMove>,
    pub skipped: Vec<SkippedItem>,
}

/// Detail for each file processed during execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionItemReport {
    pub source_path: String,
    pub destination_path: String,
    pub status: String, // "COMPLETED" | "FAILED" | "SKIPPED"
    pub error_message: Option<String>,
}

/// Execution report after applying the plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionReport {
    pub batch_id: String,
    pub total_planned: usize,
    pub successful: usize,
    pub failed: usize,
    pub items: Vec<ExecutionItemReport>,
}

/// Report after undoing an organization batch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UndoReport {
    pub batch_id: String,
    pub total_operations: usize,
    pub restored: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Optional simulation hook for testing fault injection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaultInjection {
    /// Simulates bit rot / checksum mismatch on files matching the predicate.
    CorruptDestinationChecksum(String),
}

/// Computes SHA-256 of any file on disk.
pub fn compute_file_sha256(path: &Path) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Generates a safe, non-colliding file path by appending ` (1)`, ` (2)`, etc.
pub fn resolve_collision_path(target_folder: &Path, file_name: &str) -> PathBuf {
    let p = Path::new(file_name);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or(file_name);
    let ext = p.extension().and_then(|e| e.to_str());

    let mut counter = 1;
    loop {
        let candidate_name = match ext {
            Some(e) => format!("{stem} ({counter}).{e}"),
            None => format!("{stem} ({counter})"),
        };
        let candidate_path = target_folder.join(candidate_name);
        if !candidate_path.exists() {
            return candidate_path;
        }
        counter += 1;
    }
}

pub struct SafeOrganizerEngine;

impl SafeOrganizerEngine {
    /// Computes a Dry-Run Plan without mutating a single byte on the filesystem.
    /// **Guarantee**: Completely read-only inspection.
    pub fn compute_dry_run(
        source_folder: &Path,
        rules: &[OrganizeRule],
    ) -> Result<DryRunPlan, OrganizerError> {
        if !source_folder.exists() || !source_folder.is_dir() {
            return Err(OrganizerError::InvalidSourceFolder(
                source_folder.to_string_lossy().to_string(),
            ));
        }

        let plan_id = format!("plan-{}", chrono::Utc::now().timestamp_micros());
        let mut moves = Vec::new();
        let mut skipped = Vec::new();

        let entries = fs::read_dir(source_folder)?;
        for entry_res in entries {
            let entry = entry_res?;
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    skipped.push(SkippedItem {
                        path: path.to_string_lossy().to_string(),
                        file_name,
                        reason: format!("Metadata error: {e}"),
                    });
                    continue;
                }
            };

            // Directories are not moved by the auto-organizer
            if metadata.is_dir() {
                skipped.push(SkippedItem {
                    path: path.to_string_lossy().to_string(),
                    file_name,
                    reason: "Directory - untouched by file organizer".to_string(),
                });
                continue;
            }

            let file_ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Find matching enabled rule
            let mut matched_rule: Option<&OrganizeRule> = None;
            for rule in rules {
                if !rule.is_enabled {
                    continue;
                }
                let exts: Vec<String> = rule
                    .extensions
                    .split(',')
                    .map(|s| s.trim().trim_start_matches('.').to_lowercase())
                    .collect();

                if exts.iter().any(|ext| ext == &file_ext || ext == "*") {
                    matched_rule = Some(rule);
                    break;
                }
            }

            let Some(rule) = matched_rule else {
                skipped.push(SkippedItem {
                    path: path.to_string_lossy().to_string(),
                    file_name,
                    reason: "No matching rule".to_string(),
                });
                continue;
            };

            let target_dir = PathBuf::from(&rule.target_folder);
            let dest_path = target_dir.join(&file_name);

            let collision = dest_path.exists();
            let resolved_dest = if collision {
                resolve_collision_path(&target_dir, &file_name)
            } else {
                dest_path.clone()
            };

            let file_hash = match compute_file_sha256(&path) {
                Ok(h) => h,
                Err(e) => {
                    skipped.push(SkippedItem {
                        path: path.to_string_lossy().to_string(),
                        file_name,
                        reason: format!("Read error: {e}"),
                    });
                    continue;
                }
            };

            moves.push(PlannedMove {
                source_path: path.to_string_lossy().to_string(),
                original_destination_path: dest_path.to_string_lossy().to_string(),
                resolved_destination_path: resolved_dest.to_string_lossy().to_string(),
                file_name,
                file_size: metadata.len(),
                file_hash,
                rule_name: rule.name.clone(),
                collision_detected: collision,
            });
        }

        Ok(DryRunPlan {
            plan_id,
            source_folder: source_folder.to_string_lossy().to_string(),
            moves,
            skipped,
        })
    }

    /// Executes the plan using the strict File Safety Contract:
    /// 1. Insert PENDING in `undo_operations`.
    /// 2. Ensure target directory exists.
    /// 3. Copy file to destination.
    /// 4. Verify size and SHA-256 match source 100%.
    /// 5. If verified -> remove source, update COMPLETED.
    /// 6. If mismatch / error -> remove destination copy, keep source untouched, update FAILED.
    pub fn execute_plan(
        storage: &Storage,
        plan: &DryRunPlan,
        fault_injection: Option<FaultInjection>,
    ) -> Result<ExecutionReport, OrganizerError> {
        let batch_id = format!("batch-{}", chrono::Utc::now().timestamp_millis());
        let mut items_report = Vec::new();
        let mut successful = 0;
        let mut failed = 0;

        for m in &plan.moves {
            let src_path = PathBuf::from(&m.source_path);
            let mut dest_path = PathBuf::from(&m.resolved_destination_path);

            // Double check collision at execution time
            if dest_path.exists() {
                if let Some(parent) = dest_path.parent() {
                    dest_path = resolve_collision_path(parent, &m.file_name);
                }
            }

            let op_id = storage.insert_undo_operation(&UndoOperation {
                id: None,
                batch_id: batch_id.clone(),
                source_path: src_path.to_string_lossy().to_string(),
                destination_path: dest_path.to_string_lossy().to_string(),
                file_hash: m.file_hash.clone(),
                file_size: m.file_size as i64,
                status: "PENDING".to_string(),
                error_message: None,
                created_at: None,
            })?;

            // Create parent destination folder
            if let Some(parent) = dest_path.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        storage.update_undo_operation_status(
                            op_id,
                            "FAILED",
                            Some(&format!("Cannot create target folder: {e}")),
                        )?;
                        items_report.push(ExecutionItemReport {
                            source_path: m.source_path.clone(),
                            destination_path: dest_path.to_string_lossy().to_string(),
                            status: "FAILED".to_string(),
                            error_message: Some(format!("Cannot create target folder: {e}")),
                        });
                        failed += 1;
                        continue;
                    }
                }
            }

            // Copy file to destination
            if let Err(e) = fs::copy(&src_path, &dest_path) {
                storage.update_undo_operation_status(
                    op_id,
                    "FAILED",
                    Some(&format!("Copy failed: {e}")),
                )?;
                items_report.push(ExecutionItemReport {
                    source_path: m.source_path.clone(),
                    destination_path: dest_path.to_string_lossy().to_string(),
                    status: "FAILED".to_string(),
                    error_message: Some(format!("Copy failed: {e}")),
                });
                failed += 1;
                continue;
            }

            // Handle fault injection simulation if specified
            if let Some(FaultInjection::CorruptDestinationChecksum(ref bad_file)) = fault_injection
            {
                if m.file_name == *bad_file {
                    // Corrupt destination by appending corrupted bytes
                    let _ = fs::write(&dest_path, b"CORRUPTED_BYTES_DIFFERENT_HASH");
                }
            }

            // Verify checksum and size of destination copy
            let dest_hash = match compute_file_sha256(&dest_path) {
                Ok(h) => h,
                Err(e) => {
                    let _ = fs::remove_file(&dest_path);
                    storage.update_undo_operation_status(
                        op_id,
                        "FAILED",
                        Some(&format!("Checksum read error: {e}")),
                    )?;
                    items_report.push(ExecutionItemReport {
                        source_path: m.source_path.clone(),
                        destination_path: dest_path.to_string_lossy().to_string(),
                        status: "FAILED".to_string(),
                        error_message: Some(format!("Checksum read error: {e}")),
                    });
                    failed += 1;
                    continue;
                }
            };

            let dest_size = match fs::metadata(&dest_path) {
                Ok(meta) => meta.len(),
                Err(_) => 0,
            };

            if dest_hash != m.file_hash || dest_size != m.file_size {
                // Abort! Remove incomplete/corrupt destination copy; leave source UNTOUCHED!
                let _ = fs::remove_file(&dest_path);
                let err_msg = format!(
                    "Checksum mismatch! Expected {}, got {}. Source preserved intact.",
                    m.file_hash, dest_hash
                );
                storage.update_undo_operation_status(op_id, "FAILED", Some(&err_msg))?;
                items_report.push(ExecutionItemReport {
                    source_path: m.source_path.clone(),
                    destination_path: dest_path.to_string_lossy().to_string(),
                    status: "FAILED".to_string(),
                    error_message: Some(err_msg),
                });
                failed += 1;
                continue;
            }

            // Verified 100%! Remove source file safely
            if let Err(e) = fs::remove_file(&src_path) {
                let err_msg = format!("File copied and verified, but source removal failed: {e}");
                storage.update_undo_operation_status(op_id, "FAILED", Some(&err_msg))?;
                items_report.push(ExecutionItemReport {
                    source_path: m.source_path.clone(),
                    destination_path: dest_path.to_string_lossy().to_string(),
                    status: "FAILED".to_string(),
                    error_message: Some(err_msg),
                });
                failed += 1;
                continue;
            }

            storage.update_undo_operation_status(op_id, "COMPLETED", None)?;
            items_report.push(ExecutionItemReport {
                source_path: m.source_path.clone(),
                destination_path: dest_path.to_string_lossy().to_string(),
                status: "COMPLETED".to_string(),
                error_message: None,
            });
            successful += 1;
        }

        Ok(ExecutionReport {
            batch_id,
            total_planned: plan.moves.len(),
            successful,
            failed,
            items: items_report,
        })
    }

    /// Reverses an organization batch: copies all files from destination back to their exact original source paths,
    /// verifies checksum, removes destination copy, and updates status to UNDONE.
    pub fn undo_batch(storage: &Storage, batch_id: &str) -> Result<UndoReport, OrganizerError> {
        let ops = storage.get_undo_operations_by_batch(batch_id)?;
        let mut restored = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for op in &ops {
            if op.status != "COMPLETED" {
                continue;
            }

            let op_id = op.id.unwrap_or(0);
            let dest_path = PathBuf::from(&op.destination_path);
            let src_path = PathBuf::from(&op.source_path);

            if !dest_path.exists() {
                let msg = format!(
                    "Cannot undo: file in target does not exist: {}",
                    op.destination_path
                );
                errors.push(msg.clone());
                storage.update_undo_operation_status(op_id, "FAILED", Some(&msg))?;
                failed += 1;
                continue;
            }

            // Create original source parent directory if needed
            if let Some(src_parent) = src_path.parent() {
                if !src_parent.exists() {
                    let _ = fs::create_dir_all(src_parent);
                }
            }

            // Copy file back to exact original path
            if let Err(e) = fs::copy(&dest_path, &src_path) {
                let msg = format!("Cannot copy back to original path: {e}");
                errors.push(msg.clone());
                storage.update_undo_operation_status(op_id, "FAILED", Some(&msg))?;
                failed += 1;
                continue;
            }

            // Verify checksum of restored file
            let restored_hash = match compute_file_sha256(&src_path) {
                Ok(h) => h,
                Err(e) => {
                    let msg = format!("Cannot verify restored file checksum: {e}");
                    errors.push(msg.clone());
                    storage.update_undo_operation_status(op_id, "FAILED", Some(&msg))?;
                    failed += 1;
                    continue;
                }
            };

            if restored_hash != op.file_hash {
                let msg = format!("Restored file checksum mismatch: expected {}", op.file_hash);
                errors.push(msg.clone());
                let _ = fs::remove_file(&src_path);
                storage.update_undo_operation_status(op_id, "FAILED", Some(&msg))?;
                failed += 1;
                continue;
            }

            // Remove file from organized target folder only after verified restore
            let _ = fs::remove_file(&dest_path);
            storage.update_undo_operation_status(op_id, "UNDONE", None)?;
            restored += 1;
        }

        Ok(UndoReport {
            batch_id: batch_id.to_string(),
            total_operations: ops.len(),
            restored,
            failed,
            errors,
        })
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn make_test_file(path: &Path, content: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = File::create(path).unwrap();
        f.write_all(content).unwrap();
    }

    // ── Required Stage 4 Test #1 ──────────────────────────────────────────
    #[test]
    fn test_dry_run_zero_filesystem_mutations() {
        let temp_dir = tempdir().unwrap();
        let src = temp_dir.path().join("Desktop_Source");
        fs::create_dir_all(&src).unwrap();

        let target_pdf = temp_dir.path().join("Organized_PDFs");
        let target_img = temp_dir.path().join("Organized_Images");

        // Create 3 files in source folder
        let f1 = src.join("مستند_الميزانية_2026.pdf");
        let f2 = src.join("شعار_رَوَاق.png");
        let f3 = src.join("notes.txt");
        let sub = src.join("subfolder_مجلد_فرعي");
        fs::create_dir_all(&sub).unwrap();

        make_test_file(&f1, b"PDF Content 12345678");
        make_test_file(&f2, b"PNG Image Content 87654321");
        make_test_file(&f3, b"Text Notes");

        let hash1_before = compute_file_sha256(&f1).unwrap();
        let hash2_before = compute_file_sha256(&f2).unwrap();
        let hash3_before = compute_file_sha256(&f3).unwrap();
        let mod1_before = fs::metadata(&f1).unwrap().modified().unwrap();
        let mod2_before = fs::metadata(&f2).unwrap().modified().unwrap();
        let mod3_before = fs::metadata(&f3).unwrap().modified().unwrap();

        let rules = vec![
            OrganizeRule {
                id: Some(1),
                name: "قاعدة المستندات".to_string(),
                extensions: "pdf,doc,docx".to_string(),
                target_folder: target_pdf.to_string_lossy().to_string(),
                is_enabled: true,
                created_at: None,
            },
            OrganizeRule {
                id: Some(2),
                name: "قاعدة الصور".to_string(),
                extensions: "png,jpg,svg".to_string(),
                target_folder: target_img.to_string_lossy().to_string(),
                is_enabled: true,
                created_at: None,
            },
        ];

        // 1. Run Dry-Run Preview calculation
        let plan = SafeOrganizerEngine::compute_dry_run(&src, &rules)
            .expect("Dry run preview should succeed");

        // 2. Verify plan contents
        assert_eq!(plan.moves.len(), 2); // f1 and f2 match rules
        assert_eq!(plan.skipped.len(), 2); // f3 (no rule) and subfolder (directory)

        let move_pdf = plan
            .moves
            .iter()
            .find(|m| m.file_name == "مستند_الميزانية_2026.pdf")
            .unwrap();
        assert_eq!(move_pdf.rule_name, "قاعدة المستندات");
        assert!(!move_pdf.collision_detected);

        // 3. CRITICAL VERIFICATION:
        // Zero mutations on the filesystem during dry-run:
        // - No destination folder was created
        // - All source files and subfolder still exist
        // - Hashes and timestamps are 100% untouched
        assert!(
            !target_pdf.exists(),
            "Dry-run must never create target folders"
        );
        assert!(
            !target_img.exists(),
            "Dry-run must never create target folders"
        );

        assert!(f1.exists());
        assert!(f2.exists());
        assert!(f3.exists());
        assert!(sub.exists());

        assert_eq!(compute_file_sha256(&f1).unwrap(), hash1_before);
        assert_eq!(compute_file_sha256(&f2).unwrap(), hash2_before);
        assert_eq!(compute_file_sha256(&f3).unwrap(), hash3_before);

        assert_eq!(fs::metadata(&f1).unwrap().modified().unwrap(), mod1_before);
        assert_eq!(fs::metadata(&f2).unwrap().modified().unwrap(), mod2_before);
        assert_eq!(fs::metadata(&f3).unwrap().modified().unwrap(), mod3_before);
    }

    // ── Required Stage 4 Test #2 ──────────────────────────────────────────
    #[test]
    fn test_collision_never_overwrites_existing_file() {
        let temp_dir = tempdir().unwrap();
        let src = temp_dir.path().join("Desktop_Source");
        let dest = temp_dir.path().join("Organized_Docs");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dest).unwrap();

        // 1. Existing file in destination folder
        let existing_dest_file = dest.join("تقرير_المشروع.docx");
        let original_dest_content = b"ORIGINAL IMPORTANT DESTINATION CONTENT DO NOT OVERWRITE";
        make_test_file(&existing_dest_file, original_dest_content);
        let dest_hash_before = compute_file_sha256(&existing_dest_file).unwrap();

        // 2. Incoming file in source folder with the EXACT SAME NAME but different content
        let incoming_src_file = src.join("تقرير_المشروع.docx");
        let incoming_src_content = b"NEW INCOMING REPORT FROM DESKTOP";
        make_test_file(&incoming_src_file, incoming_src_content);
        let src_hash = compute_file_sha256(&incoming_src_file).unwrap();

        let rules = vec![OrganizeRule {
            id: Some(1),
            name: "Word Documents".to_string(),
            extensions: "docx".to_string(),
            target_folder: dest.to_string_lossy().to_string(),
            is_enabled: true,
            created_at: None,
        }];

        // Compute dry run
        let plan = SafeOrganizerEngine::compute_dry_run(&src, &rules).unwrap();
        assert_eq!(plan.moves.len(), 1);
        let planned_move = &plan.moves[0];

        // Collision MUST be detected
        assert!(planned_move.collision_detected);
        assert_eq!(
            Path::new(&planned_move.resolved_destination_path)
                .file_name()
                .unwrap(),
            "تقرير_المشروع (1).docx"
        );

        // Execute plan
        let storage = Storage::open_in_memory().unwrap();
        let report = SafeOrganizerEngine::execute_plan(&storage, &plan, None).unwrap();
        assert_eq!(report.successful, 1);
        assert_eq!(report.failed, 0);

        // 3. CRITICAL VERIFICATION:
        // - Original destination file MUST BE 100% UNTOUCHED (same content, same hash)!
        // - The incoming file was copied to safe resolved name: "تقرير_المشروع (1).docx"
        let dest_file_after_content = fs::read(&existing_dest_file).unwrap();
        assert_eq!(
            dest_file_after_content, original_dest_content,
            "Existing file MUST NOT be overwritten!"
        );
        assert_eq!(
            compute_file_sha256(&existing_dest_file).unwrap(),
            dest_hash_before
        );

        let resolved_file = dest.join("تقرير_المشروع (1).docx");
        assert!(resolved_file.exists(), "Resolved safe name must exist");
        assert_eq!(compute_file_sha256(&resolved_file).unwrap(), src_hash);
        assert!(
            !incoming_src_file.exists(),
            "Source file was safely removed after verified copy"
        );
    }

    // ── Required Stage 4 Test #3 ──────────────────────────────────────────
    #[test]
    fn test_checksum_mismatch_aborts_and_preserves_source() {
        let temp_dir = tempdir().unwrap();
        let src = temp_dir.path().join("Desktop_Source");
        let dest = temp_dir.path().join("Organized_PDFs");
        fs::create_dir_all(&src).unwrap();

        let src_file = src.join("ملف_هام_وحساس.pdf");
        let original_content = b"PRECIOUS USER DATA NEVER DELETE IF CHECKSUM FAILS";
        make_test_file(&src_file, original_content);
        let src_hash = compute_file_sha256(&src_file).unwrap();

        let rules = vec![OrganizeRule {
            id: Some(1),
            name: "PDFs".to_string(),
            extensions: "pdf".to_string(),
            target_folder: dest.to_string_lossy().to_string(),
            is_enabled: true,
            created_at: None,
        }];

        let plan = SafeOrganizerEngine::compute_dry_run(&src, &rules).unwrap();
        assert_eq!(plan.moves.len(), 1);

        // Simulate fault injection: destination copy is corrupted / bit-rotted
        let storage = Storage::open_in_memory().unwrap();
        let fault = FaultInjection::CorruptDestinationChecksum("ملف_هام_وحساس.pdf".to_string());

        let report = SafeOrganizerEngine::execute_plan(&storage, &plan, Some(fault)).unwrap();
        assert_eq!(report.failed, 1);
        assert_eq!(report.successful, 0);

        // 1. Destination copy must have been safely deleted upon detecting checksum mismatch
        let corrupt_dest_file = dest.join("ملف_هام_وحساس.pdf");
        assert!(
            !corrupt_dest_file.exists(),
            "Corrupted destination copy must be deleted"
        );

        // 2. CRITICAL VERIFICATION:
        // Original source file MUST be preserved completely intact on disk!
        assert!(
            src_file.exists(),
            "Source file MUST BE PRESERVED when checksum fails!"
        );
        assert_eq!(fs::read(&src_file).unwrap(), original_content);
        assert_eq!(compute_file_sha256(&src_file).unwrap(), src_hash);

        // 3. Verify undo log records FAILED with clear reason
        let ops = storage
            .get_undo_operations_by_batch(&report.batch_id)
            .unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].status, "FAILED");
        assert!(ops[0]
            .error_message
            .as_ref()
            .unwrap()
            .contains("Checksum mismatch"));
    }

    // ── Required Stage 4 Test #4 ──────────────────────────────────────────
    #[test]
    fn test_locked_or_permission_denied_file_reported_clearly() {
        let temp_dir = tempdir().unwrap();
        let src = temp_dir.path().join("Desktop_Source");
        let dest = temp_dir.path().join("Organized_Files");
        fs::create_dir_all(&src).unwrap();

        let f_normal1 = src.join("ملف_سليم_1.txt");
        let f_unreadable = src.join("ملف_محذوف_قبل_النقل.txt");
        let f_normal2 = src.join("ملف_سليم_2.txt");

        make_test_file(&f_normal1, b"Normal 1 Content");
        make_test_file(&f_unreadable, b"Unreadable content");
        make_test_file(&f_normal2, b"Normal 2 Content");

        let rules = vec![OrganizeRule {
            id: Some(1),
            name: "Text Files".to_string(),
            extensions: "txt".to_string(),
            target_folder: dest.to_string_lossy().to_string(),
            is_enabled: true,
            created_at: None,
        }];

        let plan = SafeOrganizerEngine::compute_dry_run(&src, &rules).unwrap();
        assert_eq!(plan.moves.len(), 3);

        // Simulate file lock / deletion / permission denial by deleting the file right after dry-run
        fs::remove_file(&f_unreadable).unwrap();

        let storage = Storage::open_in_memory().unwrap();
        let report = SafeOrganizerEngine::execute_plan(&storage, &plan, None).unwrap();

        // The engine reports the error clearly for the troubled file, but STILL SUCCEEDS FOR THE OTHER TWO!
        assert_eq!(report.successful, 2);
        assert_eq!(report.failed, 1);

        let failed_item = report
            .items
            .iter()
            .find(|i| i.source_path.contains("ملف_محذوف_قبل_النقل.txt"))
            .unwrap();
        assert_eq!(failed_item.status, "FAILED");
        assert!(failed_item.error_message.is_some());

        // Verify the two normal files were transferred safely
        assert!(dest.join("ملف_سليم_1.txt").exists());
        assert!(dest.join("ملف_سليم_2.txt").exists());
        assert!(!f_normal1.exists());
        assert!(!f_normal2.exists());
    }

    // ── Required Stage 4 Test #5 ──────────────────────────────────────────
    #[test]
    fn test_undo_batch_restores_exact_original_paths() {
        let temp_dir = tempdir().unwrap();
        let src = temp_dir.path().join("Desktop_Source");
        let dest = temp_dir.path().join("Organized_Documents");
        fs::create_dir_all(&src).unwrap();

        // 1. Create 20 distinct files combining Arabic, English, numbers, and symbols
        let mut original_files = Vec::new();
        let mut original_hashes = Vec::new();

        for i in 1..=20 {
            let name = match i % 4 {
                0 => format!("تقرير_العمل_النهائي_{i}.pdf"),
                1 => format!("Rawaq_Project_Document_{i}.pdf"),
                2 => format!("ملاحظات_Notes_مختلطة({i}).pdf"),
                _ => format!("سجل_حسابات_2026_{i} 📊.pdf"),
            };

            let file_path = src.join(&name);
            let content =
                format!("Unique content for test file #{i} in Rawaq safe organizer test عربي {i}");
            make_test_file(&file_path, content.as_bytes());

            let hash = compute_file_sha256(&file_path).unwrap();
            original_files.push(file_path);
            original_hashes.push(hash);
        }

        let rules = vec![OrganizeRule {
            id: Some(1),
            name: "PDF Organization Rule".to_string(),
            extensions: "pdf".to_string(),
            target_folder: dest.to_string_lossy().to_string(),
            is_enabled: true,
            created_at: None,
        }];

        // Compute dry run and execute
        let plan = SafeOrganizerEngine::compute_dry_run(&src, &rules).unwrap();
        assert_eq!(plan.moves.len(), 20);

        let storage = Storage::open_in_memory().unwrap();
        let exec_report = SafeOrganizerEngine::execute_plan(&storage, &plan, None).unwrap();
        assert_eq!(exec_report.successful, 20);
        assert_eq!(exec_report.failed, 0);

        // Verify all 20 files are in destination and gone from source
        for (idx, src_p) in original_files.iter().enumerate() {
            assert!(
                !src_p.exists(),
                "Source file #{idx} should be removed after organize"
            );
            let file_name = src_p.file_name().unwrap();
            let dest_p = dest.join(file_name);
            assert!(dest_p.exists(), "Destination file #{idx} must exist");
            assert_eq!(compute_file_sha256(&dest_p).unwrap(), original_hashes[idx]);
        }

        // 2. TRIGGER ONE-CLICK UNDO FOR THE BATCH!
        let undo_report = SafeOrganizerEngine::undo_batch(&storage, &exec_report.batch_id).unwrap();
        assert_eq!(undo_report.restored, 20);
        assert_eq!(undo_report.failed, 0);
        assert!(undo_report.errors.is_empty());

        // 3. CRITICAL VERIFICATION:
        // Every single one of the 20 files MUST be restored to its exact original source path,
        // with the EXACT original content and SHA-256 hash!
        // And must be removed from the destination folder.
        for (idx, src_p) in original_files.iter().enumerate() {
            assert!(
                src_p.exists(),
                "Restored source file #{idx} must exist in original path!"
            );
            assert_eq!(
                compute_file_sha256(src_p).unwrap(),
                original_hashes[idx],
                "Restored file #{idx} hash must match original 100%!"
            );

            let file_name = src_p.file_name().unwrap();
            let dest_p = dest.join(file_name);
            assert!(
                !dest_p.exists(),
                "Destination file #{idx} must be removed after undo"
            );
        }

        // Verify undo operations status in SQLite updated to "UNDONE"
        let ops = storage
            .get_undo_operations_by_batch(&exec_report.batch_id)
            .unwrap();
        assert_eq!(ops.len(), 20);
        for op in &ops {
            assert_eq!(op.status, "UNDONE");
        }
    }
}
