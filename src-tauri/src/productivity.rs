use crate::security::CommandGate;
use crate::storage::{CaptureNote, FocusSessionRecord, Storage, StorageError, TaskItem};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// ─── Daily Tasks Manager ──────────────────────────────────────────────────────

pub struct TaskManager;

impl TaskManager {
    /// Validates and inserts a daily task item.
    /// Handles edge cases: rejects empty titles, sanitizes HTML/script injection,
    /// enforces maximum safe title lengths, and trims whitespace.
    pub fn create_task(
        storage: &Storage,
        title: &str,
        description: Option<&str>,
        period: Option<&str>,
        tags: Option<&str>,
    ) -> Result<TaskItem, String> {
        let cleaned_title = CommandGate::sanitize_text_payload(title);
        if cleaned_title.is_empty() {
            return Err("عنوان المهمة لا يمكن أن يكون فارغًا".to_string());
        }

        if cleaned_title.chars().count() > 255 {
            return Err("عنوان المهمة يتجاوز الحد الأقصى المسموح به (255 حرفًا)".to_string());
        }

        let cleaned_desc = description
            .map(CommandGate::sanitize_text_payload)
            .unwrap_or_default();

        let valid_period = match period.unwrap_or("today") {
            "morning" => "morning",
            "evening" => "evening",
            _ => "today",
        };

        let cleaned_tags = tags
            .map(CommandGate::sanitize_text_payload)
            .unwrap_or_default();

        let task = TaskItem {
            id: None,
            title: cleaned_title,
            description: cleaned_desc,
            period: valid_period.to_string(),
            is_completed: false,
            tags: cleaned_tags,
            created_at: None,
            updated_at: None,
        };

        let new_id = storage.insert_task(&task).map_err(|e| e.to_string())?;
        storage.get_task(new_id).map_err(|e| e.to_string())
    }

    pub fn list_tasks(storage: &Storage, period: Option<&str>) -> Result<Vec<TaskItem>, String> {
        storage.list_tasks(period).map_err(|e| e.to_string())
    }

    pub fn toggle_task(storage: &Storage, id: i64) -> Result<bool, String> {
        let mut task = storage.get_task(id).map_err(|e| e.to_string())?;
        task.is_completed = !task.is_completed;
        storage.update_task(&task).map_err(|e| e.to_string())?;
        Ok(task.is_completed)
    }

    pub fn update_task(
        storage: &Storage,
        id: i64,
        title: &str,
        description: Option<&str>,
        period: Option<&str>,
        tags: Option<&str>,
    ) -> Result<TaskItem, String> {
        let cleaned_title = CommandGate::sanitize_text_payload(title);
        if cleaned_title.is_empty() {
            return Err("عنوان المهمة لا يمكن أن يكون فارغًا".to_string());
        }

        let cleaned_desc = description
            .map(CommandGate::sanitize_text_payload)
            .unwrap_or_default();

        let valid_period = match period.unwrap_or("today") {
            "morning" => "morning",
            "evening" => "evening",
            _ => "today",
        };

        let cleaned_tags = tags
            .map(CommandGate::sanitize_text_payload)
            .unwrap_or_default();

        let mut task = storage.get_task(id).map_err(|e| e.to_string())?;
        task.title = cleaned_title;
        task.description = cleaned_desc;
        task.period = valid_period.to_string();
        task.tags = cleaned_tags;

        storage.update_task(&task).map_err(|e| e.to_string())?;
        storage.get_task(id).map_err(|e| e.to_string())
    }

    pub fn delete_task(storage: &Storage, id: i64) -> Result<(), String> {
        storage.delete_task(id).map_err(|e| e.to_string())
    }
}

// ─── Quick Capture Notes Manager ──────────────────────────────────────────────

pub struct CaptureManager;

impl CaptureManager {
    /// Inserts a sanitized quick capture note or link.
    pub fn create_capture(
        storage: &Storage,
        content: &str,
        note_type: Option<&str>,
        tags: Option<&str>,
        source_url: Option<&str>,
    ) -> Result<CaptureNote, String> {
        let cleaned_content = CommandGate::sanitize_text_payload(content);
        if cleaned_content.is_empty() {
            return Err("محتوى الملاحظة لا يمكن أن يكون فارغًا".to_string());
        }

        let valid_type = match note_type.unwrap_or("text") {
            "link" => "link",
            "snippet" => "snippet",
            _ => "text",
        };

        let cleaned_url = source_url.and_then(|u| CommandGate::sanitize_clipboard_uri(u).ok());
        let cleaned_tags = tags
            .map(CommandGate::sanitize_text_payload)
            .unwrap_or_default();

        let note = CaptureNote {
            id: None,
            content: cleaned_content,
            note_type: valid_type.to_string(),
            tags: cleaned_tags,
            source_url: cleaned_url,
            created_at: None,
        };

        let id = storage
            .insert_capture_note(&note)
            .map_err(|e| e.to_string())?;
        storage.get_capture_note(id).map_err(|e| e.to_string())
    }

    pub fn list_captures(
        storage: &Storage,
        limit: Option<usize>,
    ) -> Result<Vec<CaptureNote>, String> {
        storage.list_capture_notes(limit).map_err(|e| e.to_string())
    }

    pub fn delete_capture(storage: &Storage, id: i64) -> Result<(), String> {
        storage.delete_capture_note(id).map_err(|e| e.to_string())
    }
}

// ─── Focus Timer / Pomodoro State Machine ─────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PomodoroState {
    Idle,
    Running,
    Paused,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PomodoroStatus {
    pub state: PomodoroState,
    pub task_id: Option<i64>,
    pub target_seconds: u64,
    pub elapsed_seconds: u64,
    pub remaining_seconds: u64,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct PomodoroTimer {
    inner: Arc<RwLock<PomodoroInner>>,
}

#[derive(Debug, Clone)]
struct PomodoroInner {
    session_id: Option<i64>,
    state: PomodoroState,
    task_id: Option<i64>,
    target_seconds: u64,
    elapsed_seconds: u64,
}

impl Default for PomodoroTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl PomodoroTimer {
    pub const DEFAULT_TARGET_SECONDS: u64 = 25 * 60; // 25 minutes

    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(PomodoroInner {
                session_id: None,
                state: PomodoroState::Idle,
                task_id: None,
                target_seconds: Self::DEFAULT_TARGET_SECONDS,
                elapsed_seconds: 0,
            })),
        }
    }

    /// Start a new focus session linked to an optional task.
    pub fn start(&self, task_id: Option<i64>, target_seconds: Option<u64>) -> PomodoroStatus {
        let mut inner = self.inner.write().unwrap();
        inner.task_id = task_id;
        inner.target_seconds = target_seconds.unwrap_or(Self::DEFAULT_TARGET_SECONDS);
        inner.elapsed_seconds = 0;
        inner.state = PomodoroState::Running;
        drop(inner);
        self.get_status()
    }

    /// Pause an ongoing focus session.
    pub fn pause(&self) -> PomodoroStatus {
        let mut inner = self.inner.write().unwrap();
        if inner.state == PomodoroState::Running {
            inner.state = PomodoroState::Paused;
        }
        drop(inner);
        self.get_status()
    }

    /// Resume a paused focus session.
    pub fn resume(&self) -> PomodoroStatus {
        let mut inner = self.inner.write().unwrap();
        if inner.state == PomodoroState::Paused {
            inner.state = PomodoroState::Running;
        }
        drop(inner);
        self.get_status()
    }

    /// Advance elapsed time by a given number of seconds.
    /// If target time is reached, transitions to `Completed`.
    pub fn tick(&self, delta_seconds: u64) -> PomodoroStatus {
        let mut inner = self.inner.write().unwrap();
        if inner.state == PomodoroState::Running {
            inner.elapsed_seconds += delta_seconds;
            if inner.elapsed_seconds >= inner.target_seconds {
                inner.elapsed_seconds = inner.target_seconds;
                inner.state = PomodoroState::Completed;
            }
        }
        drop(inner);
        self.get_status()
    }

    /// Stop/reset the current focus session.
    pub fn stop(&self) -> PomodoroStatus {
        let mut inner = self.inner.write().unwrap();
        inner.state = PomodoroState::Idle;
        inner.elapsed_seconds = 0;
        drop(inner);
        self.get_status()
    }

    /// Return the current reactive status of the timer.
    pub fn get_status(&self) -> PomodoroStatus {
        let inner = self.inner.read().unwrap();
        let remaining = inner.target_seconds.saturating_sub(inner.elapsed_seconds);
        PomodoroStatus {
            state: inner.state,
            task_id: inner.task_id,
            target_seconds: inner.target_seconds,
            elapsed_seconds: inner.elapsed_seconds,
            remaining_seconds: remaining,
            is_active: inner.state == PomodoroState::Running
                || inner.state == PomodoroState::Paused,
        }
    }

    /// Persist the current session state into SQLite storage.
    pub fn save_to_storage(&self, storage: &Storage) -> Result<(), StorageError> {
        let mut inner = self.inner.write().unwrap();
        let state_str = match inner.state {
            PomodoroState::Idle => "IDLE",
            PomodoroState::Running => "RUNNING",
            PomodoroState::Paused => "PAUSED",
            PomodoroState::Completed => "COMPLETED",
        };

        let record = FocusSessionRecord {
            id: inner.session_id,
            task_id: inner.task_id,
            state: state_str.to_string(),
            target_seconds: inner.target_seconds as i64,
            elapsed_seconds: inner.elapsed_seconds as i64,
            last_tick_utc: Some(chrono_now_string()),
            created_at: None,
        };

        let session_id = storage.save_focus_session(&record)?;
        inner.session_id = Some(session_id);
        Ok(())
    }

    /// Restore the latest active session from SQLite storage into this timer instance.
    pub fn load_from_storage(&self, storage: &Storage) -> Result<PomodoroStatus, StorageError> {
        let maybe_session = storage.get_active_focus_session()?;
        if let Some(session) = maybe_session {
            let state = match session.state.as_str() {
                "RUNNING" => PomodoroState::Running,
                "PAUSED" => PomodoroState::Paused,
                "COMPLETED" => PomodoroState::Completed,
                _ => PomodoroState::Idle,
            };

            let mut inner = self.inner.write().unwrap();
            inner.session_id = session.id;
            inner.task_id = session.task_id;
            inner.state = state;
            inner.target_seconds = session.target_seconds.max(0) as u64;
            inner.elapsed_seconds = session.elapsed_seconds.max(0) as u64;
        }
        Ok(self.get_status())
    }

    /// Restore the latest active session from SQLite storage after app restart.
    pub fn restore_from_storage(storage: &Storage) -> Result<Self, StorageError> {
        let maybe_session = storage.get_active_focus_session()?;
        let timer = Self::new();
        if let Some(session) = maybe_session {
            let state = match session.state.as_str() {
                "RUNNING" => PomodoroState::Running,
                "PAUSED" => PomodoroState::Paused,
                "COMPLETED" => PomodoroState::Completed,
                _ => PomodoroState::Idle,
            };

            let mut inner = timer.inner.write().unwrap();
            inner.session_id = session.id;
            inner.task_id = session.task_id;
            inner.state = state;
            inner.target_seconds = session.target_seconds.max(0) as u64;
            inner.elapsed_seconds = session.elapsed_seconds.max(0) as u64;
        }
        Ok(timer)
    }
}

fn chrono_now_string() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", duration.as_secs(), duration.subsec_millis())
}

// ─── Automated Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::storage::Storage;

    /// Test 1: `productivity::test_pomodoro_state_machine_transitions_and_persistence`
    /// Test timer transitions: Start -> Pause -> Resume -> Complete / Stop -> Sudden shutdown and session recovery.
    #[test]
    fn test_pomodoro_state_machine_transitions_and_persistence() {
        let storage = Storage::open_in_memory().expect("Should open test storage");
        let timer = PomodoroTimer::new();

        // 1. Initial State: Idle
        let initial = timer.get_status();
        assert_eq!(initial.state, PomodoroState::Idle);
        assert!(!initial.is_active);

        // Insert a task to satisfy foreign key constraint
        let task_id = storage
            .insert_task(&TaskItem {
                id: None,
                title: "مهمة التركيز للاختبار".to_string(),
                description: "وصف".to_string(),
                period: "today".to_string(),
                is_completed: false,
                tags: "focus".to_string(),
                created_at: None,
                updated_at: None,
            })
            .unwrap();

        // 2. Start session (1500 seconds / 25 mins)
        let started = timer.start(Some(task_id), Some(1500));
        assert_eq!(started.state, PomodoroState::Running);
        assert_eq!(started.task_id, Some(task_id));
        assert_eq!(started.remaining_seconds, 1500);
        assert!(started.is_active);

        // 3. Advance time by 300 seconds (5 mins elapsed)
        let ticked = timer.tick(300);
        assert_eq!(ticked.elapsed_seconds, 300);
        assert_eq!(ticked.remaining_seconds, 1200);
        assert_eq!(ticked.state, PomodoroState::Running);

        // 4. Pause session
        let paused = timer.pause();
        assert_eq!(paused.state, PomodoroState::Paused);
        assert!(paused.is_active);

        // Ticking while paused must NOT advance time
        let tick_while_paused = timer.tick(50);
        assert_eq!(tick_while_paused.elapsed_seconds, 300);

        // 5. Resume session
        let resumed = timer.resume();
        assert_eq!(resumed.state, PomodoroState::Running);

        // 6. Persist to storage (simulating sudden shutdown or save)
        timer
            .save_to_storage(&storage)
            .expect("Should persist session");

        // 7. Simulate app crash/restart: create new timer and restore from storage
        let restored_timer = PomodoroTimer::restore_from_storage(&storage)
            .expect("Should restore session from storage");
        let restored_status = restored_timer.get_status();

        assert_eq!(restored_status.state, PomodoroState::Running);
        assert_eq!(restored_status.task_id, Some(task_id));
        assert_eq!(restored_status.elapsed_seconds, 300);
        assert_eq!(restored_status.remaining_seconds, 1200);

        // 8. Complete session: tick remaining 1200 seconds
        let completed = restored_timer.tick(1200);
        assert_eq!(completed.state, PomodoroState::Completed);
        assert_eq!(completed.remaining_seconds, 0);

        // 9. Stop and reset
        let stopped = restored_timer.stop();
        assert_eq!(stopped.state, PomodoroState::Idle);
        assert_eq!(stopped.elapsed_seconds, 0);
    }

    /// Test 2: `productivity::test_tasks_and_captures_crud_with_edge_inputs`
    /// Test adding, updating, and deleting tasks and capture notes with empty, huge, or mixed Arabic/English inputs.
    #[test]
    fn test_tasks_and_captures_crud_with_edge_inputs() {
        let storage = Storage::open_in_memory().expect("Should open test storage");

        // 1. Edge Case: Empty task title must be rejected
        let res_empty = TaskManager::create_task(&storage, "   ", None, None, None);
        assert!(res_empty.is_err(), "Empty task title must be rejected");

        // 2. Edge Case: XSS in task title and description
        let xss_title = "مهمة اختبار <script>alert(1)</script> 2026";
        let xss_desc = "وصف <b>مهم</b> <iframe src='malicious.html'></iframe>";
        let created_task = TaskManager::create_task(
            &storage,
            xss_title,
            Some(xss_desc),
            Some("morning"),
            Some("urgent,عاجل"),
        )
        .expect("Should sanitize and create task");

        assert!(!created_task.title.contains("<script>"));
        assert!(!created_task.description.contains("<iframe"));
        assert!(created_task.title.contains("مهمة اختبار"));

        let task_id = created_task.id.unwrap();

        // 3. Toggle completion
        let is_done = TaskManager::toggle_task(&storage, task_id).unwrap();
        assert!(is_done);
        let is_undone = TaskManager::toggle_task(&storage, task_id).unwrap();
        assert!(!is_undone);

        // 4. Update task with mixed Arabic / English and long description
        let huge_description = "تفاصيل العمل ".repeat(500);
        let updated = TaskManager::update_task(
            &storage,
            task_id,
            "مهمة محدثة: Deploy Tauri v2 on Windows & macOS",
            Some(&huge_description),
            Some("evening"),
            Some("release,إصدار"),
        )
        .expect("Should update task successfully");

        assert_eq!(updated.period, "evening");
        assert!(updated.title.contains("Deploy Tauri v2"));

        // 5. Delete task
        TaskManager::delete_task(&storage, task_id).expect("Should delete task");
        let remaining_tasks = TaskManager::list_tasks(&storage, None).unwrap();
        assert!(remaining_tasks.is_empty());

        // 6. Capture Notes CRUD with edge inputs
        // Empty content must be rejected
        let res_empty_note = CaptureManager::create_capture(&storage, "", None, None, None);
        assert!(
            res_empty_note.is_err(),
            "Empty note content must be rejected"
        );

        // Valid capture note with URL
        let note = CaptureManager::create_capture(
            &storage,
            "ملاحظة توثيق المشروع: راجع مسودة المعمارية",
            Some("link"),
            Some("docs,architecture"),
            Some("https://rawaq.app/arch"),
        )
        .expect("Should create capture note");

        assert_eq!(note.note_type, "link");
        assert_eq!(note.source_url, Some("https://rawaq.app/arch".to_string()));

        let note_id = note.id.unwrap();
        let notes_list = CaptureManager::list_captures(&storage, None).unwrap();
        assert_eq!(notes_list.len(), 1);

        // Delete note
        CaptureManager::delete_capture(&storage, note_id).expect("Should delete note");
        let notes_after = CaptureManager::list_captures(&storage, None).unwrap();
        assert!(notes_after.is_empty());
    }
}
