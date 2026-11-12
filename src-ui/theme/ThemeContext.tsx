import React, { createContext, useContext, useState, useEffect, useMemo } from 'react';

export type ThemePreset = 'glass' | 'island' | 'linear';
export type AccentColor = 'cyan' | 'emerald' | 'amber' | 'violet' | 'rose' | 'monochrome';

export interface ThemeSettings {
  preset: ThemePreset;
  accentColor: AccentColor;
  opacity: number;
  blur: number;
  radius: number;
}

export interface ThemeContextValue {
  settings: ThemeSettings;
  setPreset: (preset: ThemePreset) => void;
  setAccentColor: (accent: AccentColor) => void;
  setOpacity: (opacity: number) => void;
  setBlur: (blur: number) => void;
  setRadius: (radius: number) => void;
  resetToPreset: (preset: ThemePreset) => void;
}

const PRESET_DEFAULTS: Record<ThemePreset, Omit<ThemeSettings, 'preset'>> = {
  glass: {
    accentColor: 'cyan',
    opacity: 0.55,
    blur: 24,
    radius: 16,
  },
  island: {
    accentColor: 'emerald',
    opacity: 0.88,
    blur: 16,
    radius: 28,
  },
  linear: {
    accentColor: 'monochrome',
    opacity: 1.0,
    blur: 0,
    radius: 6,
  },
};

const ACCENT_PALETTE: Record<AccentColor, { primary: string; hover: string; glow: string }> = {
  cyan: {
    primary: '#38bdf8',
    hover: '#0284c7',
    glow: 'rgba(56, 189, 248, 0.35)',
  },
  emerald: {
    primary: '#34d399',
    hover: '#059669',
    glow: 'rgba(52, 211, 153, 0.35)',
  },
  amber: {
    primary: '#fbbf24',
    hover: '#d97706',
    glow: 'rgba(251, 191, 36, 0.35)',
  },
  violet: {
    primary: '#a78bfa',
    hover: '#7c3aed',
    glow: 'rgba(167, 139, 250, 0.35)',
  },
  rose: {
    primary: '#fb7185',
    hover: '#e11d48',
    glow: 'rgba(251, 113, 133, 0.35)',
  },
  monochrome: {
    primary: '#f8fafc',
    hover: '#94a3b8',
    glow: 'rgba(248, 250, 252, 0.25)',
  },
};

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

export const ThemeProvider: React.FC<{
  initialPreset?: ThemePreset;
  children: React.ReactNode;
}> = ({ initialPreset = 'glass', children }) => {
  const [settings, setSettings] = useState<ThemeSettings>(() => {
    const defaults = PRESET_DEFAULTS[initialPreset];
    return {
      preset: initialPreset,
      ...defaults,
    };
  });

  const resetToPreset = React.useCallback((preset: ThemePreset) => {
    const defaults = PRESET_DEFAULTS[preset];
    setSettings({
      preset,
      ...defaults,
    });
  }, []);

  const setPreset = React.useCallback((preset: ThemePreset) => {
    resetToPreset(preset);
  }, [resetToPreset]);

  const setAccentColor = React.useCallback((accentColor: AccentColor) => {
    setSettings((prev) => ({ ...prev, accentColor }));
  }, []);

  const setOpacity = React.useCallback((opacity: number) => {
    setSettings((prev) => ({ ...prev, opacity: Math.max(0.2, Math.min(1.0, opacity)) }));
  }, []);

  const setBlur = React.useCallback((blur: number) => {
    setSettings((prev) => ({ ...prev, blur: Math.max(0, Math.min(40, blur)) }));
  }, []);

  const setRadius = React.useCallback((radius: number) => {
    setSettings((prev) => ({ ...prev, radius: Math.max(2, Math.min(36, radius)) }));
  }, []);

  // Sync CSS variables onto document root
  useEffect(() => {
    const root = document.documentElement;
    const accent = ACCENT_PALETTE[settings.accentColor];

    root.style.setProperty('--app-glass-blur', `${settings.blur}px`);
    root.style.setProperty('--app-radius', `${settings.radius}px`);
    root.style.setProperty('--app-accent', accent.primary);
    root.style.setProperty('--app-accent-glow', accent.glow);
    root.style.setProperty('--app-opacity', settings.opacity.toString());

    if (settings.preset === 'glass') {
      root.style.setProperty('--app-bg', `rgba(15, 23, 42, ${settings.opacity})`);
      root.style.setProperty('--app-border', 'rgba(255, 255, 255, 0.12)');
      root.style.setProperty('--app-shadow', '0 12px 36px 0 rgba(0, 0, 0, 0.45)');
    } else if (settings.preset === 'island') {
      root.style.setProperty('--app-bg', `rgba(10, 10, 14, ${settings.opacity})`);
      root.style.setProperty('--app-border', 'rgba(255, 255, 255, 0.08)');
      root.style.setProperty('--app-shadow', '0 16px 48px rgba(0, 0, 0, 0.65)');
    } else {
      // linear
      root.style.setProperty('--app-bg', '#090d16');
      root.style.setProperty('--app-border', '1px solid rgba(255, 255, 255, 0.12)');
      root.style.setProperty('--app-shadow', 'none');
    }
  }, [settings]);

  const value = useMemo(
    () => ({
      settings,
      setPreset,
      setAccentColor,
      setOpacity,
      setBlur,
      setRadius,
      resetToPreset,
    }),
    [settings, setPreset, setAccentColor, setOpacity, setBlur, setRadius, resetToPreset]
  );

  return (
    <ThemeContext.Provider value={value}>
      <div
        className={`rawaq-theme-root theme-${settings.preset} accent-${settings.accentColor}`}
        data-theme-preset={settings.preset}
        data-accent-color={settings.accentColor}
      >
        {children}
      </div>
    </ThemeContext.Provider>
  );
};

export const useTheme = (): ThemeContextValue => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};
