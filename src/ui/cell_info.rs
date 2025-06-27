//! Cell information panel for OP2MapViewer

use eframe::egui::{Color32, RichText, Ui};
use crate::map::types::{Cell, CellType};

/// Widget for displaying detailed cell information
pub struct CellInfoPanel {
    show_height_gradient: bool,
    show_details: bool,
}

impl Default for CellInfoPanel {
    fn default() -> Self {
        Self {
            show_height_gradient: true,
            show_details: true,
        }
    }
}

impl CellInfoPanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the cell information panel
    pub fn show(&mut self, ui: &mut Ui, cell: Option<&Cell>) {
        ui.heading("Cell Information");

        if let Some(cell) = cell {
            // Position
            ui.horizontal(|ui| {
                ui.label("Position:");
                ui.label(RichText::new(format!("({}, {})", cell.position.x, cell.position.y))
                    .color(Color32::LIGHT_BLUE));
            });

            // Cell type with color coding
            ui.horizontal(|ui| {
                ui.label("Type:");
                let (text, color) = match cell.cell_type {
                    CellType::Normal => (String::from("Normal Ground"), Color32::from_gray(180)),
                    CellType::Lava(variant) => (
                        format!("Lava Type {}", variant),
                        Color32::RED,
                    ),
                    CellType::Microbe(stage) => (
                        format!("Microbe Stage {}", stage),
                        Color32::GREEN,
                    ),
                    CellType::Mine(depleted) => (
                        String::from(if depleted { "Depleted Mine" } else { "Active Mine" }),
                        if depleted { Color32::GRAY } else { Color32::YELLOW },
                    ),
                    CellType::Dirt(variant) => (
                        format!("Dirt Type {}", variant),
                        Color32::from_rgb(139, 69, 19),
                    ),
                    CellType::Rock(variant) => (
                        format!("Rock Type {}", variant),
                        Color32::GRAY,
                    ),
                    CellType::Tube(connections) => (
                        format!("Tube (0b{:08b})", connections),
                        Color32::BLUE,
                    ),
                    CellType::Wall(variant) => (
                        format!("Wall Type {}", variant),
                        Color32::WHITE,
                    ),
                };
                ui.label(RichText::new(text).color(color));
            });

            // Height with optional gradient visualization
            ui.horizontal(|ui| {
                ui.label("Height:");
                if self.show_height_gradient {
                    let height_color = Color32::from_gray(
                        ((cell.height as f32 / 255.0) * 200.0 + 55.0) as u8
                    );
                    ui.label(RichText::new(format!("{}", cell.height))
                        .color(height_color));
                } else {
                    ui.label(format!("{}", cell.height));
                }
            });

            // Additional details
            if self.show_details {
                if cell.has_wreckage {
                    ui.label(RichText::new("Contains wreckage")
                        .color(Color32::DARK_RED));
                }
                if cell.has_unit {
                    ui.label(RichText::new("Contains unit")
                        .color(Color32::LIGHT_GREEN));
                }
            }

            // Settings
            ui.separator();
            ui.checkbox(&mut self.show_height_gradient, "Show height gradient");
            ui.checkbox(&mut self.show_details, "Show additional details");
        } else {
            ui.label("Hover over a cell to see information");
        }
    }
}
