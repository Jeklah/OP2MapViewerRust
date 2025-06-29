//! Main application for OP2MapViewer

use eframe::egui::{self, TextureHandle};
use rfd::FileDialog;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{cell_info::CellInfoPanel, map_view::MapView};
use crate::map::{load_map, load_tilesets, Map, MapInfo, MapLoadError, TilesetCache};

/// Main application state
pub struct MapViewerApp {
    map: Option<Map>,
    map_texture: Option<TextureHandle>,
    map_path: Option<PathBuf>,
    error_message: Option<String>,
    map_view: MapView,
    cell_info: CellInfoPanel,
    settings_open: bool,
    about_open: bool,
    selected_cell_pos: Option<(i32, i32)>,
    tileset_cache: Option<Arc<TilesetCache>>,
    tileset_path: Option<PathBuf>,
}

impl Default for MapViewerApp {
    fn default() -> Self {
        Self {
            map: None,
            map_texture: None,
            map_path: None,
            error_message: None,
            map_view: MapView::new(),
            cell_info: CellInfoPanel::new(),
            settings_open: false,
            about_open: false,
            selected_cell_pos: None,
            tileset_cache: None,
            tileset_path: None,
        }
    }
}

impl MapViewerApp {
    /// Creates a new instance of the application
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set default theme
        cc.egui_ctx.set_style(egui::Style {
            visuals: egui::Visuals::dark(),
            ..Default::default()
        });

        let mut app = Self::default();

        // Try to load tilesets if they're in the expected location
        let potential_tileset_paths = ["../op2graphics_rs/tilesets.zip", "tilesets.zip"];

        for path in potential_tileset_paths {
            if Path::new(path).exists() {
                if let Ok(cache) = load_tilesets(Path::new(path)) {
                    app.tileset_cache = Some(cache);
                    app.tileset_path = Some(PathBuf::from(path));
                    break;
                }
            }
        }

        app
    }

    /// Attempts to load a map file
    fn load_map_file(&mut self, path: PathBuf) {
        match load_map(&path) {
            Ok(mut map) => {
                // If we have a tileset cache, attach it to the map
                if let Some(cache) = &self.tileset_cache {
                    map.set_tileset_cache(cache.clone());
                }

                self.map = Some(map);
                self.map_path = Some(path);
                self.error_message = None;
                self.map_texture = None; // Will be recreated on next frame
            }
            Err(MapLoadError::IoError(e)) => {
                self.error_message = Some(format!("Failed to read map file: {}", e));
            }
            Err(MapLoadError::InvalidFormat(msg)) => {
                self.error_message = Some(format!("Invalid map format: {}", msg));
            }
            Err(MapLoadError::UnsupportedVersion(ver)) => {
                self.error_message = Some(format!("Unsupported map version: {}", ver));
            }
            Err(MapLoadError::Op2UtilityError(e)) => {
                self.error_message = Some(format!("Op2Utility error: {}", e));
            }
            Err(e) => {
                self.error_message = Some(format!("Error loading map: {}", e));
            }
        }
    }

    /// Shows the main menu bar
    fn show_menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open Map...").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Map Files", &["map"])
                        .add_filter("Image", &["png", "jpg", "jpeg", "bmp"])
                        .pick_file()
                    {
                        self.load_map_file(path);
                        ui.close_menu();
                    }
                }
                if ui.button("Load Tilesets...").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Zip Files", &["zip"])
                        .pick_file()
                    {
                        match load_tilesets(&path) {
                            Ok(cache) => {
                                self.tileset_cache = Some(cache.clone());
                                self.tileset_path = Some(path);

                                // Update the map with the new tileset cache if it exists
                                if let Some(map) = &mut self.map {
                                    map.set_tileset_cache(cache);
                                }

                                self.error_message = None;
                            }
                            Err(e) => {
                                self.error_message =
                                    Some(format!("Failed to load tilesets: {}", e));
                            }
                        }
                        ui.close_menu();
                    }
                }
                if ui.button("Settings").clicked() {
                    self.settings_open = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("View", |ui| {
                let config = self.map_view.config_mut();
                ui.add(egui::Slider::new(&mut config.zoom_level, 0.1..=5.0).text("Zoom"));
                ui.checkbox(&mut config.show_grid, "Show Grid");
                ui.checkbox(&mut config.use_tilesets, "Use Tilesets");

                ui.separator();
                let mut grid_rgb = [
                    config.grid_color.r() as f32 / 255.0,
                    config.grid_color.g() as f32 / 255.0,
                    config.grid_color.b() as f32 / 255.0,
                ];
                let mut bg_rgb = [
                    config.background_color.r() as f32 / 255.0,
                    config.background_color.g() as f32 / 255.0,
                    config.background_color.b() as f32 / 255.0,
                ];
                ui.color_edit_button_rgb(&mut grid_rgb);
                ui.color_edit_button_rgb(&mut bg_rgb);
                config.grid_color = egui::Color32::from_rgb(
                    (grid_rgb[0] * 255.0) as u8,
                    (grid_rgb[1] * 255.0) as u8,
                    (grid_rgb[2] * 255.0) as u8,
                );
                config.background_color = egui::Color32::from_rgb(
                    (bg_rgb[0] * 255.0) as u8,
                    (bg_rgb[1] * 255.0) as u8,
                    (bg_rgb[2] * 255.0) as u8,
                );
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About...").clicked() {
                    self.about_open = true;
                    ui.close_menu();
                }
            });
        });
    }

    /// Shows the settings window
    fn show_settings(&mut self, ctx: &egui::Context) {
        egui::Window::new("Settings")
            .open(&mut self.settings_open)
            .show(ctx, |ui| {
                let config = self.map_view.config_mut();

                ui.heading("Display");
                ui.add(egui::Slider::new(&mut config.cell_size, 16.0..=64.0).text("Cell Size"));
                ui.checkbox(&mut config.show_grid, "Show Grid");
                ui.checkbox(&mut config.use_tilesets, "Use Tilesets");

                if let Some(path) = &self.tileset_path {
                    ui.label(format!("Tileset: {}", path.display()));
                } else {
                    ui.label("No tileset loaded");
                }

                ui.separator();
                ui.heading("Colors");
                ui.horizontal(|ui| {
                    ui.label("Grid:");
                    let mut grid_rgb = [
                        config.grid_color.r() as f32 / 255.0,
                        config.grid_color.g() as f32 / 255.0,
                        config.grid_color.b() as f32 / 255.0,
                    ];
                    ui.color_edit_button_rgb(&mut grid_rgb);
                    config.grid_color = egui::Color32::from_rgb(
                        (grid_rgb[0] * 255.0) as u8,
                        (grid_rgb[1] * 255.0) as u8,
                        (grid_rgb[2] * 255.0) as u8,
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Background:");
                    let mut bg_rgb = [
                        config.background_color.r() as f32 / 255.0,
                        config.background_color.g() as f32 / 255.0,
                        config.background_color.b() as f32 / 255.0,
                    ];
                    ui.color_edit_button_rgb(&mut bg_rgb);
                    config.background_color = egui::Color32::from_rgb(
                        (bg_rgb[0] * 255.0) as u8,
                        (bg_rgb[1] * 255.0) as u8,
                        (bg_rgb[2] * 255.0) as u8,
                    );
                });
            });
    }
}

impl eframe::App for MapViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.show_menu_bar(ui);
        });

        if self.settings_open {
            self.show_settings(ctx);
        }

        if self.about_open {
            egui::Window::new("About OP2MapViewer")
                .collapsible(false)
                .resizable(false)
                .default_size([280.0, 100.0])
                .open(&mut self.about_open)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("OP2MapViewer");
                        ui.label("Author: Arthur Bowers");
                        ui.label("Written in Rust");
                    });
                });
        }

        egui::SidePanel::right("info_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                if let Some(map) = &self.map {
                    // Show map info
                    ui.heading(&map.info.name);
                    if !map.info.description.is_empty() {
                        ui.label(&map.info.description);
                        ui.separator();
                    }

                    // Show cell info based on selected position
                    if let Some((x, y)) = self.selected_cell_pos {
                        if let Some(cell) = map.get_cell(x, y) {
                            self.cell_info.show(ui, Some(cell));
                        }
                    } else {
                        self.cell_info.show(ui, None);
                    }
                } else {
                    ui.heading("No Map Loaded");
                    ui.label("Open a map file to begin");
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            }

            if let Some(map) = &self.map {
                if let Some(pos) = self.map_view.show(ui, map) {
                    self.selected_cell_pos = Some((pos.x, pos.y));
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.heading("Welcome to OP2MapViewer");
                    ui.label("Open a map file to begin viewing");
                });
            }
        });
    }
}
