import React, { useState } from 'react';
import { useTheme, ThemePreset, AccentColor } from '../theme/ThemeContext';
import { useI18n } from '../i18n/I18nContext';

export const ThemeCustomizerBar: React.FC = () => {
  const { settings, setPreset, setAccentColor, setOpacity, setBlur, setRadius } = useTheme();
  const { locale } = useI18n();
  const [showDrawer, setShowDrawer] = useState(false);

  const isAr = locale === 'ar';

  const presets: { id: ThemePreset; name: string; icon: string }[] = [
    { id: 'glass', name: isAr ? 'رَوَاق زجاجي' : 'Ambient Glass', icon: '🪟' },
    { id: 'island', name: isAr ? 'جزيرة حية' : 'Dynamic Island', icon: '🍎' },
    { id: 'linear', name: isAr ? 'هندسي دقيق' : 'Linear Precision', icon: '⚡' },
  ];

  const colors: { id: AccentColor; name: string; hex: string }[] = [
    { id: 'cyan', name: isAr ? 'نيلي' : 'Cyan', hex: '#38bdf8' },
    { id: 'emerald', name: isAr ? 'زمردي' : 'Emerald', hex: '#34d399' },
    { id: 'amber', name: isAr ? 'عنبري' : 'Amber', hex: '#fbbf24' },
    { id: 'violet', name: isAr ? 'أرجواني' : 'Violet', hex: '#a78bfa' },
    { id: 'rose', name: isAr ? 'وردي' : 'Rose', hex: '#fb7185' },
    { id: 'monochrome', name: isAr ? 'أحادي' : 'Mono', hex: '#f8fafc' },
  ];

  return (
    <div className="theme-customizer-bar" data-testid="theme-customizer-bar">
      {/* Preset Pills */}
      <div className="theme-presets-group" data-testid="theme-presets-group">
        {presets.map((p) => (
          <button
            key={p.id}
            type="button"
            className={`preset-btn ${settings.preset === p.id ? 'is-active' : ''}`}
            onClick={() => setPreset(p.id)}
            data-testid={`preset-btn-${p.id}`}
          >
            <span className="preset-icon">{p.icon}</span>
            <span className="preset-label">{p.name}</span>
          </button>
        ))}
      </div>

      {/* Accent Color Palette */}
      <div className="theme-accents-group" data-testid="theme-accents-group">
        {colors.map((c) => (
          <button
            key={c.id}
            type="button"
            className={`accent-dot ${settings.accentColor === c.id ? 'is-selected' : ''}`}
            style={{ backgroundColor: c.hex }}
            onClick={() => setAccentColor(c.id)}
            title={c.name}
            data-testid={`accent-btn-${c.id}`}
            aria-label={c.name}
          />
        ))}
      </div>

      {/* Fine-Tuning Drawer Toggle */}
      <button
        type="button"
        className="fine-tune-toggle-btn"
        onClick={() => setShowDrawer(!showDrawer)}
        data-testid="fine-tune-toggle-btn"
      >
        <span>⚙️</span>
        <span>{isAr ? 'تخصيص دقيق' : 'Tune'}</span>
      </button>

      {/* Fine-Tuning Drawer */}
      {showDrawer && (
        <div className="fine-tune-drawer" data-testid="fine-tune-drawer">
          <div className="drawer-header">
            <h4>{isAr ? 'تخصيص المظهر بالبكسل' : 'Pixel-Level Fine Tuning'}</h4>
            <button
              type="button"
              className="close-drawer-btn"
              onClick={() => setShowDrawer(false)}
            >
              ✕
            </button>
          </div>

          <div className="slider-control">
            <div className="slider-label">
              <span>{isAr ? 'الشفافية' : 'Opacity'}</span>
              <span>{Math.round(settings.opacity * 100)}%</span>
            </div>
            <input
              type="range"
              min="0.2"
              max="1.0"
              step="0.05"
              value={settings.opacity}
              onChange={(e) => setOpacity(parseFloat(e.target.value))}
              data-testid="opacity-slider"
            />
          </div>

          <div className="slider-control">
            <div className="slider-label">
              <span>{isAr ? 'الضبابية الزجاجية' : 'Glass Blur'}</span>
              <span>{settings.blur}px</span>
            </div>
            <input
              type="range"
              min="0"
              max="40"
              step="2"
              value={settings.blur}
              onChange={(e) => setBlur(parseInt(e.target.value, 10))}
              data-testid="blur-slider"
            />
          </div>

          <div className="slider-control">
            <div className="slider-label">
              <span>{isAr ? 'استدارة الزوايا' : 'Corner Radius'}</span>
              <span>{settings.radius}px</span>
            </div>
            <input
              type="range"
              min="2"
              max="36"
              step="2"
              value={settings.radius}
              onChange={(e) => setRadius(parseInt(e.target.value, 10))}
              data-testid="radius-slider"
            />
          </div>
        </div>
      )}
    </div>
  );
};
