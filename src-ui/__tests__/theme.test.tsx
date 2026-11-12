import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { I18nProvider } from '../i18n/I18nContext';
import { ThemeProvider } from '../theme/ThemeContext';
import { ThemeCustomizerBar } from '../components/ThemeCustomizerBar';
import { App } from '../App';

describe('Multi-Style Theme Engine & Customizer UI', () => {
  it('renders ThemeCustomizerBar with 3 presets and 6 accent colors', () => {
    render(
      <I18nProvider initialLocale="ar">
        <ThemeProvider initialPreset="glass">
          <ThemeCustomizerBar />
        </ThemeProvider>
      </I18nProvider>
    );

    // Preset buttons in Arabic
    expect(screen.getByTestId('preset-btn-glass')).toBeInTheDocument();
    expect(screen.getByTestId('preset-btn-island')).toBeInTheDocument();
    expect(screen.getByTestId('preset-btn-linear')).toBeInTheDocument();
    expect(screen.getByText('رَوَاق زجاجي')).toBeInTheDocument();
    expect(screen.getByText('جزيرة حية')).toBeInTheDocument();
    expect(screen.getByText('هندسي دقيق')).toBeInTheDocument();

    // Accent colors
    expect(screen.getByTestId('accent-btn-cyan')).toBeInTheDocument();
    expect(screen.getByTestId('accent-btn-emerald')).toBeInTheDocument();
    expect(screen.getByTestId('accent-btn-amber')).toBeInTheDocument();
    expect(screen.getByTestId('accent-btn-violet')).toBeInTheDocument();
    expect(screen.getByTestId('accent-btn-rose')).toBeInTheDocument();
    expect(screen.getByTestId('accent-btn-monochrome')).toBeInTheDocument();
  });

  it('switches between presets dynamically', () => {
    const { container } = render(
      <I18nProvider initialLocale="en">
        <ThemeProvider initialPreset="glass">
          <ThemeCustomizerBar />
        </ThemeProvider>
      </I18nProvider>
    );

    const root = container.querySelector('.rawaq-theme-root');
    expect(root).toHaveAttribute('data-theme-preset', 'glass');

    // Switch to island
    fireEvent.click(screen.getByTestId('preset-btn-island'));
    expect(root).toHaveAttribute('data-theme-preset', 'island');
    expect(root).toHaveAttribute('data-accent-color', 'emerald');

    // Switch to linear
    fireEvent.click(screen.getByTestId('preset-btn-linear'));
    expect(root).toHaveAttribute('data-theme-preset', 'linear');
    expect(root).toHaveAttribute('data-accent-color', 'monochrome');

    // Switch back to glass
    fireEvent.click(screen.getByTestId('preset-btn-glass'));
    expect(root).toHaveAttribute('data-theme-preset', 'glass');
    expect(root).toHaveAttribute('data-accent-color', 'cyan');
  });

  it('changes accent color independently of preset', () => {
    const { container } = render(
      <I18nProvider initialLocale="en">
        <ThemeProvider initialPreset="glass">
          <ThemeCustomizerBar />
        </ThemeProvider>
      </I18nProvider>
    );

    const root = container.querySelector('.rawaq-theme-root');
    expect(root).toHaveAttribute('data-accent-color', 'cyan');

    // Click Amber
    fireEvent.click(screen.getByTestId('accent-btn-amber'));
    expect(root).toHaveAttribute('data-accent-color', 'amber');

    // Click Violet
    fireEvent.click(screen.getByTestId('accent-btn-violet'));
    expect(root).toHaveAttribute('data-accent-color', 'violet');
  });

  it('opens fine-tuning drawer and adjusts sliders', () => {
    render(
      <I18nProvider initialLocale="en">
        <ThemeProvider initialPreset="glass">
          <ThemeCustomizerBar />
        </ThemeProvider>
      </I18nProvider>
    );

    expect(screen.queryByTestId('fine-tune-drawer')).not.toBeInTheDocument();

    // Toggle drawer open
    fireEvent.click(screen.getByTestId('fine-tune-toggle-btn'));
    expect(screen.getByTestId('fine-tune-drawer')).toBeInTheDocument();

    // Adjust blur slider
    const blurSlider = screen.getByTestId('blur-slider');
    fireEvent.change(blurSlider, { target: { value: '30' } });
    expect(screen.getByText('30px')).toBeInTheDocument();

    // Adjust radius slider
    const radiusSlider = screen.getByTestId('radius-slider');
    fireEvent.change(radiusSlider, { target: { value: '20' } });
    expect(screen.getByText('20px')).toBeInTheDocument();
  });

  it('renders unified workspace navigation in App with tabs', () => {
    render(
      <I18nProvider initialLocale="ar">
        <App />
      </I18nProvider>
    );

    expect(screen.getByTestId('workspace-tabs')).toBeInTheDocument();
    expect(screen.getByTestId('tab-productivity-btn')).toBeInTheDocument();
    expect(screen.getByTestId('tab-displays-btn')).toBeInTheDocument();
    expect(screen.getByTestId('tab-organizer-btn')).toBeInTheDocument();
    expect(screen.getByTestId('tab-overview-btn')).toBeInTheDocument();

    // Default active tab is overview
    expect(screen.getByTestId('security-path-status')).toBeInTheDocument();

    // Switch to productivity tab
    fireEvent.click(screen.getByTestId('tab-productivity-btn'));
    expect(screen.getByTestId('productivity-hub')).toBeInTheDocument();

    // Switch to displays tab
    fireEvent.click(screen.getByTestId('tab-displays-btn'));
    expect(screen.getByTestId('multimonitor-hub')).toBeInTheDocument();
  });
});
