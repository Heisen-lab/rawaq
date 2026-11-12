export interface PathValidationResult {
  valid: boolean;
  normalizedPath?: string;
  reason?: string;
}

const MAX_PATH_LENGTH = 4096;
const MAX_COMPONENT_LENGTH = 255;

const WINDOWS_RESERVED_NAMES = new Set([
  "CON",
  "PRN",
  "AUX",
  "NUL",
  "COM1",
  "COM2",
  "COM3",
  "COM4",
  "COM5",
  "COM6",
  "COM7",
  "COM8",
  "COM9",
  "LPT1",
  "LPT2",
  "LPT3",
  "LPT4",
  "LPT5",
  "LPT6",
  "LPT7",
  "LPT8",
  "LPT9",
]);

const FORBIDDEN_SYSTEM_PREFIXES = [
  "/etc",
  "/proc",
  "/sys",
  "/dev",
  "/boot",
  "/root",
  "/system/library",
  "/private/etc",
  "c:/windows",
  "c:/program files",
  "c:/program files (x86)",
  "c:/system volume information",
];

/**
 * Validates a file or directory path against Path Traversal (`..`), null bytes,
 * control characters, URL-encoded traversal sequences, Windows reserved names,
 * and sensitive OS system directories.
 */
export function validateSafePath(
  rawPath: string,
  allowedRoots?: string[],
): PathValidationResult {
  if (typeof rawPath !== "string") {
    return { valid: false, reason: "PATH_NOT_STRING" };
  }

  // Reject null bytes or ASCII control characters (0x00 - 0x1F, 0x7F) before trimming
  for (let i = 0; i < rawPath.length; i++) {
    const code = rawPath.charCodeAt(i);
    if (code <= 0x1f || code === 0x7f) {
      return { valid: false, reason: "CONTROL_OR_NULL_BYTE_DETECTED" };
    }
  }

  const trimmed = rawPath.trim();
  if (trimmed.length === 0) {
    return { valid: false, reason: "EMPTY_PATH" };
  }

  if (trimmed.length > MAX_PATH_LENGTH) {
    return { valid: false, reason: "PATH_TOO_LONG" };
  }

  // Reject URL-encoded traversal or null byte tricks (%2e, %2f, %5c, %00)
  const lowerRaw = trimmed.toLowerCase();
  if (
    lowerRaw.includes("%2e") ||
    lowerRaw.includes("%2f") ||
    lowerRaw.includes("%5c") ||
    lowerRaw.includes("%00")
  ) {
    return { valid: false, reason: "ENCODED_TRAVERSAL_SEQUENCE" };
  }

  // Reject UNC network paths (\\server\share or //server/share)
  if (trimmed.startsWith("\\\\") || trimmed.startsWith("//")) {
    return { valid: false, reason: "UNC_NETWORK_PATH_REJECTED" };
  }

  // Normalize slashes to `/` for uniform inspection across Windows/macOS/Linux
  const normalized = trimmed.replace(/\\/g, "/");
  const segments = normalized.split("/").filter((seg) => seg.length > 0);

  if (segments.length === 0) {
    return { valid: false, reason: "ROOT_ONLY_PATH_REJECTED" };
  }

  for (let idx = 0; idx < segments.length; idx++) {
    const seg = segments[idx]!;
    if (seg === "." || seg === "..") {
      return { valid: false, reason: "RELATIVE_TRAVERSAL_SEGMENT" };
    }
    if (seg.includes("..")) {
      return { valid: false, reason: "SUSPICIOUS_DOT_SEQUENCE" };
    }
    if (seg.length > MAX_COMPONENT_LENGTH) {
      return { valid: false, reason: "COMPONENT_TOO_LONG" };
    }

    // Check Windows reserved device names (ignoring drive letter like C:)
    if (!(idx === 0 && /^[a-zA-Z]:$/.test(seg))) {
      const stem = seg.split(".")[0]?.toUpperCase() ?? "";
      if (WINDOWS_RESERVED_NAMES.has(stem)) {
        return { valid: false, reason: "RESERVED_DEVICE_NAME" };
      }
    }
  }

  const canonicalLower = normalized.replace(/\/+$/, "").toLowerCase();
  for (const prefix of FORBIDDEN_SYSTEM_PREFIXES) {
    if (
      canonicalLower === prefix ||
      canonicalLower.startsWith(`${prefix}/`)
    ) {
      return { valid: false, reason: "SYSTEM_DIRECTORY_FORBIDDEN" };
    }
  }

  // If allowedRoots are specified, verify the path is strictly inside at least one root
  if (allowedRoots && allowedRoots.length > 0) {
    const insideAllowedRoot = allowedRoots.some((root) => {
      const normRoot = root.trim().replace(/\\/g, "/").replace(/\/+$/, "").toLowerCase();
      if (!normRoot) return false;
      return (
        canonicalLower === normRoot ||
        canonicalLower.startsWith(`${normRoot}/`)
      );
    });

    if (!insideAllowedRoot) {
      return { valid: false, reason: "OUTSIDE_AUTHORIZED_SCOPE" };
    }
  }

  return {
    valid: true,
    normalizedPath: normalized.replace(/\/+/g, "/"),
  };
}

/**
 * Sanitizes user or clipboard text (notes, task titles, descriptions) by completely removing
 * executable HTML blocks (`<script>`, `<iframe>`, `<object>`, `<embed>`, `<style>`, `<svg>`, `<math>`),
 * stripping raw HTML tags, removing inline event handlers (`onerror=`, `onload=`, etc.),
 * and neutralizing dangerous URI schemes (`javascript:`, `vbscript:`, `data:text/html`),
 * while preserving valid Arabic, English, numbers, and punctuation intact.
 */
export function sanitizeInputText(input: string): string {
  if (typeof input !== "string" || input.length === 0) {
    return "";
  }

  let cleaned = input.replace(/\0/g, "");

  // 1. Remove paired dangerous tags and their entire inner content
  const dangerousBlockRegex =
    /<\s*(script|iframe|object|embed|style|svg|math|applet|frame|frameset)\b[^>]*>[\s\S]*?<\s*\/\s*\1\s*>/gi;
  cleaned = cleaned.replace(dangerousBlockRegex, "");

  // 2. Remove any remaining unclosed or self-closing HTML tags
  cleaned = cleaned.replace(/<\/?[a-zA-Z!/?][^>]*>/g, "");

  // 3. Remove standalone inline event handlers if injected outside brackets (e.g. `onerror=alert(1)`)
  cleaned = cleaned.replace(
    /\bon[a-zA-Z]+\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)/gi,
    "",
  );

  // 4. Neutralize dangerous protocol schemes (`javascript:`, `vbscript:`, `data:text/html`)
  cleaned = cleaned.replace(
    /\b(?:javascript|vbscript)\s*:[^\s]*/gi,
    "",
  );
  cleaned = cleaned.replace(/\bdata\s*:\s*text\/html[^\s]*/gi, "");

  return cleaned.trim();
}

/**
 * Validates and sanitizes a captured URI (e.g. from Clipboard Quick Capture),
 * allowing only safe `https:` and `http:` URLs and blocking `javascript:`, `data:`, `file:`, `vbscript:`.
 */
export function sanitizeClipboardUri(rawUri: string): string | null {
  if (typeof rawUri !== "string") {
    return null;
  }
  const trimmed = rawUri.trim();
  if (!trimmed || trimmed.includes("\0")) {
    return null;
  }
  try {
    const parsed = new URL(trimmed);
    const protocol = parsed.protocol.toLowerCase();
    if (protocol !== "https:" && protocol !== "http:") {
      return null;
    }
    return parsed.toString();
  } catch {
    return null;
  }
}
