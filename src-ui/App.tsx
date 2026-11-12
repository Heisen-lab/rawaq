import React, { useState } from "react";
import { useI18n } from "./i18n/I18nContext";
import { ThemeProvider } from "./theme/ThemeContext";
import { ThemeCustomizerBar } from "./components/ThemeCustomizerBar";
import { TopCapsule } from "./components/TopCapsule";
import { ProductivityHub } from "./components/ProductivityHub";
import { MultiMonitorHub } from "./components/MultiMonitorHub";
import { OrganizerPreview } from "./components/OrganizerPreview";
import { BdiText } from "./components/BdiText";
import { sanitizeInputText, validateSafePath } from "./security/sanitizer";

const SAMPLE_MIXED_FILES = [
  "تقرير_الميزانية_2026_Final(1).pdf",
  "C:\\Users\\أحمد\\Desktop\\مشروع_Rawaq\\notes_v2.md",
  "/home/salma/Desktop/ملخص_البحث_AI_2026.docx",
];

export function AppContent({
  initialTab = "overview",
}: {
  initialTab?: "productivity" | "displays" | "organizer" | "overview";
} = {}): React.ReactElement {
  const { locale, direction, toggleLocale, setLocale, t } = useI18n();
  const [activeTab, setActiveTab] = useState<"productivity" | "displays" | "organizer" | "overview">(initialTab);

  const samplePathCheck = validateSafePath(
    "/Users/rawaq/Desktop/مجلد_العمل_2026",
  );
  const sanitizedNote = sanitizeInputText(
    "ملاحظة سريعة: مراجعة ملف Rawaq <script>alert('xss')</script> الساعة 10:00",
  );

  const sampleOrganizerPlan = {
    plan_id: "plan_demo",
    source_folder: "/Users/rawaq/Desktop",
    moves: [
      {
        source_path: "/Users/rawaq/Desktop/تقرير_الميزانية_2026.pdf",
        original_destination_path: "/Users/rawaq/Documents/Financial/تقرير_الميزانية_2026.pdf",
        resolved_destination_path: "/Users/rawaq/Documents/Financial/تقرير_الميزانية_2026.pdf",
        file_name: "تقرير_الميزانية_2026.pdf",
        file_size: 1048576,
        file_hash: "a1b2c3d4e5f67890",
        rule_name: "مستندات مالية",
        collision_detected: false,
      },
    ],
    skipped: [],
  };

  return (
    <main className="rawaq-shell" data-testid="app-shell">
      {/* Top Capsule / Living Island HUD */}
      <TopCapsule visibility="collapsed" />

      {/* Header and Controls */}
      <header className="rawaq-header">
        <div>
          <h1 data-testid="app-title">{t("app.title")}</h1>
          <p data-testid="app-subtitle">{t("app.subtitle")}</p>
          <p data-testid="app-welcome">
            {t("app.welcome", { name: locale === "ar" ? "أحمد" : "Ahmad" })}
          </p>
        </div>

        <div>
          <button
            type="button"
            className="rawaq-button"
            data-testid="toggle-language-btn"
            onClick={toggleLocale}
          >
            {t("language.switchAction")}
          </button>
          <div style={{ marginBlockStart: "0.5rem" }}>
            <button
              type="button"
              data-testid="set-ar-btn"
              onClick={() => setLocale("ar")}
              disabled={locale === "ar"}
            >
              {t("language.arabic")}
            </button>
            <button
              type="button"
              data-testid="set-en-btn"
              onClick={() => setLocale("en")}
              disabled={locale === "en"}
              style={{ marginInlineStart: "0.5rem" }}
            >
              {t("language.english")}
            </button>
          </div>
          <p data-testid="active-direction-indicator">
            {t("language.currentDirection", {
              direction: direction.toUpperCase(),
            })}
          </p>
        </div>
      </header>

      {/* Multi-Style Theme Customizer Bar */}
      <ThemeCustomizerBar />

      {/* Workspace Tabs Navigation */}
      <nav className="workspace-tabs" data-testid="workspace-tabs">
        <button
          type="button"
          className={`workspace-tab-btn ${activeTab === "productivity" ? "is-active" : ""}`}
          onClick={() => setActiveTab("productivity")}
          data-testid="tab-productivity-btn"
        >
          <span>⚡</span>
          <span>{t("workspace.tabProductivity")}</span>
        </button>
        <button
          type="button"
          className={`workspace-tab-btn ${activeTab === "displays" ? "is-active" : ""}`}
          onClick={() => setActiveTab("displays")}
          data-testid="tab-displays-btn"
        >
          <span>🖥️</span>
          <span>{t("workspace.tabDisplays")}</span>
        </button>
        <button
          type="button"
          className={`workspace-tab-btn ${activeTab === "organizer" ? "is-active" : ""}`}
          onClick={() => setActiveTab("organizer")}
          data-testid="tab-organizer-btn"
        >
          <span>🧹</span>
          <span>{t("workspace.tabOrganizer")}</span>
        </button>
        <button
          type="button"
          className={`workspace-tab-btn ${activeTab === "overview" ? "is-active" : ""}`}
          onClick={() => setActiveTab("overview")}
          data-testid="tab-overview-btn"
        >
          <span>🛡️</span>
          <span>{t("workspace.tabOverview")}</span>
        </button>
      </nav>

      {/* Active Tab Views */}
      <section className="workspace-content-pane">
        {activeTab === "productivity" && <ProductivityHub />}
        {activeTab === "displays" && <MultiMonitorHub />}
        {activeTab === "organizer" && (
          <OrganizerPreview
            plan={sampleOrganizerPlan}
            onExecute={() => {}}
            executionReport={null}
            onUndo={() => {}}
            undoReport={null}
          />
        )}
        {activeTab === "overview" && (
          <div className="overview-sections-container">
            <section className="rawaq-card" aria-labelledby="bidi-preview-heading">
              <h2 id="bidi-preview-heading">{t("bidi.previewTitle")}</h2>
              <p>{t("bidi.description")}</p>
              <p data-testid="sample-file-count">
                {t("bidi.sampleFileCount", {
                  count: SAMPLE_MIXED_FILES.length,
                  name: locale === "ar" ? "سلمى" : "Salma",
                })}
              </p>
              <ul className="rawaq-file-list" data-testid="mixed-files-list">
                {SAMPLE_MIXED_FILES.map((file, idx) => (
                  <li key={file} className="rawaq-file-item">
                    <BdiText text={file} testId={`mixed-file-${idx}`} />
                  </li>
                ))}
              </ul>
            </section>

            <section className="rawaq-card" aria-labelledby="security-gate-heading">
              <h2 id="security-gate-heading">{t("security.gateTitle")}</h2>
              <p data-testid="security-path-status">
                {samplePathCheck.valid
                  ? t("security.pathValid", {
                      path: samplePathCheck.normalizedPath ?? "",
                    })
                  : t("security.pathRejected")}
              </p>
              <p data-testid="security-protected-count">
                {t("security.filesProtectedCount", {
                  count: SAMPLE_MIXED_FILES.length,
                })}
              </p>
              <p data-testid="sanitized-note-preview">
                <BdiText text={sanitizedNote} />
              </p>
            </section>
          </div>
        )}
      </section>
    </main>
  );
}

export function App({
  initialTab = "overview",
}: {
  initialTab?: "productivity" | "displays" | "organizer" | "overview";
} = {}): React.ReactElement {
  return (
    <ThemeProvider>
      <AppContent initialTab={initialTab} />
    </ThemeProvider>
  );
}
