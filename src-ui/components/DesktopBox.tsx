import React from "react";
import { useI18n } from "../i18n/I18nContext";
import { BdiText } from "./BdiText";

export interface BoxItem {
  name: string;
  path: string;
  is_dir: boolean;
  size_bytes: number;
  modified_timestamp: number;
  extension?: string;
}

export type FolderStatus = "Accessible" | "MissingOrDisconnected" | "PermissionDenied";

export interface FolderStateInfo {
  status: FolderStatus;
  items?: BoxItem[];
  path?: string;
  message?: string;
}

export interface DesktopBoxModel {
  id?: number;
  name: string;
  folder_path: string;
  display_mode: "grid" | "list";
  is_capsule_collapsed: boolean;
  sort_order: "name" | "type" | "date" | "size";
  position_x: number;
  position_y: number;
  width: number;
  height: number;
  display_id?: string;
  group_id?: number;
}

export interface DesktopBoxProps {
  box: DesktopBoxModel;
  folderState: FolderStateInfo;
  onToggleCapsule: (boxId?: number) => void;
  onChangeDisplayMode: (boxId: number | undefined, mode: "grid" | "list") => void;
  onChangeSortOrder: (boxId: number | undefined, sort: "name" | "type" | "date" | "size") => void;
  onDeleteBox?: (boxId?: number) => void;
}

export function DesktopBox({
  box,
  folderState,
  onToggleCapsule,
  onChangeDisplayMode,
  onChangeSortOrder,
  onDeleteBox,
}: DesktopBoxProps): React.ReactElement {
  const { t } = useI18n();

  const isCollapsed = box.is_capsule_collapsed;
  const items = folderState.items ?? [];
  const isDisconnected = folderState.status === "MissingOrDisconnected";
  const isPermissionDenied = folderState.status === "PermissionDenied";

  // ── Capsule Mode (Collapsed State) ──────────────────────────────────
  if (isCollapsed) {
    return (
      <div
        className="rawaq-capsule-bar"
        data-testid={`capsule-bar-${box.id ?? 0}`}
        role="region"
        aria-label={box.name}
      >
        <div className="rawaq-capsule-info">
          <span className="rawaq-capsule-icon" aria-hidden="true">
            🗂️
          </span>
          <span className="rawaq-capsule-title">
            <BdiText text={box.name} testId={`capsule-title-${box.id ?? 0}`} />
          </span>
          <span className="rawaq-capsule-badge" data-testid={`capsule-badge-${box.id ?? 0}`}>
            {isDisconnected ? "⚠️" : t("boxes.itemCount", { count: items.length })}
          </span>
        </div>

        <button
          type="button"
          className="rawaq-capsule-expand-btn"
          data-testid={`capsule-expand-btn-${box.id ?? 0}`}
          onClick={() => onToggleCapsule(box.id)}
          title={t("boxes.capsuleExpand")}
          aria-expanded="false"
        >
          {t("boxes.capsuleExpand")} ⤢
        </button>
      </div>
    );
  }

  // ── Expanded Box Mode ───────────────────────────────────────────────
  return (
    <article
      className="rawaq-box"
      data-testid={`desktop-box-${box.id ?? 0}`}
      role="region"
      aria-label={box.name}
    >
      {/* Box Header */}
      <header className="rawaq-box-header">
        <div className="rawaq-box-title-area">
          <span className="rawaq-box-icon" aria-hidden="true">
            📂
          </span>
          <h3 className="rawaq-box-title">
            <BdiText text={box.name} testId={`box-title-${box.id ?? 0}`} />
          </h3>
          <span className="rawaq-box-badge">
            {t("boxes.itemCount", { count: items.length })}
          </span>
        </div>

        <div className="rawaq-box-actions">
          {/* Display Mode Toggle */}
          <button
            type="button"
            className={`rawaq-icon-btn ${box.display_mode === "grid" ? "active" : ""}`}
            data-testid={`mode-grid-btn-${box.id ?? 0}`}
            onClick={() => onChangeDisplayMode(box.id, "grid")}
            title={t("boxes.modeGrid")}
            aria-pressed={box.display_mode === "grid"}
          >
            ▦
          </button>
          <button
            type="button"
            className={`rawaq-icon-btn ${box.display_mode === "list" ? "active" : ""}`}
            data-testid={`mode-list-btn-${box.id ?? 0}`}
            onClick={() => onChangeDisplayMode(box.id, "list")}
            title={t("boxes.modeList")}
            aria-pressed={box.display_mode === "list"}
          >
            ☰
          </button>

          {/* Sort Selector */}
          <select
            className="rawaq-sort-select"
            data-testid={`sort-select-${box.id ?? 0}`}
            value={box.sort_order}
            onChange={(e) =>
              onChangeSortOrder(
                box.id,
                e.target.value as "name" | "type" | "date" | "size",
              )
            }
          >
            <option value="name">{t("boxes.sortByName")}</option>
            <option value="type">{t("boxes.sortByType")}</option>
            <option value="date">{t("boxes.sortByDate")}</option>
            <option value="size">{t("boxes.sortBySize")}</option>
          </select>

          {/* Capsule Collapse Button */}
          <button
            type="button"
            className="rawaq-icon-btn"
            data-testid={`capsule-collapse-btn-${box.id ?? 0}`}
            onClick={() => onToggleCapsule(box.id)}
            title={t("boxes.capsuleCollapse")}
            aria-expanded="true"
          >
            ―
          </button>

          {onDeleteBox && (
            <button
              type="button"
              className="rawaq-icon-btn danger"
              data-testid={`delete-box-btn-${box.id ?? 0}`}
              onClick={() => onDeleteBox(box.id)}
              title="Delete Box"
            >
              ✕
            </button>
          )}
        </div>
      </header>

      {/* Disconnected / Missing Folder Warning Banner */}
      {isDisconnected && (
        <div
          className="rawaq-box-banner warning"
          data-testid={`disconnected-banner-${box.id ?? 0}`}
          role="alert"
        >
          <div className="banner-title">⚠️ {t("boxes.disconnectedTitle")}</div>
          <div className="banner-desc">
            {t("boxes.disconnectedDesc", { path: "" })}
            <BdiText text={box.folder_path} testId={`disconnected-path-${box.id ?? 0}`} />
          </div>
        </div>
      )}

      {/* Permission Denied Warning Banner */}
      {isPermissionDenied && (
        <div
          className="rawaq-box-banner error"
          data-testid={`permission-banner-${box.id ?? 0}`}
          role="alert"
        >
          <div>🚫 {folderState.message ?? "Permission denied"}</div>
        </div>
      )}

      {/* Box Contents (Grid or List View) */}
      {!isDisconnected && !isPermissionDenied && (
        <div
          className={`rawaq-box-content ${box.display_mode}`}
          data-testid={`box-content-${box.id ?? 0}`}
        >
          {items.map((item, idx) => (
            <div
              key={item.path}
              className="rawaq-box-item"
              data-testid={`box-item-${box.id ?? 0}-${idx}`}
            >
              <span className="item-icon" aria-hidden="true">
                {item.is_dir ? "📁" : getFileEmoji(item.extension)}
              </span>
              <span className="item-name">
                <BdiText
                  text={item.name}
                  testId={`box-item-name-${box.id ?? 0}-${idx}`}
                />
              </span>
              {box.display_mode === "list" && (
                <span className="item-meta">
                  {item.is_dir ? "مجلد" : formatFileSize(item.size_bytes)}
                </span>
              )}
            </div>
          ))}
        </div>
      )}
    </article>
  );
}

function getFileEmoji(extension?: string): string {
  switch (extension) {
    case "pdf":
      return "📕";
    case "docx":
    case "doc":
    case "txt":
    case "md":
      return "📄";
    case "png":
    case "jpg":
    case "jpeg":
    case "svg":
      return "🖼️";
    case "zip":
    case "tar":
    case "gz":
      return "📦";
    case "rs":
    case "ts":
    case "tsx":
    case "js":
      return "💻";
    default:
      return "📝";
  }
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
