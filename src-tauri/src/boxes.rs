use crate::storage::{BoxGroup, DesktopBox, Storage, StorageError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BoxError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid box configuration: {0}")]
    InvalidConfig(String),
}

/// A file or folder item scanned inside a mapped desktop box folder.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoxItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub modified_timestamp: u64,
    pub extension: Option<String>,
}

/// The current accessibility state of a box's mapped folder.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", content = "payload")]
pub enum FolderState {
    /// Folder is accessible on disk and contains items.
    Accessible { items: Vec<BoxItem> },
    /// Folder is missing, deleted, or disconnected (e.g., unplugged USB drive or unmounted network disk).
    /// The box remains safe in the database and shows a graceful disconnected banner.
    MissingOrDisconnected { path: String, message: String },
    /// Folder exists but lacks read permissions.
    PermissionDenied { path: String, message: String },
}

/// Service managing virtual folder mapping, graceful disconnection detection,
/// content sorting, and group merging/dissolving without ever touching or modifying disk files.
pub struct BoxService;

impl BoxService {
    /// Maps a directory into a new DesktopBox in the database.
    /// **Guarantee**: This operation only reads directory presence and records the path in SQLite;
    /// it NEVER modifies, renames, moves, or deletes any file or folder on the user's disk.
    pub fn create_box(
        storage: &Storage,
        name: String,
        folder_path: String,
        display_mode: Option<String>,
        sort_order: Option<String>,
    ) -> Result<DesktopBox, BoxError> {
        let b = DesktopBox {
            id: None,
            name,
            folder_path,
            display_mode: display_mode.unwrap_or_else(|| "grid".to_string()),
            is_capsule_collapsed: false,
            sort_order: sort_order.unwrap_or_else(|| "name".to_string()),
            position_x: 100.0,
            position_y: 100.0,
            width: 320.0,
            height: 400.0,
            display_id: None,
            group_id: None,
            created_at: None,
            updated_at: None,
        };

        let box_id = storage.insert_desktop_box(&b)?;
        let saved_box = storage.get_desktop_box(box_id)?;
        Ok(saved_box)
    }

    /// Updates box visual settings or metadata in SQLite.
    /// **Guarantee**: Never modifies the mapped folder or files on disk.
    pub fn update_box(storage: &Storage, b: &DesktopBox) -> Result<(), BoxError> {
        storage.update_desktop_box(b)?;
        Ok(())
    }

    /// Deletes a box record from the application database.
    /// **Guarantee**: Only removes the record from SQLite. The underlying directory and
    /// ALL its files on the user's hard drive remain 100% untouched and intact.
    pub fn delete_box(storage: &Storage, box_id: i64) -> Result<(), BoxError> {
        storage.delete_desktop_box(box_id)?;
        Ok(())
    }

    /// Scans the contents of a mapped folder safely and returns its `FolderState`.
    /// If the folder does not exist or was unplugged, returns `FolderState::MissingOrDisconnected`
    /// without panicking, crashing, or modifying database records.
    pub fn scan_folder_contents(folder_path: &str, sort_order: &str) -> FolderState {
        let p = Path::new(folder_path);

        // 1. Check path existence and verify it is a directory
        if !p.exists() || !p.is_dir() {
            return FolderState::MissingOrDisconnected {
                path: folder_path.to_string(),
                message: "المجلد غير متاح مؤقتًا أو تم فصل القرص (Folder is temporarily disconnected or missing)".to_string(),
            };
        }

        // 2. Attempt to read directory entries safely
        let read_dir = match fs::read_dir(p) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                return FolderState::PermissionDenied {
                    path: folder_path.to_string(),
                    message: "تم رفض الصلاحية للوصول إلى هذا المجلد (Permission denied)".to_string(),
                };
            }
            Err(_) => {
                return FolderState::MissingOrDisconnected {
                    path: folder_path.to_string(),
                    message: "تعذر قراءة محتويات المجلد المربوط (Unable to read mapped folder)"
                        .to_string(),
                };
            }
        };

        let mut items = Vec::new();

        for entry_res in read_dir {
            let Ok(entry) = entry_res else { continue };
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            let is_dir = metadata.is_dir();
            let size_bytes = if is_dir { 0 } else { metadata.len() };

            let modified_timestamp = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let extension = if is_dir {
                None
            } else {
                entry_path
                    .extension()
                    .map(|ext| ext.to_string_lossy().to_string().to_lowercase())
            };

            items.push(BoxItem {
                name,
                path: entry_path.to_string_lossy().to_string(),
                is_dir,
                size_bytes,
                modified_timestamp,
                extension,
            });
        }

        // 3. Sort items according to the requested sort_order
        Self::sort_items(&mut items, sort_order);

        FolderState::Accessible { items }
    }

    /// Sorts a vector of BoxItems in-place according to specified strategy.
    pub fn sort_items(items: &mut [BoxItem], sort_order: &str) {
        match sort_order {
            "type" => {
                // Directories first, then by extension, then by name
                items.sort_by(|a, b| {
                    if a.is_dir != b.is_dir {
                        return b.is_dir.cmp(&a.is_dir);
                    }
                    let ext_a = a.extension.as_deref().unwrap_or("");
                    let ext_b = b.extension.as_deref().unwrap_or("");
                    ext_a
                        .cmp(ext_b)
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                });
            }
            "date" => {
                // Newest first
                items.sort_by_key(|a| std::cmp::Reverse(a.modified_timestamp));
            }
            "size" => {
                // Largest first; directories grouped at bottom with 0 size
                items.sort_by(|a, b| {
                    b.size_bytes
                        .cmp(&a.size_bytes)
                        .then_with(|| a.name.cmp(&b.name))
                });
            }
            _ => {
                // "name" (default): Directories first, then alphabetical case-insensitive
                items.sort_by(|a, b| {
                    if a.is_dir != b.is_dir {
                        return b.is_dir.cmp(&a.is_dir);
                    }
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                });
            }
        }
    }

    /// Merges multiple boxes into a new or existing BoxGroup.
    pub fn merge_boxes_into_group(
        storage: &Storage,
        group_name: &str,
        box_ids: &[i64],
    ) -> Result<i64, BoxError> {
        if box_ids.is_empty() {
            return Err(BoxError::InvalidConfig(
                "Cannot create group with zero boxes".into(),
            ));
        }

        let group_id = storage.insert_box_group(&BoxGroup {
            id: None,
            name: group_name.to_string(),
            created_at: None,
        })?;

        for &id in box_ids {
            let mut b = storage.get_desktop_box(id)?;
            b.group_id = Some(group_id);
            storage.update_desktop_box(&b)?;
        }

        Ok(group_id)
    }

    /// Dissolves a BoxGroup, resetting all member boxes' `group_id` to NULL.
    /// **Guarantee**: All individual box settings, mapped folders, and disk files are preserved intact.
    pub fn dissolve_group(storage: &Storage, group_id: i64) -> Result<(), BoxError> {
        storage.delete_box_group(group_id)?;
        Ok(())
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn hash_file(path: &Path) -> String {
        let bytes = fs::read(path).expect("Failed to read file for hash");
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        format!("{:x}", hasher.finalize())
    }

    // ── Required Stage 3 Test #1 ──────────────────────────────────────────
    #[test]
    fn test_map_folder_never_modifies_or_deletes_disk_files() {
        let temp_dir = tempdir().expect("Failed to create tempdir");
        let folder_path = temp_dir.path().join("مشاريع_العمل_2026");
        fs::create_dir_all(&folder_path).unwrap();

        // Create sample files in folder (Arabic, English, nested)
        let file1_path = folder_path.join("تقرير_الميزانية_2026.pdf");
        let file2_path = folder_path.join("project_notes_v1.txt");
        let subfolder_path = folder_path.join("مرفقات");
        fs::create_dir_all(&subfolder_path).unwrap();

        {
            let mut f1 = File::create(&file1_path).unwrap();
            f1.write_all(b"Original PDF Content 12345 \xD8\xB9\xD8\xB1\xD8\xA8\xD9\x8A")
                .unwrap();
        }
        {
            let mut f2 = File::create(&file2_path).unwrap();
            f2.write_all(b"Clean text content for notes").unwrap();
        }

        let hash_before_1 = hash_file(&file1_path);
        let hash_before_2 = hash_file(&file2_path);
        let mod_time_before_1 = fs::metadata(&file1_path).unwrap().modified().unwrap();
        let mod_time_before_2 = fs::metadata(&file2_path).unwrap().modified().unwrap();

        // 1. Map folder in storage
        let storage = Storage::open_in_memory().unwrap();
        let box_entry = BoxService::create_box(
            &storage,
            "صندوق تقارير العمل".to_string(),
            folder_path.to_string_lossy().to_string(),
            Some("grid".to_string()),
            Some("name".to_string()),
        )
        .expect("Failed to map box");

        let box_id = box_entry.id.unwrap();

        // 2. Scan contents and verify items
        let state = BoxService::scan_folder_contents(&box_entry.folder_path, &box_entry.sort_order);
        match state {
            FolderState::Accessible { items } => {
                assert_eq!(items.len(), 3); // 2 files + 1 subfolder
            }
            _ => panic!("Expected FolderState::Accessible"),
        }

        // 3. Modify box properties repeatedly (name, position, sort order, capsule state)
        let mut modified_box = storage.get_desktop_box(box_id).unwrap();
        modified_box.name = "اسم معدل بالكامل".to_string();
        modified_box.position_x = 550.0;
        modified_box.position_y = 320.0;
        modified_box.display_mode = "list".to_string();
        modified_box.is_capsule_collapsed = true;
        modified_box.sort_order = "date".to_string();
        BoxService::update_box(&storage, &modified_box).expect("Update box should succeed");

        // 4. Delete the box from the application
        BoxService::delete_box(&storage, box_id).expect("Delete box should succeed");
        assert!(storage.get_desktop_box(box_id).is_err());

        // 5. CRITICAL VERIFICATION:
        // The mapped directory and all its files on disk MUST be 100% untouched:
        // same presence, same hashes, same modification timestamps.
        assert!(folder_path.exists(), "Folder on disk must not be deleted");
        assert!(file1_path.exists(), "File 1 on disk must not be deleted");
        assert!(file2_path.exists(), "File 2 on disk must not be deleted");
        assert!(
            subfolder_path.exists(),
            "Subfolder on disk must not be deleted"
        );

        assert_eq!(
            hash_file(&file1_path),
            hash_before_1,
            "File 1 content must not be modified"
        );
        assert_eq!(
            hash_file(&file2_path),
            hash_before_2,
            "File 2 content must not be modified"
        );

        let mod_time_after_1 = fs::metadata(&file1_path).unwrap().modified().unwrap();
        let mod_time_after_2 = fs::metadata(&file2_path).unwrap().modified().unwrap();
        assert_eq!(
            mod_time_before_1, mod_time_after_1,
            "File 1 modified timestamp must not change"
        );
        assert_eq!(
            mod_time_before_2, mod_time_after_2,
            "File 2 modified timestamp must not change"
        );
    }

    // ── Required Stage 3 Test #2 ──────────────────────────────────────────
    #[test]
    fn test_missing_or_disconnected_folder_graceful_state() {
        let temp_dir = tempdir().expect("Failed to create tempdir");
        let removable_disk_folder = temp_dir.path().join("USB_DRIVE_DISCONNECTable");
        fs::create_dir_all(&removable_disk_folder).unwrap();

        let test_file = removable_disk_folder.join("external_data.txt");
        fs::write(&test_file, b"data on external disk").unwrap();

        let storage = Storage::open_in_memory().unwrap();
        let box_entry = BoxService::create_box(
            &storage,
            "صندوق القرص الخارجي".to_string(),
            removable_disk_folder.to_string_lossy().to_string(),
            None,
            None,
        )
        .unwrap();

        let box_id = box_entry.id.unwrap();

        // 1. When folder is connected -> Accessible
        let state1 =
            BoxService::scan_folder_contents(&box_entry.folder_path, &box_entry.sort_order);
        assert!(matches!(state1, FolderState::Accessible { .. }));

        // 2. Simulate unplugging USB / deleting / renaming the external directory
        fs::remove_dir_all(&removable_disk_folder).unwrap();
        assert!(!removable_disk_folder.exists());

        // 3. Scan contents again -> MUST return graceful MissingOrDisconnected state without crashing
        let state2 =
            BoxService::scan_folder_contents(&box_entry.folder_path, &box_entry.sort_order);
        match state2 {
            FolderState::MissingOrDisconnected { path, message } => {
                assert_eq!(path, box_entry.folder_path);
                assert!(!message.is_empty());
                assert!(message.contains("المجلد غير متاح مؤقتًا"));
            }
            other => panic!("Expected MissingOrDisconnected, got: {other:?}"),
        }

        // 4. Verify box in database WAS NOT DELETED
        let preserved_box = storage
            .get_desktop_box(box_id)
            .expect("Box must remain in SQLite");
        assert_eq!(preserved_box.name, "صندوق القرص الخارجي");
        assert_eq!(preserved_box.folder_path, box_entry.folder_path);

        // 5. Simulate reconnecting the folder
        fs::create_dir_all(&removable_disk_folder).unwrap();
        fs::write(&test_file, b"restored external data").unwrap();

        let state3 =
            BoxService::scan_folder_contents(&box_entry.folder_path, &box_entry.sort_order);
        match state3 {
            FolderState::Accessible { items } => {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].name, "external_data.txt");
            }
            other => panic!("Expected Accessible after reconnection, got: {other:?}"),
        }
    }

    // ── Required Stage 3 Test #3 ──────────────────────────────────────────
    #[test]
    fn test_group_merge_and_dissolve_preserves_members() {
        let storage = Storage::open_in_memory().unwrap();

        // Create 3 distinct boxes with individual settings
        let box1 = BoxService::create_box(
            &storage,
            "المستندات".to_string(),
            "/home/user/Documents".to_string(),
            Some("grid".to_string()),
            Some("name".to_string()),
        )
        .unwrap();

        let box2 = BoxService::create_box(
            &storage,
            "الصور والتصاميم".to_string(),
            "/home/user/Pictures".to_string(),
            Some("list".to_string()),
            Some("date".to_string()),
        )
        .unwrap();

        let box3 = BoxService::create_box(
            &storage,
            "Projects & Code 🚀".to_string(),
            "/home/user/Projects".to_string(),
            Some("grid".to_string()),
            Some("size".to_string()),
        )
        .unwrap();

        let id1 = box1.id.unwrap();
        let id2 = box2.id.unwrap();
        let id3 = box3.id.unwrap();

        assert_eq!(box1.group_id, None);
        assert_eq!(box2.group_id, None);
        assert_eq!(box3.group_id, None);

        // 1. Merge the 3 boxes into a tabbed BoxGroup
        let group_id =
            BoxService::merge_boxes_into_group(&storage, "مجموعة العمل الرئيسية", &[id1, id2, id3])
                .expect("Failed to merge boxes into group");

        let group = storage.get_box_group(group_id).unwrap();
        assert_eq!(group.name, "مجموعة العمل الرئيسية");

        // Verify each box is assigned to the group while keeping individual configs
        let b1 = storage.get_desktop_box(id1).unwrap();
        let b2 = storage.get_desktop_box(id2).unwrap();
        let b3 = storage.get_desktop_box(id3).unwrap();

        assert_eq!(b1.group_id, Some(group_id));
        assert_eq!(b2.group_id, Some(group_id));
        assert_eq!(b3.group_id, Some(group_id));

        assert_eq!(b1.display_mode, "grid");
        assert_eq!(b2.display_mode, "list");
        assert_eq!(b3.sort_order, "size");

        let group_members = storage.list_boxes_by_group(group_id).unwrap();
        assert_eq!(group_members.len(), 3);

        // 2. Dissolve the group
        BoxService::dissolve_group(&storage, group_id).expect("Failed to dissolve group");

        // Group must be deleted
        assert!(storage.get_box_group(group_id).is_err());

        // ALL 3 boxes must remain with their group_id reset to None and all settings intact
        let b1_after = storage.get_desktop_box(id1).unwrap();
        let b2_after = storage.get_desktop_box(id2).unwrap();
        let b3_after = storage.get_desktop_box(id3).unwrap();

        assert_eq!(b1_after.group_id, None);
        assert_eq!(b2_after.group_id, None);
        assert_eq!(b3_after.group_id, None);

        assert_eq!(b1_after.name, "المستندات");
        assert_eq!(b1_after.folder_path, "/home/user/Documents");
        assert_eq!(b1_after.display_mode, "grid");

        assert_eq!(b2_after.name, "الصور والتصاميم");
        assert_eq!(b2_after.folder_path, "/home/user/Pictures");
        assert_eq!(b2_after.display_mode, "list");

        assert_eq!(b3_after.name, "Projects & Code 🚀");
        assert_eq!(b3_after.folder_path, "/home/user/Projects");
        assert_eq!(b3_after.sort_order, "size");
    }

    // ── Sorting edge cases ────────────────────────────────────────────────
    #[test]
    fn test_sorting_arabic_english_and_directories() {
        let mut items = vec![
            BoxItem {
                name: "ملف_ب.txt".into(),
                path: "/p/ملف_ب.txt".into(),
                is_dir: false,
                size_bytes: 500,
                modified_timestamp: 100,
                extension: Some("txt".into()),
            },
            BoxItem {
                name: "مجلد_العمل".into(),
                path: "/p/مجلد_العمل".into(),
                is_dir: true,
                size_bytes: 0,
                modified_timestamp: 200,
                extension: None,
            },
            BoxItem {
                name: "ملف_أ.pdf".into(),
                path: "/p/ملف_أ.pdf".into(),
                is_dir: false,
                size_bytes: 1200,
                modified_timestamp: 50,
                extension: Some("pdf".into()),
            },
            BoxItem {
                name: "archive.zip".into(),
                path: "/p/archive.zip".into(),
                is_dir: false,
                size_bytes: 8000,
                modified_timestamp: 300,
                extension: Some("zip".into()),
            },
        ];

        // Sort by name: directories first, then alphabetical
        BoxService::sort_items(&mut items, "name");
        assert!(items[0].is_dir); // مجلد_العمل first
        assert_eq!(items[0].name, "مجلد_العمل");

        // Sort by size: largest first
        BoxService::sort_items(&mut items, "size");
        assert_eq!(items[0].name, "archive.zip");
        assert_eq!(items[0].size_bytes, 8000);

        // Sort by date: newest first
        BoxService::sort_items(&mut items, "date");
        assert_eq!(items[0].name, "archive.zip"); // 300
        assert_eq!(items[1].name, "مجلد_العمل"); // 200
    }
}
