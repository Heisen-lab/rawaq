export type SupportedLocale = "ar" | "en";
export type TextDirection = "rtl" | "ltr";

export interface TranslationParityReport {
  isValid: boolean;
  missingInArabic: string[];
  missingInEnglish: string[];
  emptyValueKeys: string[];
  placeholderMismatches: Array<{
    key: string;
    arPlaceholders: string[];
    enPlaceholders: string[];
  }>;
}

/**
 * Flattens a nested translation dictionary into dot-separated keys (`app.title`, etc.).
 */
export function flattenTranslations(
  obj: Record<string, unknown>,
  prefix = "",
): Record<string, string> {
  const result: Record<string, string> = {};

  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (value !== null && typeof value === "object" && !Array.isArray(value)) {
      Object.assign(
        result,
        flattenTranslations(value as Record<string, unknown>, fullKey),
      );
    } else if (typeof value === "string") {
      result[fullKey] = value;
    } else {
      throw new Error(`Invalid non-string translation value at key "${fullKey}"`);
    }
  }

  return result;
}

/**
 * Extracts sorted unique `{placeholder}` tokens from a translation string.
 */
export function extractPlaceholders(template: string): string[] {
  const matches = template.matchAll(/\{([a-zA-Z0-9_]+)\}/g);
  const set = new Set<string>();
  for (const match of matches) {
    if (match[1]) {
      set.add(match[1]);
    }
  }
  return Array.from(set).sort();
}

/**
 * Verifies 100% parity of translation keys, non-empty values, and `{placeholder}` variables
 * between Arabic (`ar`) and English (`en`) catalogues.
 */
export function verifyTranslationParity(
  arCatalog: Record<string, unknown>,
  enCatalog: Record<string, unknown>,
): TranslationParityReport {
  const arFlat = flattenTranslations(arCatalog);
  const enFlat = flattenTranslations(enCatalog);

  const arKeys = new Set(Object.keys(arFlat));
  const enKeys = new Set(Object.keys(enFlat));

  const missingInArabic: string[] = [];
  const missingInEnglish: string[] = [];
  const emptyValueKeys: string[] = [];
  const placeholderMismatches: TranslationParityReport["placeholderMismatches"] =
    [];

  for (const key of enKeys) {
    if (!arKeys.has(key)) {
      missingInArabic.push(key);
    }
  }

  for (const key of arKeys) {
    if (!enKeys.has(key)) {
      missingInEnglish.push(key);
    }
  }

  const allKeys = new Set([...arKeys, ...enKeys]);
  for (const key of Array.from(allKeys).sort()) {
    const arVal = arFlat[key];
    const enVal = enFlat[key];

    if (typeof arVal === "string" && arVal.trim().length === 0) {
      emptyValueKeys.push(`ar:${key}`);
    }
    if (typeof enVal === "string" && enVal.trim().length === 0) {
      emptyValueKeys.push(`en:${key}`);
    }

    if (typeof arVal === "string" && typeof enVal === "string") {
      const arPlaceholders = extractPlaceholders(arVal);
      const enPlaceholders = extractPlaceholders(enVal);

      if (
        arPlaceholders.length !== enPlaceholders.length ||
        arPlaceholders.some((item, idx) => item !== enPlaceholders[idx])
      ) {
        placeholderMismatches.push({
          key,
          arPlaceholders,
          enPlaceholders,
        });
      }
    }
  }

  const isValid =
    missingInArabic.length === 0 &&
    missingInEnglish.length === 0 &&
    emptyValueKeys.length === 0 &&
    placeholderMismatches.length === 0;

  return {
    isValid,
    missingInArabic: missingInArabic.sort(),
    missingInEnglish: missingInEnglish.sort(),
    emptyValueKeys: emptyValueKeys.sort(),
    placeholderMismatches,
  };
}

/**
 * Formats a translation template by substituting `{key}` placeholders.
 */
export function formatTranslation(
  template: string,
  params?: Record<string, string | number>,
): string {
  if (!params) {
    return template;
  }
  return template.replace(/\{([a-zA-Z0-9_]+)\}/g, (fullMatch, token: string) => {
    if (Object.prototype.hasOwnProperty.call(params, token)) {
      const value = params[token];
      return value !== undefined ? String(value) : fullMatch;
    }
    return fullMatch;
  });
}

/**
 * Returns the canonical text direction for a supported locale.
 */
export function getLocaleDirection(locale: SupportedLocale): TextDirection {
  return locale === "ar" ? "rtl" : "ltr";
}
