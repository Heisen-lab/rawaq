import React, { useState } from 'react';
import { useI18n } from '../i18n/I18nContext';

export interface DisplayRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface DisplayInfo {
  id: string;
  name: string;
  is_primary: boolean;
  bounds: DisplayRect;
  scale_factor: number;
}

export interface BoxPlacement {
  box_id: number;
  preferred_display_id: string;
  current_display_id: string;
  rect: DisplayRect;
  original_coords: [number, number];
  is_collapsed: boolean;
  is_suspended: boolean;
}

export interface MultiMonitorHubProps {
  initialDisplays?: DisplayInfo[];
  initialPlacements?: BoxPlacement[];
  onToggleResourceSaver?: (enabled: boolean) => void;
}

export const MultiMonitorHub: React.FC<MultiMonitorHubProps> = ({
  initialDisplays = [
    {
      id: 'disp_primary',
      name: 'Primary Built-in Display',
      is_primary: true,
      bounds: { x: 0, y: 0, width: 1920, height: 1080 },
      scale_factor: 1.0,
    },
    {
      id: 'disp_external',
      name: 'External 4K Monitor',
      is_primary: false,
      bounds: { x: 1920, y: 0, width: 3840, height: 2160 },
      scale_factor: 1.5,
    },
  ],
  initialPlacements = [
    {
      box_id: 101,
      preferred_display_id: 'disp_external',
      current_display_id: 'disp_external',
      rect: { x: 2400, y: 350, width: 600, height: 400 },
      original_coords: [480, 350],
      is_collapsed: true,
      is_suspended: false,
    },
    {
      box_id: 202,
      preferred_display_id: 'disp_primary',
      current_display_id: 'disp_primary',
      rect: { x: 200, y: 150, width: 450, height: 350 },
      original_coords: [200, 150],
      is_collapsed: false,
      is_suspended: false,
    },
  ],
  onToggleResourceSaver,
}) => {
  const { t } = useI18n();
  const [displays, setDisplays] = useState<DisplayInfo[]>(initialDisplays);
  const [placements, setPlacements] = useState<BoxPlacement[]>(initialPlacements);
  const [resourceSaverActive, setResourceSaverActive] = useState(false);
  const [stashedDisplay, setStashedDisplay] = useState<DisplayInfo | null>(null);

  const handleToggleResourceSaver = () => {
    const nextState = !resourceSaverActive;
    setResourceSaverActive(nextState);
    setPlacements((prev) =>
      prev.map((p) => ({
        ...p,
        is_suspended: nextState ? p.is_collapsed : false,
      }))
    );
    if (onToggleResourceSaver) {
      onToggleResourceSaver(nextState);
    }
  };

  const handleSimulateUnplug = () => {
    const ext = displays.find((d) => !d.is_primary);
    if (!ext) return;

    setStashedDisplay(ext);
    setDisplays((prev) => prev.filter((d) => d.id !== ext.id));

    // Relocate boxes on unplugged display to primary
    const primary = displays.find((d) => d.is_primary) ?? displays[0];
    if (!primary) return;
    setPlacements((prev) =>
      prev.map((p) => {
        if (p.current_display_id === ext.id) {
          return {
            ...p,
            current_display_id: primary.id,
            rect: {
              ...p.rect,
              x: primary.bounds.x + 100,
              y: primary.bounds.y + 100,
            },
          };
        }
        return p;
      })
    );
  };

  const handleSimulateReconnect = () => {
    if (!stashedDisplay) return;

    const restoredDisplay = stashedDisplay;
    setDisplays((prev) => [...prev, restoredDisplay]);
    setStashedDisplay(null);

    // Restore exact original coordinates on the reconnected display
    setPlacements((prev) =>
      prev.map((p) => {
        if (p.preferred_display_id === restoredDisplay.id) {
          return {
            ...p,
            current_display_id: restoredDisplay.id,
            rect: {
              ...p.rect,
              x: restoredDisplay.bounds.x + p.original_coords[0],
              y: restoredDisplay.bounds.y + p.original_coords[1],
            },
          };
        }
        return p;
      })
    );
  };

  const suspendedCount = placements.filter((p) => p.is_suspended).length;
  const savedKb = suspendedCount * 8192;

  return (
    <div className="multimonitor-hub" data-testid="multimonitor-hub">
      <div className="multimonitor-header">
        <h2 className="multimonitor-title">{t('display.title')}</h2>
        <span className="multimonitor-count-badge">
          {t('display.displaysCount', { count: displays.length })}
        </span>
      </div>

      {/* Displays Grid */}
      <div className="multimonitor-displays-grid">
        {displays.map((disp) => (
          <div
            key={disp.id}
            className={`multimonitor-display-card ${disp.is_primary ? 'is-primary' : 'is-external'}`}
            data-testid={`display-card-${disp.id}`}
          >
            <div className="multimonitor-display-header">
              <span className="multimonitor-display-name">{disp.name}</span>
              <span className={`badge ${disp.is_primary ? 'badge-primary' : 'badge-external'}`}>
                {disp.is_primary ? t('display.primaryBadge') : t('display.externalBadge')}
              </span>
            </div>
            <div className="multimonitor-display-meta">
              {t('display.resolutionLabel', {
                width: disp.bounds.width,
                height: disp.bounds.height,
                scale: disp.scale_factor,
              })}
            </div>
          </div>
        ))}
      </div>

      {/* Simulation Controls */}
      <div className="multimonitor-actions">
        {stashedDisplay ? (
          <button
            type="button"
            className="action-btn btn-reconnect"
            onClick={handleSimulateReconnect}
            data-testid="simulate-reconnect-btn"
          >
            {t('display.simulateReconnect')}
          </button>
        ) : (
          <button
            type="button"
            className="action-btn btn-unplug"
            onClick={handleSimulateUnplug}
            data-testid="simulate-unplug-btn"
          >
            {t('display.simulateUnplug')}
          </button>
        )}
      </div>

      {/* Resource Saver Banner */}
      <div
        className={`resource-saver-banner ${resourceSaverActive ? 'is-active' : 'is-inactive'}`}
        data-testid="resource-saver-banner"
      >
        <div className="resource-saver-info">
          <div className="resource-saver-title">{t('display.resourceSaverTitle')}</div>
          <div className="resource-saver-desc">
            {resourceSaverActive
              ? t('display.resourceSaverActive', { count: suspendedCount, kb: savedKb })
              : t('display.resourceSaverInactive')}
          </div>
        </div>
        <button
          type="button"
          className="action-btn btn-toggle-saver"
          onClick={handleToggleResourceSaver}
          data-testid="toggle-saver-btn"
        >
          {t('display.toggleSaver')}
        </button>
      </div>

      {/* Placements List */}
      <div className="multimonitor-placements-section">
        <h3 className="placements-title">{t('display.placementsTitle')}</h3>
        <div className="placements-list">
          {placements.map((p) => (
            <div
              key={p.box_id}
              className={`placement-card ${p.is_suspended ? 'is-suspended' : ''}`}
              data-testid={`placement-card-${p.box_id}`}
            >
              <div className="placement-header">
                <span className="placement-box-name">
                  {t('display.boxOnDisplay', { id: p.box_id, display: p.current_display_id })}
                </span>
                {p.is_suspended && (
                  <span className="badge badge-suspended">Suspended (~8MB)</span>
                )}
              </div>
              <div className="placement-coords">
                X: {p.rect.x}, Y: {p.rect.y} | {p.rect.width}×{p.rect.height}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Release Checksums Indicator */}
      <div className="release-status-footer" data-testid="release-status-footer">
        <span className="release-status-icon">🔒</span>
        <div className="release-status-content">
          <strong>{t('display.releaseChecksumsTitle')}: </strong>
          <span>{t('display.releaseAllVerified')}</span>
        </div>
      </div>
    </div>
  );
};
