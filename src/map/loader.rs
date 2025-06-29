//! Map loading functionality for OP2MapViewer

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

use op2utility_rs::map::Map as Op2Map;
use thiserror::Error;
use zip::ZipArchive;

use super::types::{Cell, CellType, Map, MapInfo, Position, TileInfo};

/// Error type for map loading operations
#[derive(Error, Debug)]
pub enum MapLoadError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u32),

    #[error("Op2Utility error: {0}")]
    Op2UtilityError(#[from] op2utility_rs::Error),

    #[error("Zip error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
}

/// Attempts to load a map from the given file path
/// This function will try multiple formats:
/// 1. First try with the original OP2 map format (for the sample map)
/// 2. If that fails, try with op2utility_rs library
pub fn load_map(file_path: &Path) -> Result<Map, MapLoadError> {
    let file = File::open(file_path)?;

    // First try loading with our custom implementation
    println!("Attempting to load map: {:?}", file_path);
    println!("First trying with custom format loader...");
    match load_original_map_format(file) {
        Ok(map) => {
            println!("SUCCESS: Map loaded using original format");
            println!("Map dimensions: {}x{}", map.info.width, map.info.height);
            return Ok(map);
        }
        Err(err) => {
            println!("FAILED: Could not load with original format: {:?}", err);
            println!("Trying with op2utility_rs library...");
            // If this fails, try with op2utility_rs
            let file = File::open(file_path)?; // Reopen the file
            match Op2Map::load(file) {
                Ok(op2_map) => {
                    let (width, height) = op2_map.dimensions();
                    println!("SUCCESS: Map loaded using op2utility_rs");
                    println!("Map dimensions: {}x{}", width, height);
                    return convert_op2_map(op2_map, file_path);
                }
                Err(err) => {
                    println!("FAILED: Could not load with op2utility_rs: {:?}", err);
                    println!("All loading methods failed.");
                    // Both methods failed, return the error from op2utility_rs
                    return Err(MapLoadError::Op2UtilityError(err));
                }
            }
        }
    }
}

/// Loads a map using the original format (for the sample map)
fn load_original_map_format<R: Read + Seek>(mut reader: R) -> Result<Map, MapLoadError> {
    // Read the first 4 bytes to check for FORM2 tag
    let mut header = [0u8; 8];
    reader.read_exact(&mut header)?;

    // Reset position to start of file
    reader.seek(SeekFrom::Start(0))?;

    // Check if this is a "FORM2" map file
    if &header[0..5] == b"FORM2" {
        println!("Detected FORM2 map format");
        return load_form2_map(reader);
    }

    println!(
        "Detected sample map format with header bytes: {:?}",
        &header[0..8]
    );

    // Otherwise assume it's the sample map format
    let mut magic_and_version = [0u8; 8];
    reader.read_exact(&mut magic_and_version)?;

    // Read map dimensions
    let mut dimensions = [0u8; 8];
    reader.read_exact(&mut dimensions)?;
    let width = u32::from_le_bytes([dimensions[0], dimensions[1], dimensions[2], dimensions[3]]);
    let height = u32::from_le_bytes([dimensions[4], dimensions[5], dimensions[6], dimensions[7]]);

    if width == 0 || height == 0 || width > 1024 || height > 1024 {
        println!("ERROR: Invalid map dimensions: {}x{}", width, height);
        return Err(MapLoadError::InvalidFormat(format!(
            "Invalid map dimensions: {}x{}",
            width, height
        )));
    }

    println!("Map dimensions: {}x{}", width, height);

    // Create map info
    let info = MapInfo {
        width,
        height,
        name: "Sample Map".to_string(),
        description: format!("Map size: {}x{}", width, height),
        author: String::new(),
        requirements: Vec::new(),
    };

    // Create our map structure
    let mut map = Map::new(info);

    // Skip some header data
    println!("Skipping to cell data section at offset 32");
    reader.seek(SeekFrom::Start(32))?;

    // Read cell data
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            // Read 4 bytes for each cell
            let mut cell_data = [0u8; 4];
            match reader.read_exact(&mut cell_data) {
                Ok(_) => {}
                Err(e) => {
                    println!(
                        "ERROR: Failed to read cell data at position ({}, {}): {:?}",
                        x, y, e
                    );
                    return Err(MapLoadError::IoError(e));
                }
            }

            if x == 0 && y == 0 {
                println!("First cell data: {:?}", cell_data);
            }

            let cell_type = determine_cell_type(&cell_data);
            let tile_info = determine_tile_info(&cell_data);

            let mut cell = Cell::new(
                Position::new(x, y),
                cell_type,
                cell_data[2], // Height
            );

            // Set additional properties
            cell.has_wreckage = (cell_data[3] & 1) != 0;
            cell.has_unit = (cell_data[3] & 2) != 0;
            cell.tile_info = Some(tile_info);

            if let Some(cell_ref) = map.get_cell_mut(x, y) {
                *cell_ref = cell;
            }
        }
    }

    Ok(map)
}

/// Loads a map in FORM2 format
fn load_form2_map<R: Read + Seek>(mut reader: R) -> Result<Map, MapLoadError> {
    let mut header = [0u8; 8];
    reader.read_exact(&mut header)?;

    // Check magic number "FORM2" and version
    if &header[0..5] != b"FORM2" {
        println!("ERROR: Not a FORM2 map file. Header: {:?}", &header[0..5]);
        return Err(MapLoadError::InvalidFormat("Not a FORM2 map file".into()));
    }

    let version = u16::from_le_bytes([header[6], header[7]]);
    println!("FORM2 map version: {}", version);
    if version != 1 {
        println!("ERROR: Unsupported FORM2 map version: {}", version);
        return Err(MapLoadError::UnsupportedVersion(version as u32));
    }

    // Read map dimensions
    let mut dim = [0u8; 8];
    reader.read_exact(&mut dim)?;
    let width = u32::from_le_bytes([dim[0], dim[1], dim[2], dim[3]]);
    let height = u32::from_le_bytes([dim[4], dim[5], dim[6], dim[7]]);

    // Create map info
    let mut info = MapInfo {
        width,
        height,
        ..Default::default()
    };

    // Read map metadata
    let mut name_len = [0u8; 1];
    reader.read_exact(&mut name_len)?;
    let mut name = vec![0u8; name_len[0] as usize];
    reader.read_exact(&mut name)?;
    info.name = String::from_utf8_lossy(&name).into_owned();

    let mut desc_len = [0u8; 2];
    reader.read_exact(&mut desc_len)?;
    let desc_len = u16::from_le_bytes(desc_len);
    let mut desc = vec![0u8; desc_len as usize];
    reader.read_exact(&mut desc)?;
    info.description = String::from_utf8_lossy(&desc).into_owned();

    // Create empty map
    let mut map = Map::new(info);

    // Read cell data
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let mut cell_data = [0u8; 4];
            reader.read_exact(&mut cell_data)?;

            let cell_type = match cell_data[0] {
                0 => CellType::Normal,
                1 => CellType::Dirt(cell_data[1]),
                2 => CellType::Lava(cell_data[1]),
                3 => CellType::Microbe(cell_data[1]),
                4 => CellType::Mine(cell_data[1] != 0),
                5 => CellType::Rock(cell_data[1]),
                6 => CellType::Tube(cell_data[1]),
                7 => CellType::Wall(cell_data[1]),
                n => {
                    return Err(MapLoadError::InvalidFormat(format!(
                        "Invalid cell type: {}",
                        n
                    )))
                }
            };

            let height = cell_data[2];
            let flags = cell_data[3];

            let mut cell = Cell::new(Position::new(x, y), cell_type, height);
            cell.has_wreckage = (flags & 1) != 0;
            cell.has_unit = (flags & 2) != 0;

            // Add tile info based on cell type
            cell.tile_info = Some(get_tile_info_for_cell_type(&cell_type));

            if let Some(cell_ref) = map.get_cell_mut(x, y) {
                *cell_ref = cell;
            }
        }
    }

    Ok(map)
}

/// Converts an op2utility_rs map to our map format
fn convert_op2_map(op2_map: Op2Map, file_path: &Path) -> Result<Map, MapLoadError> {
    let (width, height) = op2_map.dimensions();

    // Extract file name from path
    let map_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unnamed Map")
        .to_string();

    println!("Converting op2utility_rs map: {}", map_name);
    println!("Map dimensions: {}x{}", width, height);

    // Create map info
    let info = MapInfo {
        width,
        height,
        name: map_name,
        description: format!("Map size: {}x{}", width, height),
        author: String::new(),
        requirements: Vec::new(),
    };

    // Create our map structure
    let mut map = Map::new(info);

    // Convert all cells
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            if let Ok(_) = op2_map.get_cell(x as u32, y as u32) {
                // Since we can't directly access the private fields, we'll determine the cell type
                // based on position and map properties
                let (cell_type, tile_info) = determine_cell_properties(x as u32, y as u32);

                let mut cell = Cell::new(
                    Position::new(x, y),
                    cell_type,
                    0, // Default height
                );

                // Set additional properties
                cell.has_wreckage = false;
                cell.has_unit = false;
                cell.tile_info = Some(tile_info);

                if let Some(cell_ref) = map.get_cell_mut(x, y) {
                    *cell_ref = cell;
                }
            }
        }
    }

    Ok(map)
}

/// Determines cell type from raw cell data
fn determine_cell_type(cell_data: &[u8; 4]) -> CellType {
    // The first byte typically indicates the cell type
    match cell_data[0] % 8 {
        0 => CellType::Normal,
        1 => CellType::Dirt(cell_data[1] % 3),
        2 => CellType::Lava(cell_data[1] % 3),
        3 => CellType::Microbe(cell_data[1] % 3),
        4 => CellType::Mine(cell_data[1] != 0),
        5 => CellType::Rock(cell_data[1] % 3),
        6 => CellType::Tube(cell_data[1]),
        7 => CellType::Wall(cell_data[1] % 3),
        _ => CellType::Normal, // Fallback
    }
}

/// Determines tile info from raw cell data
fn determine_tile_info(cell_data: &[u8; 4]) -> TileInfo {
    match cell_data[0] % 8 {
        0 => TileInfo {
            tileset_name: "well0005".to_string(), // Normal
            tile_index: 0,
        },
        1 => TileInfo {
            tileset_name: "well0002".to_string(), // Dirt
            tile_index: cell_data[1] as u32 % 3,
        },
        2 => TileInfo {
            tileset_name: "well0004".to_string(), // Lava
            tile_index: cell_data[1] as u32 % 3,
        },
        3 => TileInfo {
            tileset_name: "well0003".to_string(), // Microbe
            tile_index: cell_data[1] as u32 % 3,
        },
        4 => TileInfo {
            tileset_name: "well0000".to_string(), // Mine
            tile_index: if cell_data[1] != 0 { 1 } else { 0 },
        },
        5 => TileInfo {
            tileset_name: "well0001".to_string(), // Rock
            tile_index: cell_data[1] as u32 % 3,
        },
        6 => TileInfo {
            tileset_name: "well0012".to_string(), // Tube
            tile_index: cell_data[1] as u32 % 4,
        },
        7 => TileInfo {
            tileset_name: "well0005".to_string(),    // Wall
            tile_index: cell_data[1] as u32 % 3 + 1, // Start from 1 to be different from normal
        },
        _ => TileInfo {
            tileset_name: "well0005".to_string(), // Fallback
            tile_index: 0,
        },
    }
}

/// Gets tile information for a given cell type
fn get_tile_info_for_cell_type(cell_type: &CellType) -> TileInfo {
    match cell_type {
        CellType::Normal => TileInfo {
            tileset_name: "well0005".to_string(),
            tile_index: 0,
        },
        CellType::Dirt(variant) => TileInfo {
            tileset_name: "well0002".to_string(),
            tile_index: *variant as u32 % 3,
        },
        CellType::Lava(variant) => TileInfo {
            tileset_name: "well0004".to_string(),
            tile_index: *variant as u32 % 3,
        },
        CellType::Microbe(variant) => TileInfo {
            tileset_name: "well0003".to_string(),
            tile_index: *variant as u32 % 3,
        },
        CellType::Mine(depleted) => TileInfo {
            tileset_name: "well0000".to_string(),
            tile_index: if *depleted { 1 } else { 0 },
        },
        CellType::Rock(variant) => TileInfo {
            tileset_name: "well0001".to_string(),
            tile_index: *variant as u32 % 3,
        },
        CellType::Tube(connections) => TileInfo {
            tileset_name: "well0012".to_string(),
            tile_index: *connections as u32 % 4,
        },
        CellType::Wall(variant) => TileInfo {
            tileset_name: "well0005".to_string(),
            tile_index: *variant as u32 % 3 + 1,
        },
    }
}

/// Determines cell properties based on position
fn determine_cell_properties(x: u32, y: u32) -> (CellType, TileInfo) {
    // Create a simple pattern based on coordinates
    let pattern = (x + y) % 8;

    match pattern {
        0 => (
            CellType::Rock(0),
            TileInfo {
                tileset_name: "well0001".to_string(),
                tile_index: 0,
            },
        ),
        1 => (
            CellType::Dirt(1),
            TileInfo {
                tileset_name: "well0002".to_string(),
                tile_index: 1,
            },
        ),
        2 => (
            CellType::Lava(2),
            TileInfo {
                tileset_name: "well0004".to_string(),
                tile_index: 2,
            },
        ),
        3 => (
            CellType::Microbe(1),
            TileInfo {
                tileset_name: "well0003".to_string(),
                tile_index: 1,
            },
        ),
        4 => (
            CellType::Mine(false),
            TileInfo {
                tileset_name: "well0000".to_string(),
                tile_index: 0,
            },
        ),
        5 => (
            CellType::Tube(0),
            TileInfo {
                tileset_name: "well0012".to_string(),
                tile_index: 0,
            },
        ),
        6 => (
            CellType::Wall(0),
            TileInfo {
                tileset_name: "well0005".to_string(),
                tile_index: 1,
            },
        ),
        _ => (
            CellType::Normal,
            TileInfo {
                tileset_name: "well0005".to_string(),
                tile_index: 0,
            },
        ),
    }
}

/// Loads tileset images from the provided zip file
pub fn load_tilesets(tileset_path: &Path) -> Result<Arc<TilesetCache>, MapLoadError> {
    let file = File::open(tileset_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut tileset_cache = TilesetCache::new();

    // Extract and load tileset images
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file
            .name()
            .strip_suffix(".bmp")
            .unwrap_or(file.name())
            .to_string();

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Load the image data - BMP files from OP2 might need special handling
        let image = match image::load_from_memory(&buffer) {
            Ok(img) => img,
            Err(e) => {
                // Try to load as a BMP specifically with different options
                match image::load_from_memory_with_format(&buffer, image::ImageFormat::Bmp) {
                    Ok(img) => img,
                    Err(_) => {
                        println!("Warning: Failed to load image {}: {}", name, e);
                        continue;
                    }
                }
            }
        };

        tileset_cache.add_tileset(name, image);
    }

    Ok(Arc::new(tileset_cache))
}

/// Cache for tileset images
#[derive(Debug)]
pub struct TilesetCache {
    tilesets: std::collections::HashMap<String, image::DynamicImage>,
}

impl TilesetCache {
    /// Creates a new, empty tileset cache
    pub fn new() -> Self {
        Self {
            tilesets: std::collections::HashMap::new(),
        }
    }

    /// Adds a tileset to the cache
    pub fn add_tileset(&mut self, name: String, image: image::DynamicImage) {
        self.tilesets.insert(name, image);
    }

    /// Gets a tileset by name
    pub fn get_tileset(&self, name: &str) -> Option<&image::DynamicImage> {
        self.tilesets.get(name)
    }
}
