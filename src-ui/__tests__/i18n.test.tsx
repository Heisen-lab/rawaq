import React from "react";
import { describe, expect, it, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import fs from "node:fs";
import path from "node:path";
import arCatalog from "../i18n/ar.json";
import enCatalog from "../i18n/en.json";
import {
  extractPlaceholders,
  flattenTranslations,
  formatTranslation,
  verifyTranslationParity,
} from "../i18n/parity";
import { I18nProvider, useI18n } from "../i18n/I18nContext";
import { App } from "../App";

describe("Stage 1 BiDi i18n Engine & RTL/LTR Switching", () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.lang = "ar";
    document.documentElement.dir = "rtl";
  });

  it("i18n::test_arabic_english_key_and_placeholder_parity", () => {
    const report = verifyTranslationParity(arCatalog, enCatalog);
    expect(report.missingInArabic).toEqual([]);
    expect(report.missingInEnglish).toEqual([]);
    expect(report.emptyValueKeys).toEqual([]);
    expect(report.placeholderMismatches).toEqual([]);
    expect(report.isValid).toBe(true);

    // Verify placeholders `{name}`, `{count}`, `{path}` are present and identical
    const arFlat = flattenTranslations(arCatalog);
    const enFlat = flattenTranslations(enCatalog);
    expect(extractPlaceholders(arFlat["app.welcome"] ?? "")).toEqual(["name"]);
    expect(extractPlaceholders(enFlat["app.welcome"] ?? "")).toEqual(["name"]);
    expect(extractPlaceholders(arFlat["bidi.sampleFileCount"] ?? "")).toEqual([
      "count",
      "name",
    ]);
    expect(extractPlaceholders(enFlat["bidi.sampleFileCount"] ?? "")).toEqual([
      "count",
      "name",
    ]);
  });

  it("i18n::test_arabic_english_key_and_placeholder_parity fails immediately when a key or placeholder is missing (Error Path)", () => {
    const brokenEn = structuredClone(enCatalog) as Record<string, unknown>;
    delete (brokenEn["app"] as Record<string, unknown>)["subtitle"];
    (brokenEn["app"] as Record<string, unknown>)["welcome"] =
      "Welcome without placeholder";
    (brokenEn["language"] as Record<string, unknown>)["arabic"] = "   ";

    const report = verifyTranslationParity(arCatalog, brokenEn);
    expect(report.isValid).toBe(false);
    expect(report.missingInEnglish).toContain("app.subtitle");
    expect(report.emptyValueKeys).toContain("en:language.arabic");
    expect(report.placeholderMismatches).toEqual([
      {
        key: "app.welcome",
        arPlaceholders: ["name"],
        enPlaceholders: [],
      },
    ]);
  });

  it("i18n::test_runtime_direction_switch_rtl_ltr", () => {
    render(
      <I18nProvider initialLocale="ar">
        <App />
      </I18nProvider>,
    );

    const bidiRoot = screen.getByTestId("bidi-root");
    expect(bidiRoot).toHaveAttribute("dir", "rtl");
    expect(bidiRoot).toHaveAttribute("lang", "ar");
    expect(document.documentElement.dir).toBe("rtl");
    expect(document.documentElement.lang).toBe("ar");
    expect(screen.getByTestId("app-title")).toHaveTextContent("رَوَاق");

    // Verify mixed-direction filenames are isolated inside <bdi dir="auto">
    const mixedFile0 = screen.getByTestId("mixed-file-0");
    expect(mixedFile0.tagName.toLowerCase()).toBe("bdi");
    expect(mixedFile0).toHaveAttribute("dir", "auto");
    expect(mixedFile0).toHaveTextContent("تقرير_الميزانية_2026_Final(1).pdf");

    // Switch at runtime to English (LTR) without reload
    const toggleBtn = screen.getByTestId("toggle-language-btn");
    fireEvent.click(toggleBtn);

    expect(bidiRoot).toHaveAttribute("dir", "ltr");
    expect(bidiRoot).toHaveAttribute("lang", "en");
    expect(document.documentElement.dir).toBe("ltr");
    expect(document.documentElement.lang).toBe("en");
    expect(screen.getByTestId("app-title")).toHaveTextContent("Rawaq");
    expect(screen.getByTestId("active-direction-indicator")).toHaveTextContent(
      "Active direction: LTR",
    );

    // Mixed file remains safely isolated inside <bdi dir="auto"> after switching
    expect(screen.getByTestId("mixed-file-1").tagName.toLowerCase()).toBe("bdi");
    expect(screen.getByTestId("mixed-file-1")).toHaveAttribute("dir", "auto");

    // Switch back to Arabic (RTL)
    fireEvent.click(screen.getByTestId("set-ar-btn"));
    expect(bidiRoot).toHaveAttribute("dir", "rtl");
    expect(document.documentElement.dir).toBe("rtl");
    expect(screen.getByTestId("app-title")).toHaveTextContent("رَوَاق");
  });

  it("handles edge cases in translation formatting, unknown keys, and non-string values", () => {
    expect(
      formatTranslation("Hello {name}, you have {count} items and {missing}", {
        name: "أحمد",
        count: 0,
      }),
    ).toBe("Hello أحمد, you have 0 items and {missing}");

    expect(() =>
      flattenTranslations({ invalidKey: 12345 as unknown as string }),
    ).toThrow(/Invalid non-string translation value/);

    // Verify useI18n throws a clear error if used outside provider
    function OrphanConsumer(): React.ReactElement {
      const { t } = useI18n();
      return <span>{t("app.title")}</span>;
    }
    expect(() => render(<OrphanConsumer />)).toThrow(
      /useI18n must be used within an I18nProvider/,
    );
  });

  it("enforces BiDi CSS Logical Properties and forbids physical left/right layout rules in bidi.css", () => {
    const cssPath = path.resolve(process.cwd(), "src-ui/styles/bidi.css");
    const cssContent = fs.readFileSync(cssPath, "utf-8");

    // Strip CSS comments before checking property declarations
    const withoutComments = cssContent.replace(/\/\*[\s\S]*?\*\//g, "");
    expect(withoutComments).not.toMatch(
      /\b(margin-left|margin-right|padding-left|padding-right|border-left|border-right|text-align\s*:\s*(left|right))\b/i,
    );
    expect(withoutComments).toMatch(/border-inline-start/);
    expect(withoutComments).toMatch(/padding-inline/);
    expect(withoutComments).toMatch(/text-align:\s*start/);
  });
});
