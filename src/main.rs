use eframe::egui;
use egui::{ColorImage, TextureHandle};

use rfd::FileDialog;
use std::path::PathBuf;

struct MapViewerApp {
    map_image: Option<ColorImage>,
    map_texture: Option<TextureHandle>,
    map_path: Option<PathBuf>,
    error_message: Option<String>,
}

impl Default for MapViewerApp {
    fn default() -> Self {
        Self {
            map_image: None,
            map_texture: None,
            map_path: None,
            error_message: None,
        }
    }
}

impl MapViewerApp {
    fn update_with_frame(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Open Map Image...").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Image", &["png", "jpg", "jpeg", "bmp"])
                        .pick_file()
                    {
                        match load_image_as_color_image(&path) {
                            Ok(img) => {
                                self.map_image = Some(img);
                                self.map_path = Some(path);
                                self.error_message = None;
                                self.map_texture = None; // Will be recreated on next frame
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Failed to load image: {e}"));
                            }
                        }
                    }
                }
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref msg) = self.error_message {
                ui.colored_label(egui::Color32::RED, msg);
            }

            if let Some(ref img) = self.map_image {
                // Only create the texture once per image
                if self.map_texture.is_none() {
                    self.map_texture = Some(ui.ctx().load_texture(
                        "map_texture",
                        img.clone(),
                        egui::TextureOptions::default(),
                    ));
                }
                if let Some(ref tex) = self.map_texture {
                    ui.heading(
                        self.map_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("Loaded Map Image"),
                    );
                    ui.add(egui::Image::new((tex.id(), tex.size_vec2())));
                }
            } else {
                ui.label("Open a map image (PNG/JPG/BMP) to view it here.");
            }
        });
    }
}

/// Loads an image file and converts it to egui's ColorImage.
/// This is a stub for map loading; in a real app, parse OP2 map files and render them.
fn load_image_as_color_image(path: &PathBuf) -> Result<ColorImage, String> {
    let dyn_img = image::open(path).map_err(|e| e.to_string())?;
    let size = [dyn_img.width() as usize, dyn_img.height() as usize];
    let rgba = dyn_img.to_rgba8();
    let pixels = rgba
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    Ok(ColorImage {
        size,
        pixels,
    })
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "OP2MapViewerRust (Minimal Prototype)",
        options,
        Box::new(|_cc| Box::new(EguiAppWrapper {
            app: MapViewerApp::default(),
        })),
    );
}

/// Wrapper struct to adapt MapViewerApp to the new eframe App trait
struct EguiAppWrapper {
    app: MapViewerApp,
}

impl eframe::App for EguiAppWrapper {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Pass frame to update so we can call frame.close()
        self.app.update_with_frame(ctx, frame);
    }
}
