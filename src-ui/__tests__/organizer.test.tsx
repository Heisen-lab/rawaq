import { useState } from "react";
import { describe, expect, it } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { I18nProvider } from "../i18n/I18nContext";
import {
  OrganizerPreview,
  type DryRunPlanModel,
  type ExecutionReportModel,
  type UndoReportModel,
} from "../components/OrganizerPreview";

describe("Stage 4 Safe Auto-Organizer & Dry-Run Preview UI", () => {
  const samplePlan: DryRunPlanModel = {
    plan_id: "plan-test-001",
    source_folder: "/home/ahmad/Desktop",
    moves: [
      {
        source_path: "/home/ahmad/Desktop/تقرير_الميزانية_2026.pdf",
        original_destination_path: "/home/ahmad/Documents/PDFs/تقرير_الميزانية_2026.pdf",
        resolved_destination_path: "/home/ahmad/Documents/PDFs/تقرير_الميزانية_2026.pdf",
        file_name: "تقرير_الميزانية_2026.pdf",
        file_size: 1048576,
        file_hash: "hash123456",
        rule_name: "قاعدة ملفات PDF",
        collision_detected: false,
      },
      {
        source_path: "/home/ahmad/Desktop/project_notes.docx",
        original_destination_path: "/home/ahmad/Documents/Word/project_notes.docx",
        resolved_destination_path: "/home/ahmad/Documents/Word/project_notes (1).docx",
        file_name: "project_notes.docx",
        file_size: 204800,
        file_hash: "hash789012",
        rule_name: "قاعدة المستندات",
        collision_detected: true, // collision resolved safely!
      },
    ],
    skipped: [
      {
        path: "/home/ahmad/Desktop/archive.zip",
        file_name: "archive.zip",
        reason: "No matching rule",
      },
    ],
  };

  function TestHarness({
    initialPlan = samplePlan as DryRunPlanModel | null,
    initialLocale = "ar" as "ar" | "en",
  }) {
    const [execReport, setExecReport] = useState<ExecutionReportModel | null>(null);
    const [undoReport, setUndoReport] = useState<UndoReportModel | null>(null);

    return (
      <I18nProvider initialLocale={initialLocale}>
        <OrganizerPreview
          plan={initialPlan}
          onExecute={() => {
            setExecReport({
              batch_id: "batch-2026-11-12",
              total_planned: 2,
              successful: 2,
              failed: 0,
              items: [],
            });
          }}
          executionReport={execReport}
          onUndo={(batchId) => {
            setUndoReport({
              batch_id: batchId,
              total_operations: 2,
              restored: 2,
              failed: 0,
              errors: [],
            });
          }}
          undoReport={undoReport}
        />
      </I18nProvider>
    );
  }

  it("renders Dry-Run preview with collision detection in Arabic (RTL)", () => {
    render(<TestHarness initialLocale="ar" />);

    expect(document.documentElement.dir).toBe("rtl");
    expect(screen.getByTestId("organizer-preview")).toBeInTheDocument();

    // Verify 2 moves rendered in table
    expect(screen.getByTestId("move-row-0")).toBeInTheDocument();
    expect(screen.getByTestId("move-row-1")).toBeInTheDocument();

    // Verify mixed Arabic-English file name is isolated with <bdi>
    const file0Bdi = screen.getByTestId("move-filename-0");
    expect(file0Bdi.tagName.toLowerCase()).toBe("bdi");
    expect(file0Bdi).toHaveTextContent("تقرير_الميزانية_2026.pdf");

    // File 1 has a collision badge
    expect(screen.queryByTestId("collision-badge-0")).not.toBeInTheDocument();
    const collisionBadge1 = screen.getByTestId("collision-badge-1");
    expect(collisionBadge1).toBeInTheDocument();
    expect(collisionBadge1).toHaveTextContent("تعارض تم حله بأمان");

    // Verify skipped section exists
    expect(screen.getByTestId("skipped-section")).toBeInTheDocument();
    expect(screen.getByTestId("skipped-item-0")).toHaveTextContent("archive.zip");

    // Trigger Execution
    const executeBtn = screen.getByTestId("approve-execute-btn");
    expect(executeBtn).toBeEnabled();
    fireEvent.click(executeBtn);

    // Verify execution report banner appears
    expect(screen.getByTestId("execution-report-banner")).toBeInTheDocument();
    expect(screen.getByText(/تم تنفيذ الفرز: 2 ناجح/)).toBeInTheDocument();

    // Trigger One-Click Undo
    const undoBtn = screen.getByTestId("trigger-undo-btn");
    fireEvent.click(undoBtn);

    // Verify undo success banner
    expect(screen.getByTestId("undo-success-banner")).toBeInTheDocument();
    expect(screen.getByText(/تم استعادة 2 ملف إلى مكانه الأصلي بنجاح/)).toBeInTheDocument();
  });

  it("renders Dry-Run preview in English (LTR)", () => {
    render(<TestHarness initialLocale="en" />);

    expect(document.documentElement.dir).toBe("ltr");
    expect(screen.getByText("Dry-Run Preview")).toBeInTheDocument();
    expect(screen.getByText("Source File")).toBeInTheDocument();
    expect(screen.getByText("Proposed Destination")).toBeInTheDocument();
    expect(screen.getByTestId("approve-execute-btn")).toHaveTextContent("Approve & Execute Move");

    // Collision badge in English
    const collisionBadge1 = screen.getByTestId("collision-badge-1");
    expect(collisionBadge1).toHaveTextContent("Safely resolved collision");
  });

  it("handles empty plan gracefully", () => {
    render(<TestHarness initialPlan={null} />);
    expect(screen.getByTestId("organizer-empty")).toBeInTheDocument();
  });
});
