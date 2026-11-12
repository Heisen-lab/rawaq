import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import {
  ProductivityHub,
  TaskItemUI,
  CaptureNoteUI,
  PomodoroStatusUI,
  SearchResultItemUI,
} from "../components/ProductivityHub";
import { I18nProvider } from "../i18n/I18nContext";

describe("Stage 6 Productivity Hub, Tasks, Timer & Search UI", () => {
  const sampleTasks: TaskItemUI[] = [
    {
      id: 1,
      title: "إعداد تقرير الميزانية السنويّة",
      description: "مراجعة الأرقام مع الإدارة",
      period: "morning",
      is_completed: false,
      tags: "finance,ميزانية",
    },
    {
      id: 2,
      title: "Build Tauri v2 Desktop Engine",
      description: "Multi-monitor and BiDi support",
      period: "today",
      is_completed: true,
      tags: "code,rust",
    },
    {
      id: 3,
      title: "قراءة الملاحظات المسائية",
      description: "",
      period: "evening",
      is_completed: false,
      tags: "reading",
    },
  ];

  const sampleCaptures: CaptureNoteUI[] = [
    {
      id: 101,
      content: "ملاحظة سريعة: رابط توثيق المعمارية",
      note_type: "link",
      tags: "docs",
      source_url: "https://rawaq.app/docs",
    },
  ];

  const runningTimer: PomodoroStatusUI = {
    state: "running",
    task_id: 2,
    target_seconds: 1500,
    elapsed_seconds: 300,
    remaining_seconds: 1200,
    is_active: true,
  };

  it("renders ProductivityHub in Arabic (RTL) and filters tasks by period", () => {
    render(
      <I18nProvider initialLocale="ar">
        <ProductivityHub tasks={sampleTasks} captures={sampleCaptures} />
      </I18nProvider>
    );

    // Default period is "today"
    expect(screen.getByText("Build Tauri v2 Desktop Engine")).toBeDefined();
    expect(screen.queryByText("إعداد تقرير الميزانية السنويّة")).toBeNull();

    // Switch to Morning tab
    const morningTab = screen.getByTestId("period-morning-tab");
    fireEvent.click(morningTab);
    expect(screen.getByText("إعداد تقرير الميزانية السنويّة")).toBeDefined();
    expect(screen.queryByText("Build Tauri v2 Desktop Engine")).toBeNull();

    // Switch to Evening tab
    const eveningTab = screen.getByTestId("period-evening-tab");
    fireEvent.click(eveningTab);
    expect(screen.getByText("قراءة الملاحظات المسائية")).toBeDefined();
  });

  it("handles adding, toggling, and deleting tasks", () => {
    const onAddTask = vi.fn();
    const onToggleTask = vi.fn();
    const onDeleteTask = vi.fn();

    render(
      <I18nProvider initialLocale="ar">
        <ProductivityHub
          tasks={sampleTasks}
          onAddTask={onAddTask}
          onToggleTask={onToggleTask}
          onDeleteTask={onDeleteTask}
        />
      </I18nProvider>
    );

    // Add new task
    const input = screen.getByTestId("new-task-input");
    fireEvent.change(input, { target: { value: "مهمة اختبار جديدة" } });
    const addBtn = screen.getByTestId("add-task-btn");
    fireEvent.click(addBtn);
    expect(onAddTask).toHaveBeenCalledWith("مهمة اختبار جديدة", "today");

    // Toggle task
    const toggleCheckbox = screen.getByTestId("toggle-task-2");
    fireEvent.click(toggleCheckbox);
    expect(onToggleTask).toHaveBeenCalledWith(2);

    // Delete task
    const deleteBtn = screen.getByTestId("delete-task-2");
    fireEvent.click(deleteBtn);
    expect(onDeleteTask).toHaveBeenCalledWith(2);
  });

  it("renders Pomodoro focus timer and controls correctly", () => {
    const onPause = vi.fn();
    const onStop = vi.fn();

    render(
      <I18nProvider initialLocale="ar">
        <ProductivityHub timerStatus={runningTimer} onTimerPause={onPause} onTimerStop={onStop} />
      </I18nProvider>
    );

    // Check formatted digits: 1200 seconds = 20:00
    expect(screen.getByTestId("pomodoro-timer-display").textContent).toBe("20:00");

    // Click pause
    const pauseBtn = screen.getByTestId("timer-pause-btn");
    fireEvent.click(pauseBtn);
    expect(onPause).toHaveBeenCalled();

    // Click stop
    const stopBtn = screen.getByTestId("timer-stop-btn");
    fireEvent.click(stopBtn);
    expect(onStop).toHaveBeenCalled();
  });

  it("handles Quick Capture note submission and deletion", () => {
    const onAddCapture = vi.fn();
    const onDeleteCapture = vi.fn();

    render(
      <I18nProvider initialLocale="ar">
        <ProductivityHub
          captures={sampleCaptures}
          onAddCapture={onAddCapture}
          onDeleteCapture={onDeleteCapture}
        />
      </I18nProvider>
    );

    expect(screen.getByText("ملاحظة سريعة: رابط توثيق المعمارية")).toBeDefined();

    // Delete capture note
    const deleteBtn = screen.getByTestId("delete-capture-101");
    fireEvent.click(deleteBtn);
    expect(onDeleteCapture).toHaveBeenCalledWith(101);

    // Add new text capture note
    const input = screen.getByTestId("capture-input");
    fireEvent.change(input, { target: { value: "فكرة ملتقطة جديدة" } });
    const submitBtn = screen.getByTestId("save-capture-btn");
    fireEvent.click(submitBtn);
    expect(onAddCapture).toHaveBeenCalledWith("فكرة ملتقطة جديدة", "text", undefined);
  });

  it("renders unified search results with performance indicator", () => {
    const onSearch = vi.fn();
    const sampleResults: SearchResultItemUI[] = [
      {
        kind: "task",
        id: "1",
        title: "إعداد تقرير الميزانية",
        subtitle: "مهمة (morning)",
      },
      {
        kind: "file",
        id: "C:/Desktop/report.pdf",
        title: "report.pdf",
        subtitle: "C:/Desktop/report.pdf",
      },
    ];

    render(
      <I18nProvider initialLocale="ar">
        <ProductivityHub
          onSearch={onSearch}
          searchResults={sampleResults}
          searchElapsedMs={2}
        />
      </I18nProvider>
    );

    const searchInput = screen.getByTestId("unified-search-input");
    fireEvent.change(searchInput, { target: { value: "تقرير" } });
    expect(onSearch).toHaveBeenCalledWith("تقرير");

    expect(screen.getByTestId("search-results-panel")).toBeDefined();
    expect(screen.getByText("استغرق البحث 2 مللي ثانية")).toBeDefined();
    expect(screen.getByText("إعداد تقرير الميزانية")).toBeDefined();
    expect(screen.getByText("report.pdf")).toBeDefined();
  });
});
