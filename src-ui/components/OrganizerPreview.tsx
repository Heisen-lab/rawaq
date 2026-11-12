import React from "react";
import { useI18n } from "../i18n/I18nContext";
import { BdiText } from "./BdiText";

export interface PlannedMoveModel {
  source_path: string;
  original_destination_path: string;
  resolved_destination_path: string;
  file_name: string;
  file_size: number;
  file_hash: string;
  rule_name: string;
  collision_detected: boolean;
}

export interface SkippedItemModel {
  path: string;
  file_name: string;
  reason: string;
}

export interface DryRunPlanModel {
  plan_id: string;
  source_folder: string;
  moves: PlannedMoveModel[];
  skipped: SkippedItemModel[];
}

export interface ExecutionItemReportModel {
  source_path: string;
  destination_path: string;
  status: string;
  error_message?: string;
}

export interface ExecutionReportModel {
  batch_id: string;
  total_planned: number;
  successful: number;
  failed: number;
  items: ExecutionItemReportModel[];
}

export interface UndoReportModel {
  batch_id: string;
  total_operations: number;
  restored: number;
  failed: number;
  errors: string[];
}

export interface OrganizerPreviewProps {
  plan: DryRunPlanModel | null;
  onExecute: () => void;
  isExecuting?: boolean;
  executionReport: ExecutionReportModel | null;
  onUndo: (batchId: string) => void;
  undoReport: UndoReportModel | null;
}

export function OrganizerPreview({
  plan,
  onExecute,
  isExecuting = false,
  executionReport,
  onUndo,
  undoReport,
}: OrganizerPreviewProps): React.ReactElement {
  const { t } = useI18n();

  if (!plan) {
    return (
      <div className="rawaq-card" data-testid="organizer-empty">
        <p>{t("organizer.emptyPlan")}</p>
      </div>
    );
  }

  const moves = plan.moves;
  const skipped = plan.skipped;

  return (
    <section
      className="rawaq-organizer-container"
      data-testid="organizer-preview"
      aria-labelledby="organizer-heading"
    >
      <header className="rawaq-organizer-header">
        <div>
          <h2 id="organizer-heading" className="rawaq-organizer-title">
            {t("organizer.previewHeading")}
          </h2>
          <p className="rawaq-organizer-subtitle">
            {t("organizer.previewSubtitle")}
          </p>
        </div>

        <div className="rawaq-organizer-source-badge">
          <span>📁 </span>
          <BdiText text={plan.source_folder} testId="plan-source-folder" />
        </div>
      </header>

      {/* Undo Banner if an undo was executed */}
      {undoReport && (
        <div
          className="rawaq-box-banner warning"
          data-testid="undo-success-banner"
          role="status"
        >
          <div className="banner-title">
            ✓ {t("organizer.undoSuccess", { count: undoReport.restored })}
          </div>
          <div className="banner-desc">
            معرّف الدفعة: <code>{undoReport.batch_id}</code>
          </div>
        </div>
      )}

      {/* Post-execution Report Card */}
      {executionReport && !undoReport && (
        <div
          className="rawaq-box-banner"
          style={{ backgroundColor: "rgba(56, 189, 248, 0.1)", borderInlineStart: "4px solid #38bdf8" }}
          data-testid="execution-report-banner"
          role="status"
        >
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <div>
              <div className="banner-title" style={{ color: "#38bdf8" }}>
                ✓ تم تنفيذ الفرز: {executionReport.successful} ناجح / {executionReport.failed} متعذر
              </div>
              <div className="banner-desc">
                معرّف الدفعة: <code>{executionReport.batch_id}</code>
              </div>
            </div>
            <button
              type="button"
              className="rawaq-button"
              data-testid="trigger-undo-btn"
              onClick={() => onUndo(executionReport.batch_id)}
            >
              ⤺ {t("organizer.undoBtn")}
            </button>
          </div>
        </div>
      )}

      {/* Moves Preview Table */}
      <div className="rawaq-table-wrapper">
        <table className="rawaq-table" data-testid="planned-moves-table">
          <thead>
            <tr>
              <th scope="col">{t("organizer.sourceHeader")}</th>
              <th scope="col">{t("organizer.destHeader")}</th>
              <th scope="col">{t("organizer.ruleHeader")}</th>
              <th scope="col">الحجم</th>
            </tr>
          </thead>
          <tbody>
            {moves.map((m, idx) => (
              <tr key={m.source_path} data-testid={`move-row-${idx}`}>
                <td>
                  <span className="file-icon" aria-hidden="true">
                    📄{" "}
                  </span>
                  <BdiText text={m.file_name} testId={`move-filename-${idx}`} />
                </td>
                <td>
                  <BdiText
                    text={m.resolved_destination_path}
                    testId={`move-dest-${idx}`}
                  />
                  {m.collision_detected && (
                    <div
                      className="collision-badge"
                      data-testid={`collision-badge-${idx}`}
                    >
                      ⚠️{" "}
                      {t("organizer.collisionWarning", {
                        resolved: m.resolved_destination_path.split("/").pop() ?? "",
                      })}
                    </div>
                  )}
                </td>
                <td>
                  <span className="rule-badge">{m.rule_name}</span>
                </td>
                <td style={{ fontVariantNumeric: "tabular-nums" }}>
                  {formatSize(m.file_size)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Skipped Items */}
      {skipped.length > 0 && (
        <details className="rawaq-skipped-details" data-testid="skipped-section">
          <summary>
            {t("organizer.skippedHeading", { count: skipped.length })}
          </summary>
          <ul className="rawaq-file-list" style={{ marginBlockStart: "0.5rem" }}>
            {skipped.map((s, idx) => (
              <li
                key={s.path}
                className="rawaq-file-item"
                data-testid={`skipped-item-${idx}`}
                style={{ borderInlineStartColor: "#64748b" }}
              >
                <BdiText text={s.file_name} /> —{" "}
                <span style={{ color: "var(--text-secondary)", fontSize: "0.8125rem" }}>
                  {s.reason}
                </span>
              </li>
            ))}
          </ul>
        </details>
      )}

      {/* Execution Approval Bar */}
      <footer className="rawaq-organizer-footer">
        <span style={{ color: "var(--text-secondary)", fontSize: "0.875rem" }}>
          إجمالي الملفات المخطط نقلها: <strong>{moves.length}</strong>
        </span>

        <button
          type="button"
          className="rawaq-button primary"
          data-testid="approve-execute-btn"
          onClick={onExecute}
          disabled={moves.length === 0 || isExecuting || executionReport !== null}
        >
          {isExecuting ? "جارٍ التنفيذ والتحقق..." : t("organizer.executeBtn")}
        </button>
      </footer>
    </section>
  );
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
