import { useState } from "react";
import { useI18n } from "../i18n/I18nContext";
import { BdiText } from "./BdiText";

export interface QuickShelfItemUI {
  id: string;
  original_path: string;
  file_name: string;
  file_size: number;
  is_accessible: boolean;
  is_directory: boolean;
  extension: string;
  added_at_utc: string;
}

export type CapsuleVisibilityUI = "visible" | "collapsed" | "hidden_fullscreen";

export type ClipboardContentKindUI =
  | "safe_url"
  | "email_address"
  | "plain_text"
  | "rejected_dangerous";

export interface ClipboardSuggestionUI {
  kind: ClipboardContentKindUI;
  raw_preview: string;
  sanitized_content: string;
  title: string;
  is_actionable: boolean;
  rejection_reason?: string | null;
}

export interface TopCapsuleProps {
  visibility?: CapsuleVisibilityUI;
  isFullscreen?: boolean;
  shelfItems?: QuickShelfItemUI[];
  clipboardSuggestion?: ClipboardSuggestionUI | null;
  optInClipboard?: boolean;
  onToggleCollapse?: () => void;
  onClearShelf?: () => void;
  onRemoveShelfItem?: (id: string) => void;
  onSaveClipboardNote?: (suggestion: ClipboardSuggestionUI) => void;
  onDismissClipboard?: () => void;
  onToggleOptIn?: (enabled: boolean) => void;
}

export function TopCapsule({
  visibility = "visible",
  isFullscreen = false,
  shelfItems = [],
  clipboardSuggestion = null,
  optInClipboard = true,
  onToggleCollapse,
  onClearShelf,
  onRemoveShelfItem,
  onSaveClipboardNote,
  onDismissClipboard,
  onToggleOptIn,
}: TopCapsuleProps) {
  const { t } = useI18n();
  const [activeTab, setActiveTab] = useState<"shelf" | "clipboard">("shelf");

  // When an app enters fullscreen, the capsule is automatically hidden to not interfere
  if (isFullscreen || visibility === "hidden_fullscreen") {
    return (
      <aside
        className="top-capsule-hidden-notice"
        role="status"
        aria-label="fullscreen-status"
      >
        <span className="capsule-fs-tag">{t("island.fullscreenNotice")}</span>
      </aside>
    );
  }

  // Collapsed Mode (Sleek floating pill)
  if (visibility === "collapsed") {
    return (
      <header className="top-capsule-container collapsed" data-testid="top-capsule-collapsed">
        <div className="top-capsule-pill" onClick={onToggleCollapse} role="button" tabIndex={0}>
          <span className="pill-brand">رَوَاق</span>
          <span className="pill-badge" data-testid="shelf-badge">
            📁 {shelfItems.length}
          </span>
          {clipboardSuggestion && clipboardSuggestion.is_actionable && (
            <span className="pill-badge-alert" data-testid="clipboard-badge">
              📋 {t("island.clipboardTitle")}
            </span>
          )}
          <button
            type="button"
            className="capsule-toggle-btn"
            onClick={(e) => {
              e.stopPropagation();
              onToggleCollapse?.();
            }}
            aria-label={t("island.toggleCapsule")}
          >
            ▼
          </button>
        </div>
      </header>
    );
  }

  // Expanded Mode (Interactive Living Island)
  return (
    <header className="top-capsule-container expanded" data-testid="top-capsule-expanded">
      <div className="top-capsule-card">
        {/* Island Header Bar */}
        <div className="island-header">
          <div className="island-title-group">
            <span className="island-icon">🏝️</span>
            <h2 className="island-heading">{t("island.title")}</h2>
          </div>
          <div className="island-header-actions">
            <button
              type="button"
              className="capsule-toggle-btn"
              onClick={onToggleCollapse}
              data-testid="toggle-collapse-btn"
              aria-label={t("island.toggleCapsule")}
            >
              ▲
            </button>
          </div>
        </div>

        {/* Island Tabs */}
        <div className="island-tabs-bar" role="tablist">
          <button
            type="button"
            role="tab"
            aria-selected={activeTab === "shelf"}
            className={`island-tab-btn ${activeTab === "shelf" ? "active" : ""}`}
            onClick={() => setActiveTab("shelf")}
            data-testid="tab-shelf"
          >
            📁 {t("island.shelfTitle")} ({shelfItems.length})
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={activeTab === "clipboard"}
            className={`island-tab-btn ${activeTab === "clipboard" ? "active" : ""}`}
            onClick={() => setActiveTab("clipboard")}
            data-testid="tab-clipboard"
          >
            📋 {t("island.clipboardTitle")}
          </button>
        </div>

        {/* Tab 1: Quick Shelf */}
        {activeTab === "shelf" && (
          <div className="island-shelf-panel" data-testid="shelf-panel">
            <div className="shelf-actions-bar">
              <span className="shelf-count-text">
                {t("island.shelfCount", { count: shelfItems.length })}
              </span>
              {shelfItems.length > 0 && (
                <button
                  type="button"
                  className="shelf-clear-btn"
                  onClick={onClearShelf}
                  data-testid="clear-shelf-btn"
                >
                  {t("island.clearShelf")}
                </button>
              )}
            </div>

            {shelfItems.length === 0 ? (
              <div className="shelf-empty-box" data-testid="shelf-empty">
                <span className="shelf-empty-icon">📥</span>
                <p>{t("island.shelfEmpty")}</p>
              </div>
            ) : (
              <ul className="shelf-items-list" data-testid="shelf-list">
                {shelfItems.map((item) => (
                  <li key={item.id} className="shelf-item-row" data-testid={`shelf-item-${item.id}`}>
                    <div className="shelf-item-info">
                      <span className="shelf-item-icon">
                        {item.is_directory ? "📁" : "📄"}
                      </span>
                      <span className="shelf-item-name">
                        <BdiText text={item.file_name} />
                      </span>
                      <span className="shelf-item-size">
                        {item.file_size} B
                      </span>
                      {!item.is_accessible && (
                        <span className="shelf-item-missing-badge">غير متاح</span>
                      )}
                    </div>
                    <button
                      type="button"
                      className="shelf-item-remove-btn"
                      onClick={() => onRemoveShelfItem?.(item.id)}
                      data-testid={`remove-${item.id}`}
                      aria-label={`${t("island.removeItem")} ${item.file_name}`}
                    >
                      ✕
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {/* Tab 2: Smart Clipboard */}
        {activeTab === "clipboard" && (
          <div className="island-clipboard-panel" data-testid="clipboard-panel">
            <div className="clipboard-optin-bar">
              <label className="clipboard-optin-label">
                <input
                  type="checkbox"
                  checked={optInClipboard}
                  onChange={(e) => onToggleOptIn?.(e.target.checked)}
                  data-testid="optin-checkbox"
                />
                <span>{t("island.optInNotice")}</span>
              </label>
            </div>

            {clipboardSuggestion ? (
              <div
                className={`clipboard-card ${
                  clipboardSuggestion.kind === "rejected_dangerous" ? "rejected" : "actionable"
                }`}
                data-testid="clipboard-card"
              >
                <div className="clipboard-card-header">
                  <span className="clipboard-badge-type">
                    {clipboardSuggestion.kind === "safe_url" && "🔗 رابط ويب"}
                    {clipboardSuggestion.kind === "email_address" && "✉️ بريد إلكتروني"}
                    {clipboardSuggestion.kind === "plain_text" && "📝 نص ملاحظة"}
                    {clipboardSuggestion.kind === "rejected_dangerous" && "⚠️ محظور"}
                  </span>
                  <span className="clipboard-card-title">
                    <BdiText text={clipboardSuggestion.title} />
                  </span>
                </div>

                {clipboardSuggestion.kind === "rejected_dangerous" ? (
                  <div className="clipboard-rejection-alert" data-testid="clipboard-rejected-alert">
                    <strong>{t("island.clipboardRejected")}: </strong>
                    <span>{clipboardSuggestion.rejection_reason || "محتوى غير موثوق"}</span>
                  </div>
                ) : (
                  <>
                    <div className="clipboard-preview-box">
                      <BdiText text={clipboardSuggestion.sanitized_content} />
                    </div>
                    <div className="clipboard-actions-row">
                      <button
                        type="button"
                        className="btn-clipboard-save"
                        onClick={() => onSaveClipboardNote?.(clipboardSuggestion)}
                        data-testid="clipboard-save-btn"
                      >
                        {t("island.clipboardSave")}
                      </button>
                      <button
                        type="button"
                        className="btn-clipboard-dismiss"
                        onClick={onDismissClipboard}
                        data-testid="clipboard-dismiss-btn"
                      >
                        {t("island.clipboardDismiss")}
                      </button>
                    </div>
                  </>
                )}
              </div>
            ) : (
              <div className="clipboard-empty-state">
                <p>لا يوجد محتوى جديد في الحافظة</p>
              </div>
            )}
          </div>
        )}
      </div>
    </header>
  );
}
