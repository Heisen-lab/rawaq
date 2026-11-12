import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TopCapsule, QuickShelfItemUI, ClipboardSuggestionUI } from "../components/TopCapsule";
import { I18nProvider } from "../i18n/I18nContext";

describe("Stage 5 Living Island & Top Capsule UI", () => {
  const mockShelfItems: QuickShelfItemUI[] = [
    {
      id: "shelf-1",
      original_path: "/home/user/Desktop/تقرير_الميزانية.pdf",
      file_name: "تقرير_الميزانية.pdf",
      file_size: 4096,
      is_accessible: true,
      is_directory: false,
      extension: "pdf",
      added_at_utc: "2026-11-12T10:00:00Z",
    },
    {
      id: "shelf-2",
      original_path: "/home/user/Desktop/design_v2.png",
      file_name: "design_v2.png",
      file_size: 8192,
      is_accessible: true,
      is_directory: false,
      extension: "png",
      added_at_utc: "2026-11-12T10:01:00Z",
    },
  ];

  it("renders TopCapsule in expanded mode with Quick Shelf in Arabic (RTL)", () => {
    const onClearShelf = vi.fn();
    const onRemoveShelfItem = vi.fn();

    render(
      <I18nProvider initialLocale="ar">
        <TopCapsule
          visibility="visible"
          shelfItems={mockShelfItems}
          onClearShelf={onClearShelf}
          onRemoveShelfItem={onRemoveShelfItem}
        />
      </I18nProvider>
    );

    // Title and items
    expect(screen.getByText("كبسولة رَوَاق التفاعلية")).toBeDefined();
    expect(screen.getByTestId("shelf-panel")).toBeDefined();
    expect(screen.getByText("تقرير_الميزانية.pdf")).toBeDefined();
    expect(screen.getByText("design_v2.png")).toBeDefined();

    // Remove single item
    const removeBtn = screen.getByTestId("remove-shelf-1");
    fireEvent.click(removeBtn);
    expect(onRemoveShelfItem).toHaveBeenCalledWith("shelf-1");

    // Clear all
    const clearBtn = screen.getByTestId("clear-shelf-btn");
    fireEvent.click(clearBtn);
    expect(onClearShelf).toHaveBeenCalled();
  });

  it("renders TopCapsule in collapsed pill mode and toggles expansion", () => {
    const onToggleCollapse = vi.fn();

    render(
      <I18nProvider initialLocale="ar">
        <TopCapsule
          visibility="collapsed"
          shelfItems={mockShelfItems}
          onToggleCollapse={onToggleCollapse}
        />
      </I18nProvider>
    );

    const pill = screen.getByTestId("top-capsule-collapsed");
    expect(pill).toBeDefined();
    expect(screen.getByTestId("shelf-badge").textContent).toContain("2");

    const toggleBtn = screen.getByRole("button", { name: "تبديل وضع الكبسولة" });
    fireEvent.click(toggleBtn);
    expect(onToggleCollapse).toHaveBeenCalled();
  });

  it("renders fullscreen auto-hide notice when isFullscreen is active", () => {
    render(
      <I18nProvider initialLocale="ar">
        <TopCapsule
          visibility="visible"
          isFullscreen={true}
          shelfItems={mockShelfItems}
        />
      </I18nProvider>
    );

    expect(screen.getByRole("status")).toBeDefined();
    expect(screen.getByText("وضع ملء الشاشة مفعّل (الكبسولة مخفية تلقائيًا)")).toBeDefined();
    expect(screen.queryByTestId("top-capsule-expanded")).toBeNull();
  });

  it("handles Smart Clipboard suggestions: actionable URL vs rejected malicious content", () => {
    const safeSuggestion: ClipboardSuggestionUI = {
      kind: "safe_url",
      raw_preview: "https://rawaq.app/docs",
      sanitized_content: "https://rawaq.app/docs",
      title: "رابط: rawaq.app",
      is_actionable: true,
    };

    const onSave = vi.fn();
    const onDismiss = vi.fn();

    const { rerender } = render(
      <I18nProvider initialLocale="ar">
        <TopCapsule
          visibility="visible"
          shelfItems={[]}
          clipboardSuggestion={safeSuggestion}
          onSaveClipboardNote={onSave}
          onDismissClipboard={onDismiss}
        />
      </I18nProvider>
    );

    // Switch to clipboard tab
    const clipboardTab = screen.getByTestId("tab-clipboard");
    fireEvent.click(clipboardTab);

    expect(screen.getByText("رابط: rawaq.app")).toBeDefined();
    expect(screen.getByText("https://rawaq.app/docs")).toBeDefined();

    // Click Save
    const saveBtn = screen.getByTestId("clipboard-save-btn");
    fireEvent.click(saveBtn);
    expect(onSave).toHaveBeenCalledWith(safeSuggestion);

    // Rerender with RejectedDangerous suggestion
    const rejectedSuggestion: ClipboardSuggestionUI = {
      kind: "rejected_dangerous",
      raw_preview: "javascript:alert(1)",
      sanitized_content: "",
      title: "محتوى غير آمن تم حظره",
      is_actionable: false,
      rejection_reason: "تم حظر المخطط غير المصرح به",
    };

    rerender(
      <I18nProvider initialLocale="ar">
        <TopCapsule
          visibility="visible"
          shelfItems={[]}
          clipboardSuggestion={rejectedSuggestion}
        />
      </I18nProvider>
    );

    // Ensure rejection alert is visible
    expect(screen.getByTestId("clipboard-rejected-alert")).toBeDefined();
    expect(screen.getByText("تم حظر المخطط غير المصرح به")).toBeDefined();
    expect(screen.queryByTestId("clipboard-save-btn")).toBeNull();
  });
});
