import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { I18nProvider } from '../i18n/I18nContext';
import { MultiMonitorHub } from '../components/MultiMonitorHub';

describe('Stage 7 Multi-Monitor Hub & Resource Saver UI', () => {
  it('renders MultiMonitorHub in Arabic (RTL) with displays and placements', () => {
    render(
      <I18nProvider initialLocale="ar">
        <MultiMonitorHub />
      </I18nProvider>
    );

    // Title and display count
    expect(screen.getByText('إدارة الشاشات المتعددة وتوفير الموارد')).toBeInTheDocument();
    expect(screen.getByText('الشاشات المتصلة: 2')).toBeInTheDocument();

    // Primary and external display cards
    expect(screen.getByText('Primary Built-in Display')).toBeInTheDocument();
    expect(screen.getByText('External 4K Monitor')).toBeInTheDocument();
    expect(screen.getByText('الأساسية')).toBeInTheDocument();
    expect(screen.getByText('خارجية')).toBeInTheDocument();

    // Box placements
    expect(screen.getByText(/الصندوق #101/)).toBeInTheDocument();
    expect(screen.getByText(/الصندوق #202/)).toBeInTheDocument();

    // Release checksums badge
    expect(screen.getByText(/بصمات حزم الإصدار الرسمية/)).toBeInTheDocument();
  });

  it('toggles Resource Saver Mode and suspends collapsed boxes', () => {
    const handleToggle = vi.fn();
    render(
      <I18nProvider initialLocale="ar">
        <MultiMonitorHub onToggleResourceSaver={handleToggle} />
      </I18nProvider>
    );

    // Initially inactive
    expect(screen.getByText(/معطّل — جميع الصناديق تعمل بالرسم الكامل/)).toBeInTheDocument();

    // Toggle on
    const toggleBtn = screen.getByTestId('toggle-saver-btn');
    fireEvent.click(toggleBtn);

    expect(handleToggle).toHaveBeenCalledWith(true);
    // Suspended 1 box (box 101 was collapsed) -> 8192 KB saved
    expect(screen.getByText(/مفعّل — تم تعليق 1 صندوق وتوفير ~8192 ك.ب/)).toBeInTheDocument();
    expect(screen.getByText('Suspended (~8MB)')).toBeInTheDocument();

    // Toggle off
    fireEvent.click(toggleBtn);
    expect(handleToggle).toHaveBeenCalledWith(false);
    expect(screen.getByText(/معطّل — جميع الصناديق تعمل بالرسم الكامل/)).toBeInTheDocument();
  });

  it('handles unplug and reconnect simulation restoring coordinates', () => {
    render(
      <I18nProvider initialLocale="en">
        <MultiMonitorHub />
      </I18nProvider>
    );

    expect(screen.getByText('Connected Displays: 2')).toBeInTheDocument();

    // Unplug external display
    const unplugBtn = screen.getByTestId('simulate-unplug-btn');
    fireEvent.click(unplugBtn);

    // Should now show 1 display
    expect(screen.getByText('Connected Displays: 1')).toBeInTheDocument();
    expect(screen.queryByText('External 4K Monitor')).not.toBeInTheDocument();

    // Box 101 moved to primary display
    expect(screen.getByText(/Box #101 on Display: disp_primary/)).toBeInTheDocument();

    // Reconnect external display
    const reconnectBtn = screen.getByTestId('simulate-reconnect-btn');
    fireEvent.click(reconnectBtn);

    // Both displays back
    expect(screen.getByText('Connected Displays: 2')).toBeInTheDocument();
    expect(screen.getByText('External 4K Monitor')).toBeInTheDocument();

    // Box 101 restored to external display with exact coordinates
    expect(screen.getByText(/Box #101 on Display: disp_external/)).toBeInTheDocument();
    expect(screen.getByText(/X: 2400, Y: 350/)).toBeInTheDocument();
  });
});
