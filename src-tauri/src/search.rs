use crate::storage::Storage;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Kinds of items indexed and returned by the Unified Local Search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchResultKind {
    File,
    Task,
    Note,
}

/// A single unified search hit across files, tasks, or captured notes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResultItem {
    pub kind: SearchResultKind,
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub extra_info: Option<String>,
}

/// Envelope returned by search queries with performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub normalized_query: String,
    pub elapsed_ms: u128,
    pub results: Vec<SearchResultItem>,
}

/// Normalizes Arabic text for tolerant and resilient search:
/// - Unifies all Alef variants (أ, إ, آ, ٱ) -> ا
/// - Unifies Teh Marbuta (ة) -> ه
/// - Unifies Alef Maqsura (ى) -> ي
/// - Strips Tashkeel / diacritics (Fatha, Damma, Kasra, Shadda, Sukun, Tanween, etc.)
/// - Strips Tatweel / Kashida (ـ)
/// - Lowercases English/Latin characters
pub fn normalize_arabic_text(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            // Alef variations -> Plain Alef
            'أ' | 'إ' | 'آ' | 'ٱ' => normalized.push('ا'),
            // Teh Marbuta -> Heh
            'ة' => normalized.push('ه'),
            // Alef Maqsura -> Ya
            'ى' => normalized.push('ي'),
            // Tashkeel / Arabic diacritics removal
            '\u{064B}'  // Fathatan
            | '\u{064C}'  // Dammatan
            | '\u{064D}'  // Kasratan
            | '\u{064E}'  // Fatha
            | '\u{064F}'  // Damma
            | '\u{0650}'  // Kasra
            | '\u{0651}'  // Shadda
            | '\u{0652}'  // Sukun
            | '\u{0670}'  // Superscript Alef
            | '\u{0640}'  // Tatweel / Kashida
            => {},
            // Lowercase standard characters
            other => {
                for lower in other.to_lowercase() {
                    normalized.push(lower);
                }
            }
        }
    }
    normalized
}

/// A lightweight mock/cached file reference for search indexing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchableFile {
    pub path: String,
    pub name: String,
}

/// Unified local search engine executing across desktop box files, task items, and capture notes.
pub struct UnifiedSearchEngine;

impl UnifiedSearchEngine {
    /// Perform a high-speed search across files, tasks, and notes with Arabic normalization.
    pub fn search(storage: &Storage, raw_query: &str, files: &[SearchableFile]) -> SearchResponse {
        let start = Instant::now();
        let query_trimmed = raw_query.trim();

        if query_trimmed.is_empty() {
            return SearchResponse {
                query: String::new(),
                normalized_query: String::new(),
                elapsed_ms: start.elapsed().as_millis(),
                results: Vec::new(),
            };
        }

        let norm_query = normalize_arabic_text(query_trimmed);
        let query_terms: Vec<&str> = norm_query.split_whitespace().collect();
        let mut results = Vec::new();

        // 1. Search Files
        for file in files {
            let norm_name = normalize_arabic_text(&file.name);
            let norm_path = normalize_arabic_text(&file.path);
            let matches = query_terms
                .iter()
                .all(|t| norm_name.contains(t) || norm_path.contains(t));

            if matches {
                results.push(SearchResultItem {
                    kind: SearchResultKind::File,
                    id: file.path.clone(),
                    title: file.name.clone(),
                    subtitle: file.path.clone(),
                    extra_info: Some("ملف سطح المكتب".to_string()),
                });
            }
        }

        // 2. Search Tasks
        if let Ok(tasks) = storage.list_tasks(None) {
            for task in tasks {
                let norm_title = normalize_arabic_text(&task.title);
                let norm_desc = normalize_arabic_text(&task.description);
                let norm_tags = normalize_arabic_text(&task.tags);
                let matches = query_terms.iter().all(|t| {
                    norm_title.contains(t) || norm_desc.contains(t) || norm_tags.contains(t)
                });

                if matches {
                    results.push(SearchResultItem {
                        kind: SearchResultKind::Task,
                        id: task.id.unwrap_or(0).to_string(),
                        title: task.title,
                        subtitle: format!("مهمة ({})", task.period),
                        extra_info: Some(if task.is_completed {
                            "مكتملة".to_string()
                        } else {
                            "قيد الانتظار".to_string()
                        }),
                    });
                }
            }
        }

        // 3. Search Capture Notes
        if let Ok(notes) = storage.list_capture_notes(Some(200)) {
            for note in notes {
                let norm_content = normalize_arabic_text(&note.content);
                let norm_tags = normalize_arabic_text(&note.tags);
                let norm_url = note
                    .source_url
                    .as_deref()
                    .map(normalize_arabic_text)
                    .unwrap_or_default();

                let matches = query_terms.iter().all(|t| {
                    norm_content.contains(t) || norm_tags.contains(t) || norm_url.contains(t)
                });

                if matches {
                    results.push(SearchResultItem {
                        kind: SearchResultKind::Note,
                        id: note.id.unwrap_or(0).to_string(),
                        title: truncate_title(&note.content, 40),
                        subtitle: format!("ملاحظة ملتقطة ({})", note.note_type),
                        extra_info: note.source_url,
                    });
                }
            }
        }

        let elapsed = start.elapsed();
        SearchResponse {
            query: raw_query.to_string(),
            normalized_query: norm_query,
            elapsed_ms: elapsed.as_millis(),
            results,
        }
    }
}

fn truncate_title(s: &str, max_chars: usize) -> String {
    let trimmed = s.trim();
    if trimmed.chars().count() <= max_chars {
        trimmed.to_string()
    } else {
        let prefix: String = trimmed.chars().take(max_chars).collect();
        format!("{prefix}...")
    }
}

// ─── Automated Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::storage::{CaptureNote, Storage, TaskItem};

    /// Test 3: `search::test_unified_search_arabic_normalization_and_speed`
    /// Search for Arabic words (with normalization for Alef أ/إ/آ and Teh Marbuta ة/ه and diacritics)
    /// and English across files, notes, and tasks in under 50 milliseconds.
    #[test]
    fn test_unified_search_arabic_normalization_and_speed() {
        let storage = Storage::open_in_memory().expect("Should open test storage");

        // 1. Seed tasks with varying Arabic letters & diacritics
        storage
            .insert_task(&TaskItem {
                id: None,
                title: "إعداد تقرير الميزانية السنويّة".to_string(),
                description: "مراجعة الأرقام الماليّة مع الإدارة".to_string(),
                period: "morning".to_string(),
                is_completed: false,
                tags: "finance,ميزانية".to_string(),
                created_at: None,
                updated_at: None,
            })
            .unwrap();

        storage
            .insert_task(&TaskItem {
                id: None,
                title: "Build Tauri v2 Desktop Engine".to_string(),
                description: "Cross-platform verification on Windows and Linux".to_string(),
                period: "today".to_string(),
                is_completed: true,
                tags: "code,rust".to_string(),
                created_at: None,
                updated_at: None,
            })
            .unwrap();

        // 2. Seed capture notes
        storage
            .insert_capture_note(&CaptureNote {
                id: None,
                content: "مُلاَحَظَةٌ: رابط وثائق رَوَاق الرسمية".to_string(),
                note_type: "text".to_string(),
                tags: "docs".to_string(),
                source_url: Some("https://rawaq.app/docs".to_string()),
                created_at: None,
            })
            .unwrap();

        // 3. Seed files (including diacritics and different Alef forms)
        let mut files = Vec::new();
        for i in 1..=50 {
            files.push(SearchableFile {
                name: format!("مِلَفُّ_أَبْحَاث_{i}.pdf"),
                path: format!("C:/Desktop/مجلد/ملف_أبحاث_{i}.pdf"),
            });
        }
        files.push(SearchableFile {
            name: "Financial_Report_2026.xlsx".to_string(),
            path: "C:/Desktop/Financial_Report_2026.xlsx".to_string(),
        });

        // Test Query A: Searching with plain Alef and Heh "اعداد تقرير الميزانيه"
        // Target in DB has: "إعداد" (Alef Hamza Below) and "السنويّة" (Teh Marbuta with Shadda)
        let query_a = "اعداد تقرير الميزانيه";
        let resp_a = UnifiedSearchEngine::search(&storage, query_a, &files);

        assert!(
            resp_a.elapsed_ms < 50,
            "Search execution must be under 50ms (took {}ms)",
            resp_a.elapsed_ms
        );
        assert!(
            !resp_a.results.is_empty(),
            "Should match normalized Arabic task"
        );
        assert!(resp_a
            .results
            .iter()
            .any(|r| r.kind == SearchResultKind::Task && r.title.contains("تقرير الميزانية")));

        // Test Query B: Searching for "ملاحظه" (plain Heh)
        // Target in DB has: "مُلاَحَظَةٌ" (Teh Marbuta with Damma, Fatha, Tanween)
        let query_b = "ملاحظه";
        let resp_b = UnifiedSearchEngine::search(&storage, query_b, &files);
        assert!(
            resp_b.elapsed_ms < 50,
            "Search execution must be under 50ms (took {}ms)",
            resp_b.elapsed_ms
        );
        assert!(
            resp_b
                .results
                .iter()
                .any(|r| r.kind == SearchResultKind::Note),
            "Should find note despite Tashkeel and Teh Marbuta difference"
        );

        // Test Query C: Searching for "ابحاث" (plain Alef) across 50 seeded files with "أَبْحَاث"
        let query_c = "ابحاث";
        let resp_c = UnifiedSearchEngine::search(&storage, query_c, &files);
        assert!(
            resp_c.elapsed_ms < 50,
            "Search execution across 50+ files must be under 50ms (took {}ms)",
            resp_c.elapsed_ms
        );
        assert_eq!(
            resp_c.results.len(),
            50,
            "Should find all 50 files matching normalized Alef and stripped Tashkeel"
        );

        // Test Query D: Case-insensitive English search "tauri desktop"
        let query_d = "TAURI desktop";
        let resp_d = UnifiedSearchEngine::search(&storage, query_d, &files);
        assert!(
            resp_d.elapsed_ms < 50,
            "Search execution must be under 50ms (took {}ms)",
            resp_d.elapsed_ms
        );
        assert!(resp_d
            .results
            .iter()
            .any(|r| r.kind == SearchResultKind::Task && r.title.contains("Tauri")));
    }
}
