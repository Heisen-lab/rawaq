use crate::core_domain::{Language, TextDirection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const AR_CATALOG_JSON: &str = include_str!("../../src-ui/i18n/ar.json");
const EN_CATALOG_JSON: &str = include_str!("../../src-ui/i18n/en.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceholderMismatch {
    pub key: String,
    pub ar_placeholders: Vec<String>,
    pub en_placeholders: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParityReport {
    pub is_valid: bool,
    pub missing_in_arabic: Vec<String>,
    pub missing_in_english: Vec<String>,
    pub empty_value_keys: Vec<String>,
    pub placeholder_mismatches: Vec<PlaceholderMismatch>,
}

/// Flattens a nested JSON object into dot-separated translation keys.
pub fn flatten_catalog(value: &Value, prefix: &str) -> Result<BTreeMap<String, String>, String> {
    let mut map = BTreeMap::new();
    let Some(obj) = value.as_object() else {
        return Err("Root translation catalog must be a JSON object".to_string());
    };

    for (key, val) in obj {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };

        match val {
            Value::Object(_) => {
                let nested = flatten_catalog(val, &full_key)?;
                map.extend(nested);
            }
            Value::String(s) => {
                map.insert(full_key, s.clone());
            }
            _ => {
                return Err(format!(
                    "Invalid non-string value at translation key '{full_key}'"
                ));
            }
        }
    }

    Ok(map)
}

/// Extracts sorted unique `{placeholder}` variable names from a translation template.
pub fn extract_placeholders(template: &str) -> Vec<String> {
    let mut set = BTreeSet::new();
    let chars: Vec<char> = template.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '{' {
            let mut end = i + 1;
            while end < chars.len() && chars[end] != '}' && chars[end] != '{' {
                end += 1;
            }
            if end < chars.len() && chars[end] == '}' && end > i + 1 {
                let token: String = chars[(i + 1)..end].iter().collect();
                if token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    set.insert(token);
                }
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }

    set.into_iter().collect()
}

/// Verifies 100% key and placeholder parity between two JSON translation strings.
pub fn verify_catalog_parity(ar_json: &str, en_json: &str) -> Result<ParityReport, String> {
    let ar_val: Value =
        serde_json::from_str(ar_json).map_err(|e| format!("Invalid Arabic JSON: {e}"))?;
    let en_val: Value =
        serde_json::from_str(en_json).map_err(|e| format!("Invalid English JSON: {e}"))?;

    let ar_flat = flatten_catalog(&ar_val, "")?;
    let en_flat = flatten_catalog(&en_val, "")?;

    let mut missing_in_arabic = Vec::new();
    let mut missing_in_english = Vec::new();
    let mut empty_value_keys = Vec::new();
    let mut placeholder_mismatches = Vec::new();

    for key in en_flat.keys() {
        if !ar_flat.contains_key(key) {
            missing_in_arabic.push(key.clone());
        }
    }

    for key in ar_flat.keys() {
        if !en_flat.contains_key(key) {
            missing_in_english.push(key.clone());
        }
    }

    let all_keys: BTreeSet<&String> = ar_flat.keys().chain(en_flat.keys()).collect();
    for key in all_keys {
        if let Some(ar_str) = ar_flat.get(key) {
            if ar_str.trim().is_empty() {
                empty_value_keys.push(format!("ar:{key}"));
            }
        }
        if let Some(en_str) = en_flat.get(key) {
            if en_str.trim().is_empty() {
                empty_value_keys.push(format!("en:{key}"));
            }
        }

        if let (Some(ar_str), Some(en_str)) = (ar_flat.get(key), en_flat.get(key)) {
            let ar_ph = extract_placeholders(ar_str);
            let en_ph = extract_placeholders(en_str);
            if ar_ph != en_ph {
                placeholder_mismatches.push(PlaceholderMismatch {
                    key: key.clone(),
                    ar_placeholders: ar_ph,
                    en_placeholders: en_ph,
                });
            }
        }
    }

    let is_valid = missing_in_arabic.is_empty()
        && missing_in_english.is_empty()
        && empty_value_keys.is_empty()
        && placeholder_mismatches.is_empty();

    Ok(ParityReport {
        is_valid,
        missing_in_arabic,
        missing_in_english,
        empty_value_keys,
        placeholder_mismatches,
    })
}

/// Verifies the embedded `src-ui/i18n/ar.json` and `src-ui/i18n/en.json` catalogs.
pub fn verify_embedded_catalogs() -> Result<ParityReport, String> {
    verify_catalog_parity(AR_CATALOG_JSON, EN_CATALOG_JSON)
}

/// Wraps a mixed-direction filename or path in an HTML `<bdi dir="auto">` isolation element.
pub fn wrap_bidi_isolated(text: &str) -> String {
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    format!("<bdi dir=\"auto\">{escaped}</bdi>")
}

/// Runtime state machine for switching language and direction without restarting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDirectionState {
    pub language: Language,
    pub direction: TextDirection,
}

impl RuntimeDirectionState {
    pub fn new(initial: Language) -> Self {
        Self {
            language: initial,
            direction: initial.direction(),
        }
    }

    pub fn switch_to(&mut self, next: Language) {
        self.language = next;
        self.direction = next.direction();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arabic_english_key_and_placeholder_parity() {
        let report =
            verify_embedded_catalogs().expect("Embedded ar.json and en.json should parse cleanly");
        assert!(
            report.is_valid,
            "Translation parity check failed: {report:?}"
        );
        assert!(report.missing_in_arabic.is_empty());
        assert!(report.missing_in_english.is_empty());
        assert!(report.empty_value_keys.is_empty());
        assert!(report.placeholder_mismatches.is_empty());

        // Verify negative / error path: missing key or mismatched placeholder fails immediately
        let sample_ar = r#"{
            "app": { "welcome": "مرحبًا {name} لديك {count} ملفات", "onlyAr": "نص" }
        }"#;
        let sample_en = r#"{
            "app": { "welcome": "Welcome {name}", "onlyEn": "   " }
        }"#;

        let broken_report = verify_catalog_parity(sample_ar, sample_en).unwrap();
        assert!(!broken_report.is_valid);
        assert_eq!(broken_report.missing_in_arabic, vec!["app.onlyEn"]);
        assert_eq!(broken_report.missing_in_english, vec!["app.onlyAr"]);
        assert_eq!(broken_report.empty_value_keys, vec!["en:app.onlyEn"]);
        assert_eq!(broken_report.placeholder_mismatches.len(), 1);
        assert_eq!(broken_report.placeholder_mismatches[0].key, "app.welcome");
        assert_eq!(
            broken_report.placeholder_mismatches[0].ar_placeholders,
            vec!["count", "name"]
        );
        assert_eq!(
            broken_report.placeholder_mismatches[0].en_placeholders,
            vec!["name"]
        );
    }

    #[test]
    fn test_runtime_direction_switch_rtl_ltr() {
        let mut state = RuntimeDirectionState::new(Language::Ar);
        assert_eq!(state.language.code(), "ar");
        assert_eq!(state.direction, TextDirection::Rtl);

        state.switch_to(Language::En);
        assert_eq!(state.language.code(), "en");
        assert_eq!(state.direction, TextDirection::Ltr);

        state.switch_to(Language::Ar);
        assert_eq!(state.language.code(), "ar");
        assert_eq!(state.direction, TextDirection::Rtl);

        // Verify mixed-direction filename isolation with <bdi dir="auto">
        let isolated = wrap_bidi_isolated("تقرير_الميزانية_2026_Final(1).pdf");
        assert_eq!(
            isolated,
            "<bdi dir=\"auto\">تقرير_الميزانية_2026_Final(1).pdf</bdi>"
        );
    }
}
