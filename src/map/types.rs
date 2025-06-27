//! Map data structures for OP2MapViewer

use std::fmt;

/// A 2D position in the map
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Map cell types in Outpost 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Normal,
    Lava(u8),     // Variant indicates lava color/type
    Microbe(u8),  // Variant indicates microbe growth stage
    Mine(bool),   // Boolean indicates if mine is depleted
    Dirt(u8),     // Variant indicates dirt type
    Rock(u8),     // Variant indicates rock type
    Tube(u8),     // Variant indicates tube connections
    Wall(u8),     // Variant indicates wall type
}

impl fmt::Display for CellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellType::Normal => write!(f, "Normal Ground"),
            CellType::Lava(variant) => write!(f, "Lava Type {}", variant),
            CellType::Microbe(stage) => write!(f, "Microbe Growth Stage {}", stage),
            CellType::Mine(depleted) => write!(f, "Mine ({})", if *depleted { "Depleted" } else { "Active" }),
            CellType::Dirt(variant) => write!(f, "Dirt Type {}", variant),
            CellType::Rock(variant) => write!(f, "Rock Type {}", variant),
            CellType::Tube(connections) => write!(f, "Tube (Connections: {:08b})", connections),
            CellType::Wall(variant) => write!(f, "Wall Type {}", variant),
        }
    }
}

/// A single cell in the map
#[derive(Debug, Clone)]
pub struct Cell {
    pub position: Position,
    pub cell_type: CellType,
    pub height: u8,
    pub has_wreckage: bool,
    pub has_unit: bool,
}

impl Cell {
    pub fn new(position: Position, cell_type: CellType, height: u8) -> Self {
        Self {
            position,
            cell_type,
            height,
            has_wreckage: false,
            has_unit: false,
        }
    }

    pub fn description(&self) -> String {
        format!(
            "Position: ({}, {})\nType: {}\nHeight: {}\n{}{}",
            self.position.x,
            self.position.y,
            self.cell_type,
            self.height,
            if self.has_wreckage { "Contains wreckage\n" } else { "" },
            if self.has_unit { "Contains unit\n" } else { "" }
        )
    }
}

/// Map metadata and dimensions
#[derive(Debug, Clone)]
pub struct MapInfo {
    pub width: u32,
    pub height: u32,
    pub name: String,
    pub description: String,
    pub author: String,
    pub requirements: Vec<String>,
}

impl Default for MapInfo {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            name: String::new(),
            description: String::new(),
            author: String::new(),
            requirements: Vec::new(),
        }
    }
}

/// Complete map data
#[derive(Debug, Clone)]
pub struct Map {
    pub info: MapInfo,
    pub cells: Vec<Vec<Cell>>,
}

impl Map {
    pub fn new(info: MapInfo) -> Self {
        let cells = vec![vec![
            Cell::new(
                Position::new(0, 0),
                CellType::Normal,
                0
            );
            info.width as usize];
            info.height as usize
        ];
        Self { info, cells }
    }

    pub fn get_cell(&self, x: i32, y: i32) -> Option<&Cell> {
        if x < 0 || y < 0 {
            return None;
        }
        self.cells
            .get(y as usize)
            .and_then(|row| row.get(x as usize))
    }

    pub fn get_cell_mut(&mut self, x: i32, y: i32) -> Option<&mut Cell> {
        if x < 0 || y < 0 {
            return None;
        }
        self.cells
            .get_mut(y as usize)
            .and_then(|row| row.get_mut(x as usize))
    }
}
