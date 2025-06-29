//! Map viewing widget for OP2MapViewer

use eframe::egui::{self, Rect, Sense, TextureHandle, TextureId, TextureOptions, Ui, Vec2};
use egui::{Color32, Image, Pos2, Stroke};
use image::RgbaImage;

use crate::map::types::{Map, Position, TileInfo};

/// Configuration for the map viewer
#[derive(Clone, Debug)]
pub struct MapViewConfig {
    pub zoom_level: f32,
    pub show_grid: bool,
    pub cell_size: f32,
    pub grid_color: Color32,
    pub background_color: Color32,
    pub use_tilesets: bool,
}

impl Default for MapViewConfig {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            show_grid: false,
            cell_size: 32.0,
            grid_color: Color32::from_gray(128),
            background_color: Color32::BLACK,
            use_tilesets: true,
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
    tile_textures: std::collections::HashMap<String, TextureHandle>,
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
            tile_textures: std::collections::HashMap::new(),
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
            tile_textures: std::collections::HashMap::new(),
        }
    }

    /// Get or create a texture for a tileset
    fn get_or_create_tile_texture(
        &mut self,
        ui: &mut Ui,
        map: &Map,
        tileset_name: &str,
    ) -> Option<TextureHandle> {
        if self.tile_textures.contains_key(tileset_name) {
            return self.tile_textures.get(tileset_name).cloned();
        }

        // If we have a tileset cache, load the texture
        if let Some(cache) = &map.tileset_cache {
            if let Some(image) = cache.get_tileset(tileset_name) {
                // Convert the image to RGBA8 format
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];

                // Create a texture from the image
                let texture = ui.ctx().load_texture(
                    tileset_name,
                    egui::ColorImage::from_rgba_unmultiplied(
                        size,
                        rgba_image.as_flat_samples().as_slice(),
                    ),
                    TextureOptions::default(),
                );

                self.tile_textures
                    .insert(tileset_name.to_string(), texture.clone());
                return Some(texture);
            }
        }

        None
    }

    /// Extract a single tile from a tileset texture
    fn extract_tile(&self, tileset: &TextureHandle, tile_index: u32) -> (TextureId, Rect) {
        // Assuming tileset is a texture atlas with tiles laid out in a grid
        // For this simple implementation, assuming 32x32 tiles in a horizontal strip
        let tile_size = 32.0;
        let x = (tile_index as f32) * tile_size;

        // The texture UV coordinates are normalized [0.0-1.0]
        let texture_size = tileset.size_vec2();
        let uv_min_x = x / texture_size.x;
        let uv_min_y = 0.0;
        let uv_max_x = (x + tile_size) / texture_size.x;
        let uv_max_y = tile_size / texture_size.y;

        (
            tileset.id(),
            Rect::from_min_max(Pos2::new(uv_min_x, uv_min_y), Pos2::new(uv_max_x, uv_max_y)),
        )
    }

    /// Show the map viewer widget
    pub fn show(&mut self, ui: &mut Ui, map: &Map) -> Option<Position> {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

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
        painter.rect_filled(response.rect, 0.0, self.config.background_color);

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

                    // Check if we should use tilesets and if this cell has tileset info
                    let use_tile = self.config.use_tilesets
                        && cell.tile_info.is_some()
                        && map.tileset_cache.is_some();

                    if use_tile {
                        if let Some(tile_info) = &cell.tile_info {
                            if let Some(texture) =
                                self.get_or_create_tile_texture(ui, map, &tile_info.tileset_name)
                            {
                                // Extract the specific tile from the tileset
                                let (texture_id, uv_rect) =
                                    self.extract_tile(&texture, tile_info.tile_index);

                                // Draw the tile
                                painter.image(texture_id, cell_rect, uv_rect, Color32::WHITE);
                            } else {
                                // Fallback to colored rectangle if texture loading failed
                                let cell_color = get_cell_color(cell);
                                painter.rect_filled(cell_rect, 0.0, cell_color);
                            }
                        } else {
                            // Fallback to colored rectangle if no tile info
                            let cell_color = get_cell_color(cell);
                            painter.rect_filled(cell_rect, 0.0, cell_color);
                        }
                    } else {
                        // Use colored rectangle representation
                        let cell_color = get_cell_color(cell);
                        painter.rect_filled(cell_rect, 0.0, cell_color);
                    }

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

/// Helper function to get a color for a cell type
fn get_cell_color(cell: &crate::map::types::Cell) -> Color32 {
    match cell.cell_type {
        crate::map::types::CellType::Normal => Color32::from_gray(64),
        crate::map::types::CellType::Lava(_) => Color32::RED,
        crate::map::types::CellType::Microbe(_) => Color32::GREEN,
        crate::map::types::CellType::Mine(depleted) => {
            if depleted {
                Color32::DARK_GRAY
            } else {
                Color32::YELLOW
            }
        }
        crate::map::types::CellType::Dirt(_) => Color32::from_rgb(139, 69, 19),
        crate::map::types::CellType::Rock(_) => Color32::GRAY,
        crate::map::types::CellType::Tube(_) => Color32::BLUE,
        crate::map::types::CellType::Wall(_) => Color32::WHITE,
    }
}
