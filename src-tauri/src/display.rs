//! Display topology management, multi-monitor adaptation, and resource saver mode for Rawaq.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DisplayError {
    #[error("Display not found: {0}")]
    DisplayNotFound(String),
    #[error("Box placement not found: {0}")]
    BoxNotFound(u64),
    #[error("No displays available in topology")]
    NoDisplaysAvailable,
    #[error("Invalid coordinates: {0}")]
    InvalidCoordinates(String),
}

/// 2D rectangle in virtual desktop screen space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl DisplayRect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        px >= self.x
            && px < self.x + self.width as i32
            && py >= self.y
            && py < self.y + self.height as i32
    }

    pub fn contains_rect(&self, other: &DisplayRect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && (other.x + other.width as i32) <= (self.x + self.width as i32)
            && (other.y + other.height as i32) <= (self.y + self.height as i32)
    }

    /// Clamps self so it is strictly within the boundaries of `container`.
    /// If self is larger than container, it shrinks to fit container.
    pub fn clamp_within(&mut self, container: &DisplayRect) {
        if self.width > container.width {
            self.width = container.width;
        }
        if self.height > container.height {
            self.height = container.height;
        }
        let max_x = container.x + container.width as i32 - self.width as i32;
        let max_y = container.y + container.height as i32 - self.height as i32;

        if self.x < container.x {
            self.x = container.x;
        } else if self.x > max_x {
            self.x = max_x;
        }

        if self.y < container.y {
            self.y = container.y;
        } else if self.y > max_y {
            self.y = max_y;
        }
    }
}

/// Information representing an attached display monitor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub is_primary: bool,
    pub bounds: DisplayRect,
    pub scale_factor: f64,
}

/// Placement record of a desktop box in the display topology.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoxPlacement {
    pub box_id: u64,
    pub preferred_display_id: String,
    pub current_display_id: String,
    pub rect: DisplayRect,
    /// Coordinates (x, y) relative to preferred display origin.
    pub original_coords: (i32, i32),
    pub is_collapsed: bool,
    pub is_suspended: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceSaverReport {
    pub enabled: bool,
    pub suspended_boxes_count: usize,
    pub estimated_memory_saved_kb: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologyChangeReport {
    pub displays_count: usize,
    pub relocated_boxes_count: usize,
    pub restored_boxes_count: usize,
}

/// Manages multi-monitor display topology, relative clamping, and resource saving.
pub struct DisplayTopologyManager {
    displays: HashMap<String, DisplayInfo>,
    placements: HashMap<u64, BoxPlacement>,
    resource_saver_enabled: bool,
}

impl Default for DisplayTopologyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayTopologyManager {
    pub fn new() -> Self {
        Self {
            displays: HashMap::new(),
            placements: HashMap::new(),
            resource_saver_enabled: false,
        }
    }

    pub fn register_display(&mut self, display: DisplayInfo) -> TopologyChangeReport {
        let id = display.id.clone();
        self.displays.insert(id, display);
        self.reconcile_placements()
    }

    pub fn unregister_display(&mut self, display_id: &str) -> Result<TopologyChangeReport, DisplayError> {
        if self.displays.remove(display_id).is_none() {
            return Err(DisplayError::DisplayNotFound(display_id.to_string()));
        }
        Ok(self.reconcile_placements())
    }

    pub fn primary_display(&self) -> Option<&DisplayInfo> {
        self.displays.values().find(|d| d.is_primary).or_else(|| self.displays.values().next())
    }

    pub fn place_box(
        &mut self,
        box_id: u64,
        target_display_id: &str,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<DisplayRect, DisplayError> {
        let display = self.displays.get(target_display_id)
            .ok_or_else(|| DisplayError::DisplayNotFound(target_display_id.to_string()))?;

        // Calculate relative coordinates to the display origin
        let rel_x = x - display.bounds.x;
        let rel_y = y - display.bounds.y;

        let mut rect = DisplayRect::new(x, y, width, height);
        rect.clamp_within(&display.bounds);

        let placement = BoxPlacement {
            box_id,
            preferred_display_id: target_display_id.to_string(),
            current_display_id: target_display_id.to_string(),
            rect,
            original_coords: (rel_x, rel_y),
            is_collapsed: false,
            is_suspended: self.resource_saver_enabled,
        };

        self.placements.insert(box_id, placement);
        Ok(rect)
    }

    pub fn set_display_resolution(
        &mut self,
        display_id: &str,
        new_bounds: DisplayRect,
        new_scale: f64,
    ) -> Result<TopologyChangeReport, DisplayError> {
        let (old_bounds, is_primary, name) = {
            let d = self.displays.get(display_id)
                .ok_or_else(|| DisplayError::DisplayNotFound(display_id.to_string()))?;
            (d.bounds, d.is_primary, d.name.clone())
        };

        self.displays.insert(
            display_id.to_string(),
            DisplayInfo {
                id: display_id.to_string(),
                name,
                is_primary,
                bounds: new_bounds,
                scale_factor: new_scale,
            },
        );

        // Adjust placements on this display proportionally and clamp safely in bounds
        for placement in self.placements.values_mut() {
            if placement.current_display_id == display_id {
                let ratio_x = if old_bounds.width > 0 {
                    (placement.rect.x - old_bounds.x) as f64 / old_bounds.width as f64
                } else {
                    0.0
                };
                let ratio_y = if old_bounds.height > 0 {
                    (placement.rect.y - old_bounds.y) as f64 / old_bounds.height as f64
                } else {
                    0.0
                };

                let mut new_w = placement.rect.width;
                let mut new_h = placement.rect.height;
                if new_w > new_bounds.width {
                    new_w = new_bounds.width;
                }
                if new_h > new_bounds.height {
                    new_h = new_bounds.height;
                }

                let new_x = new_bounds.x + (ratio_x * new_bounds.width as f64).round() as i32;
                let new_y = new_bounds.y + (ratio_y * new_bounds.height as f64).round() as i32;

                let mut new_rect = DisplayRect::new(new_x, new_y, new_w, new_h);
                new_rect.clamp_within(&new_bounds);
                placement.rect = new_rect;
            }
        }

        Ok(TopologyChangeReport {
            displays_count: self.displays.len(),
            relocated_boxes_count: 0,
            restored_boxes_count: 0,
        })
    }

    pub fn reconcile_placements(&mut self) -> TopologyChangeReport {
        let mut relocated = 0;
        let mut restored = 0;

        let primary_id = self.primary_display().map(|d| d.id.clone());

        for placement in self.placements.values_mut() {
            // Case 1: Preferred display is available and currently not on it -> RESTORE
            if let Some(target_display) = self.displays.get(&placement.preferred_display_id) {
                if placement.current_display_id != placement.preferred_display_id {
                    let orig_x = target_display.bounds.x + placement.original_coords.0;
                    let orig_y = target_display.bounds.y + placement.original_coords.1;
                    let mut rect = DisplayRect::new(orig_x, orig_y, placement.rect.width, placement.rect.height);
                    rect.clamp_within(&target_display.bounds);

                    placement.rect = rect;
                    placement.current_display_id = placement.preferred_display_id.clone();
                    restored += 1;
                }
            }
            // Case 2: Preferred display is NOT available -> RELOCATE to fallback (primary)
            else if let Some(ref fallback_id) = primary_id {
                if let Some(fallback_display) = self.displays.get(fallback_id) {
                    if placement.current_display_id != *fallback_id {
                        let mut rect = placement.rect;
                        rect.x = fallback_display.bounds.x + (rect.x % fallback_display.bounds.width as i32).abs();
                        rect.y = fallback_display.bounds.y + (rect.y % fallback_display.bounds.height as i32).abs();
                        rect.clamp_within(&fallback_display.bounds);

                        placement.rect = rect;
                        placement.current_display_id = fallback_id.clone();
                        relocated += 1;
                    }
                }
            }
        }

        TopologyChangeReport {
            displays_count: self.displays.len(),
            relocated_boxes_count: relocated,
            restored_boxes_count: restored,
        }
    }

    pub fn toggle_resource_saver(&mut self, enabled: bool) -> ResourceSaverReport {
        self.resource_saver_enabled = enabled;
        let mut suspended = 0;
        for placement in self.placements.values_mut() {
            if enabled && placement.is_collapsed {
                placement.is_suspended = true;
                suspended += 1;
            } else if !enabled {
                placement.is_suspended = false;
            }
        }
        ResourceSaverReport {
            enabled,
            suspended_boxes_count: suspended,
            estimated_memory_saved_kb: (suspended as u64) * 8192,
        }
    }

    pub fn set_box_collapsed(&mut self, box_id: u64, collapsed: bool) -> Result<bool, DisplayError> {
        let placement = self.placements.get_mut(&box_id)
            .ok_or(DisplayError::BoxNotFound(box_id))?;
        placement.is_collapsed = collapsed;
        if self.resource_saver_enabled {
            placement.is_suspended = collapsed;
        }
        Ok(placement.is_suspended)
    }

    pub fn get_placement(&self, box_id: u64) -> Option<&BoxPlacement> {
        self.placements.get(&box_id)
    }

    pub fn list_placements(&self) -> Vec<BoxPlacement> {
        self.placements.values().cloned().collect()
    }

    pub fn list_displays(&self) -> Vec<DisplayInfo> {
        self.displays.values().cloned().collect()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_unplug_and_reconnect_monitor_restores_exact_coordinates() {
        let mut manager = DisplayTopologyManager::new();

        // 1. Setup two displays: Primary D1 (1920x1080 at 0,0) and External D2 (1920x1080 at 1920,0)
        let d1 = DisplayInfo {
            id: "D1".to_string(),
            name: "Primary Built-in Display".to_string(),
            is_primary: true,
            bounds: DisplayRect::new(0, 0, 1920, 1080),
            scale_factor: 1.0,
        };
        let d2 = DisplayInfo {
            id: "D2".to_string(),
            name: "External 4K Monitor".to_string(),
            is_primary: false,
            bounds: DisplayRect::new(1920, 0, 1920, 1080),
            scale_factor: 1.0,
        };

        manager.register_display(d1.clone());
        manager.register_display(d2.clone());

        // 2. Place box 101 on D2 at exact coordinates (2400, 300) with size 400x300
        let rect = manager
            .place_box(101, "D2", 2400, 300, 400, 300)
            .expect("Placement on D2 should succeed");

        assert_eq!(rect.x, 2400);
        assert_eq!(rect.y, 300);
        let initial_placement = manager.get_placement(101).expect("Placement exists");
        assert_eq!(initial_placement.current_display_id, "D2");
        assert_eq!(initial_placement.preferred_display_id, "D2");

        // 3. Unplug D2: simulate monitor disconnected
        let unplug_report = manager
            .unregister_display("D2")
            .expect("Unregister D2 should succeed");
        assert_eq!(unplug_report.relocated_boxes_count, 1);

        let relocated_placement = manager.get_placement(101).expect("Placement exists");
        assert_eq!(relocated_placement.current_display_id, "D1");
        assert_eq!(relocated_placement.preferred_display_id, "D2");
        // Ensure box is fully within D1 boundaries and not lost in void space
        assert!(d1.bounds.contains_rect(&relocated_placement.rect));

        // 4. Reconnect D2: monitor plugged back in
        let reconnect_report = manager.register_display(d2);
        assert_eq!(reconnect_report.restored_boxes_count, 1);

        let restored_placement = manager.get_placement(101).expect("Placement exists");
        assert_eq!(restored_placement.current_display_id, "D2");
        assert_eq!(restored_placement.preferred_display_id, "D2");
        // Verify EXACT original coordinates restored!
        assert_eq!(restored_placement.rect.x, 2400);
        assert_eq!(restored_placement.rect.y, 300);
        assert_eq!(restored_placement.rect.width, 400);
        assert_eq!(restored_placement.rect.height, 300);
    }

    #[test]
    pub fn test_resolution_and_dpi_scale_change_keeps_boxes_in_bounds() {
        let mut manager = DisplayTopologyManager::new();

        // 1. Initial 4K display (3840x2160) at scale 2.0
        let d1 = DisplayInfo {
            id: "D1".to_string(),
            name: "High-DPI 4K Display".to_string(),
            is_primary: true,
            bounds: DisplayRect::new(0, 0, 3840, 2160),
            scale_factor: 2.0,
        };
        manager.register_display(d1);

        // 2. Place box 201 near bottom-right corner of 4K screen
        let placed_rect = manager
            .place_box(201, "D1", 3200, 1800, 500, 300)
            .expect("Placement should succeed");
        assert_eq!(placed_rect.x, 3200);
        assert_eq!(placed_rect.y, 1800);

        // 3. User switches resolution from 4K (3840x2160) to HD (1366x768) at scale 1.0
        let new_hd_bounds = DisplayRect::new(0, 0, 1366, 768);
        manager
            .set_display_resolution("D1", new_hd_bounds, 1.0)
            .expect("Resolution update should succeed");

        let updated_placement = manager.get_placement(201).expect("Placement exists");
        let updated_rect = updated_placement.rect;

        // Verify the box is strictly clamped and fully visible inside 1366x768
        assert!(new_hd_bounds.contains_rect(&updated_rect));
        assert!(updated_rect.x >= 0);
        assert!(updated_rect.y >= 0);
        assert!((updated_rect.x + updated_rect.width as i32) <= 1366);
        assert!((updated_rect.y + updated_rect.height as i32) <= 768);

        // 4. Extreme edge case: screen shrinks even further to 640x480 (smaller than original box width)
        let tiny_bounds = DisplayRect::new(0, 0, 640, 480);
        manager
            .set_display_resolution("D1", tiny_bounds, 1.0)
            .expect("Resolution update to tiny bounds should succeed");

        let tiny_placement = manager.get_placement(201).expect("Placement exists");
        assert!(tiny_bounds.contains_rect(&tiny_placement.rect));
        assert!(tiny_placement.rect.width <= 640);
        assert!(tiny_placement.rect.height <= 480);
    }
}
