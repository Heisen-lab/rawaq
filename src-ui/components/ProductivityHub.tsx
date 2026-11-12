import { useState, useMemo } from "react";
import { useI18n } from "../i18n/I18nContext";
import { BdiText } from "./BdiText";

export interface TaskItemUI {
  id: number;
  title: string;
  description: string;
  period: "morning" | "today" | "evening";
  is_completed: boolean;
  tags: string;
  created_at?: string;
}

export interface CaptureNoteUI {
  id: number;
  content: string;
  note_type: "text" | "link" | "snippet";
  tags: string;
  source_url?: string | null;
  created_at?: string;
}

export interface PomodoroStatusUI {
  state: "idle" | "running" | "paused" | "completed";
  task_id?: number | null;
  target_seconds: number;
  elapsed_seconds: number;
  remaining_seconds: number;
  is_active: boolean;
}

export interface SearchResultItemUI {
  kind: "file" | "task" | "note";
  id: string;
  title: string;
  subtitle: string;
  extra_info?: string | null;
}

export interface ProductivityHubProps {
  tasks?: TaskItemUI[];
  captures?: CaptureNoteUI[];
  timerStatus?: PomodoroStatusUI;
  searchResults?: SearchResultItemUI[];
  searchElapsedMs?: number;
  onSearch?: (query: string) => void;
  onAddTask?: (title: string, period: "morning" | "today" | "evening") => void;
  onToggleTask?: (id: number) => void;
  onDeleteTask?: (id: number) => void;
  onAddCapture?: (content: string, noteType: "text" | "link" | "snippet", url?: string) => void;
  onDeleteCapture?: (id: number) => void;
  onTimerStart?: (taskId?: number) => void;
  onTimerPause?: () => void;
  onTimerResume?: () => void;
  onTimerStop?: () => void;
}

export function ProductivityHub({
  tasks = [],
  captures = [],
  timerStatus = {
    state: "idle",
    target_seconds: 1500,
    elapsed_seconds: 0,
    remaining_seconds: 1500,
    is_active: false,
  },
  searchResults,
  searchElapsedMs,
  onSearch,
  onAddTask,
  onToggleTask,
  onDeleteTask,
  onAddCapture,
  onDeleteCapture,
  onTimerStart,
  onTimerPause,
  onTimerResume,
  onTimerStop,
}: ProductivityHubProps) {
  const { t } = useI18n();

  const [searchQuery, setSearchQuery] = useState("");
  const [activePeriod, setActivePeriod] = useState<"morning" | "today" | "evening">("today");
  const [newTaskTitle, setNewTaskTitle] = useState("");
  const [newCaptureContent, setNewCaptureContent] = useState("");
  const [newCaptureUrl, setNewCaptureUrl] = useState("");
  const [newCaptureType, setNewCaptureType] = useState<"text" | "link" | "snippet">("text");

  const filteredTasks = useMemo(() => {
    return tasks.filter((task) => task.period === activePeriod);
  }, [tasks, activePeriod]);

  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setSearchQuery(val);
    onSearch?.(val);
  };

  const handleTaskSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newTaskTitle.trim()) return;
    onAddTask?.(newTaskTitle.trim(), activePeriod);
    setNewTaskTitle("");
  };

  const handleCaptureSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newCaptureContent.trim()) return;
    onAddCapture?.(
      newCaptureContent.trim(),
      newCaptureType,
      newCaptureUrl.trim() ? newCaptureUrl.trim() : undefined
    );
    setNewCaptureContent("");
    setNewCaptureUrl("");
  };

  const formatTimer = (totalSeconds: number) => {
    const mins = Math.floor(totalSeconds / 60);
    const secs = totalSeconds % 60;
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  };

  return (
    <div className="rawaq-productivity-hub" data-testid="productivity-hub">
      {/* ─── 1. Unified Local Search Bar ─── */}
      <section className="search-section" data-testid="search-section">
        <div className="search-input-wrapper">
          <span className="search-icon">🔍</span>
          <input
            type="search"
            className="search-input"
            value={searchQuery}
            onChange={handleSearchChange}
            placeholder={t("productivity.searchPlaceholder")}
            data-testid="unified-search-input"
            aria-label={t("productivity.searchPlaceholder")}
          />
        </div>

        {searchQuery.trim().length > 0 && (
          <div className="search-results-panel" data-testid="search-results-panel">
            {typeof searchElapsedMs === "number" && (
              <div className="search-meta-bar" data-testid="search-meta">
                <span>{t("productivity.searchTook", { ms: searchElapsedMs })}</span>
              </div>
            )}

            {searchResults && searchResults.length > 0 ? (
              <ul className="search-results-list" data-testid="search-results-list">
                {searchResults.map((item) => (
                  <li key={`${item.kind}-${item.id}`} className="search-result-item" data-testid={`search-item-${item.id}`}>
                    <span className={`search-badge ${item.kind}`}>
                      {item.kind === "file" && t("productivity.badgeFile")}
                      {item.kind === "task" && t("productivity.badgeTask")}
                      {item.kind === "note" && t("productivity.badgeNote")}
                    </span>
                    <div className="search-result-text">
                      <strong className="search-result-title">
                        <BdiText text={item.title} />
                      </strong>
                      <span className="search-result-sub">
                        <BdiText text={item.subtitle} />
                      </span>
                    </div>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="search-no-results" data-testid="search-no-results">
                {t("productivity.noResults", { query: searchQuery })}
              </p>
            )}
          </div>
        )}
      </section>

      {/* ─── 2. Focus Timer (Pomodoro) ─── */}
      <section className="pomodoro-section" data-testid="pomodoro-section">
        <div className="pomodoro-card">
          <div className="pomodoro-header">
            <span className="pomodoro-icon">⏱️</span>
            <h3 className="pomodoro-title">{t("productivity.pomodoroTitle")}</h3>
            <span className={`pomodoro-status-tag ${timerStatus.state}`}>
              {timerStatus.state.toUpperCase()}
            </span>
          </div>

          <div className="pomodoro-display" data-testid="pomodoro-timer-display">
            <span className="pomodoro-digits">{formatTimer(timerStatus.remaining_seconds)}</span>
          </div>

          <div className="pomodoro-controls">
            {timerStatus.state === "idle" || timerStatus.state === "completed" ? (
              <button
                type="button"
                className="btn-timer primary"
                onClick={() => onTimerStart?.()}
                data-testid="timer-start-btn"
              >
                ▶ {t("productivity.timerStart")}
              </button>
            ) : null}

            {timerStatus.state === "running" ? (
              <button
                type="button"
                className="btn-timer warning"
                onClick={onTimerPause}
                data-testid="timer-pause-btn"
              >
                ⏸ {t("productivity.timerPause")}
              </button>
            ) : null}

            {timerStatus.state === "paused" ? (
              <button
                type="button"
                className="btn-timer primary"
                onClick={onTimerResume}
                data-testid="timer-resume-btn"
              >
                ▶ {t("productivity.timerResume")}
              </button>
            ) : null}

            {timerStatus.is_active ? (
              <button
                type="button"
                className="btn-timer danger"
                onClick={onTimerStop}
                data-testid="timer-stop-btn"
              >
                ⏹ {t("productivity.timerStop")}
              </button>
            ) : null}
          </div>
        </div>
      </section>

      {/* ─── 3. Daily Tasks (Morning / Today / Evening) ─── */}
      <section className="tasks-section" data-testid="tasks-section">
        <div className="tasks-header">
          <h3 className="tasks-title">{t("productivity.tasksTitle")}</h3>
          <div className="period-tabs-bar" role="tablist">
            <button
              type="button"
              role="tab"
              aria-selected={activePeriod === "morning"}
              className={`period-tab-btn ${activePeriod === "morning" ? "active" : ""}`}
              onClick={() => setActivePeriod("morning")}
              data-testid="period-morning-tab"
            >
              🌅 {t("productivity.periodMorning")}
            </button>
            <button
              type="button"
              role="tab"
              aria-selected={activePeriod === "today"}
              className={`period-tab-btn ${activePeriod === "today" ? "active" : ""}`}
              onClick={() => setActivePeriod("today")}
              data-testid="period-today-tab"
            >
              ☀️ {t("productivity.periodToday")}
            </button>
            <button
              type="button"
              role="tab"
              aria-selected={activePeriod === "evening"}
              className={`period-tab-btn ${activePeriod === "evening" ? "active" : ""}`}
              onClick={() => setActivePeriod("evening")}
              data-testid="period-evening-tab"
            >
              🌙 {t("productivity.periodEvening")}
            </button>
          </div>
        </div>

        <form onSubmit={handleTaskSubmit} className="add-task-form">
          <input
            type="text"
            className="add-task-input"
            value={newTaskTitle}
            onChange={(e) => setNewTaskTitle(e.target.value)}
            placeholder={t("productivity.addTaskPlaceholder")}
            data-testid="new-task-input"
          />
          <button type="submit" className="add-task-submit-btn" data-testid="add-task-btn">
            +
          </button>
        </form>

        <ul className="tasks-list" data-testid="tasks-list">
          {filteredTasks.length === 0 ? (
            <li className="tasks-empty-item">لا توجد مهام في هذه الفترة</li>
          ) : (
            filteredTasks.map((task) => (
              <li
                key={task.id}
                className={`task-row ${task.is_completed ? "completed" : ""}`}
                data-testid={`task-row-${task.id}`}
              >
                <label className="task-checkbox-label">
                  <input
                    type="checkbox"
                    checked={task.is_completed}
                    onChange={() => onToggleTask?.(task.id)}
                    data-testid={`toggle-task-${task.id}`}
                  />
                  <span className="task-title-text">
                    <BdiText text={task.title} />
                  </span>
                </label>
                <div className="task-actions-group">
                  {task.tags && <span className="task-tag-badge">{task.tags}</span>}
                  <button
                    type="button"
                    className="task-delete-btn"
                    onClick={() => onDeleteTask?.(task.id)}
                    data-testid={`delete-task-${task.id}`}
                    aria-label="delete task"
                  >
                    🗑️
                  </button>
                </div>
              </li>
            ))
          )}
        </ul>
      </section>

      {/* ─── 4. Quick Capture ─── */}
      <section className="capture-section" data-testid="capture-section">
        <h3 className="capture-title">{t("productivity.captureTitle")}</h3>
        <form onSubmit={handleCaptureSubmit} className="capture-form">
          <div className="capture-type-selector">
            <label>
              <input
                type="radio"
                name="capture_type"
                value="text"
                checked={newCaptureType === "text"}
                onChange={() => setNewCaptureType("text")}
              />
              📝 نص
            </label>
            <label>
              <input
                type="radio"
                name="capture_type"
                value="link"
                checked={newCaptureType === "link"}
                onChange={() => setNewCaptureType("link")}
              />
              🔗 رابط
            </label>
          </div>
          <input
            type="text"
            className="capture-content-input"
            value={newCaptureContent}
            onChange={(e) => setNewCaptureContent(e.target.value)}
            placeholder={t("productivity.capturePlaceholder")}
            data-testid="capture-input"
          />
          {newCaptureType === "link" && (
            <input
              type="url"
              className="capture-url-input"
              value={newCaptureUrl}
              onChange={(e) => setNewCaptureUrl(e.target.value)}
              placeholder="https://..."
              data-testid="capture-url-input"
            />
          )}
          <button type="submit" className="capture-submit-btn" data-testid="save-capture-btn">
            {t("productivity.saveCapture")}
          </button>
        </form>

        <ul className="captures-list" data-testid="captures-list">
          {captures.map((note) => (
            <li key={note.id} className="capture-item" data-testid={`capture-item-${note.id}`}>
              <div className="capture-item-text">
                <span className="capture-badge">{note.note_type}</span>
                <p className="capture-content">
                  <BdiText text={note.content} />
                </p>
                {note.source_url && (
                  <a
                    href={note.source_url}
                    target="_blank"
                    rel="noreferrer"
                    className="capture-link"
                  >
                    {note.source_url}
                  </a>
                )}
              </div>
              <button
                type="button"
                className="capture-delete-btn"
                onClick={() => onDeleteCapture?.(note.id)}
                data-testid={`delete-capture-${note.id}`}
                aria-label="delete capture"
              >
                ✕
              </button>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
