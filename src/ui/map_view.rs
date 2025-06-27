//! Map viewing widget for OP2MapViewer

use eframe::egui::{self, Sense, Rect, Ui, Vec2};
use egui::{Color32, Stroke, Pos2};

use crate::map::types::{Map, Position};

/// Configuration for the map viewer
#[derive(Clone, Debug)]
pub struct MapViewConfig {
    pub zoom_level: f32,
    pub show_grid: bool,
    pub cell_size: f32,
    pub grid_color: Color32,
    pub background_color: Color32,
}

impl Default for MapViewConfig {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            show_grid: false,
            cell_size: 32.0,
            grid_color: Color32::from_gray(128),
            background_color: Color32::BLACK,
        }
    }
}

/// Map viewing widget that handles rendering and interaction
pub struct MapView {
    config: MapViewConfig,
    pan_offset: Vec2,
    dragging: bool,
    drag_start: Option<Pos2>,
    drag_start_offset: Option<Vec2>,
    hovered_cell: Option<Position>,
}

impl MapView {
    pub fn new() -> Self {
        Self {
            config: MapViewConfig::default(),
            pan_offset: Vec2::ZERO,
            dragging: false,
            drag_start: None,
            drag_start_offset: None,
            hovered_cell: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_config(config: MapViewConfig) -> Self {
        Self {
            config,
            pan_offset: Vec2::ZERO,
            dragging: false,
            drag_start: None,
            drag_start_offset: None,
            hovered_cell: None,
        }
    }

    /// Show the map viewer widget
    pub fn show(&mut self, ui: &mut Ui, map: &Map) -> Option<Position> {
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            Sense::click_and_drag(),
        );

        // Handle mouse input
        if response.clicked() {
            self.dragging = true;
            if let Some(pos) = response.hover_pos() {
                self.drag_start = Some(pos);
                self.drag_start_offset = Some(self.pan_offset);
            }
        }
        if response.drag_stopped() {
            self.dragging = false;
            self.drag_start = None;
            self.drag_start_offset = None;
        }
        if self.dragging {
            if let (Some(start), Some(start_offset)) = (self.drag_start, self.drag_start_offset) {
                if let Some(current) = response.hover_pos() {
                    self.pan_offset = start_offset + (current - start);
                }
            }
        }

        // Calculate visible area
        let cell_size = self.config.cell_size * self.config.zoom_level;
        // Calculate map dimensions for potential future use in viewport bounds checking
        let _map_width = (map.info.width as f32) * cell_size;
        let _map_height = (map.info.height as f32) * cell_size;

        // Draw background
        painter.rect_filled(
            response.rect,
            0.0,
            self.config.background_color,
        );

        // Draw cells
        let visible_rect = response.rect;
        let offset = self.pan_offset + visible_rect.center().to_vec2();

        // Calculate visible cell range
        let min_x = (((-offset.x) / cell_size) - 1.0).floor() as i32;
        let min_y = (((-offset.y) / cell_size) - 1.0).floor() as i32;
        let max_x = (((visible_rect.width() - offset.x) / cell_size) + 1.0).ceil() as i32;
        let max_y = (((visible_rect.height() - offset.y) / cell_size) + 1.0).ceil() as i32;

        // Update hovered cell
        self.hovered_cell = response.hover_pos().map(|pos| {
            let map_pos = pos - offset;
            Position::new(
                (map_pos.x / cell_size).floor() as i32,
                (map_pos.y / cell_size).floor() as i32,
            )
        });

        // Draw visible cells
        for y in min_y..max_y {
            for x in min_x..max_x {
                if x < 0 || y < 0 || x >= map.info.width as i32 || y >= map.info.height as i32 {
                    continue;
                }

                if let Some(cell) = map.get_cell(x, y) {
                    let cell_rect = Rect::from_min_size(
                        Pos2::new(
                            offset.x + (x as f32 * cell_size),
                            offset.y + (y as f32 * cell_size),
                        ),
                        Vec2::splat(cell_size),
                    );

                    // Draw cell based on type
                    let cell_color = match cell.cell_type {
                        crate::map::types::CellType::Normal => Color32::from_gray(64),
                        crate::map::types::CellType::Lava(_) => Color32::RED,
                        crate::map::types::CellType::Microbe(_) => Color32::GREEN,
                        crate::map::types::CellType::Mine(depleted) => {
                            if depleted { Color32::DARK_GRAY } else { Color32::YELLOW }
                        },
                        crate::map::types::CellType::Dirt(_) => Color32::from_rgb(139, 69, 19),
                        crate::map::types::CellType::Rock(_) => Color32::GRAY,
                        crate::map::types::CellType::Tube(_) => Color32::BLUE,
                        crate::map::types::CellType::Wall(_) => Color32::WHITE,
                    };

                    painter.rect_filled(cell_rect, 0.0, cell_color);

                    // Draw grid if enabled
                    if self.config.show_grid {
                        painter.rect_stroke(
                            cell_rect,
                            0.0,
                            Stroke::new(1.0, self.config.grid_color),
                        );
                    }
                }
            }
        }

        // Return hovered cell position if any
        self.hovered_cell
    }

    /// Get the current configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &MapViewConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut MapViewConfig {
        &mut self.config
    }
}
