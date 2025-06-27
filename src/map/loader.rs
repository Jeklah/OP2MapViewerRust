//! Map loading functionality for OP2MapViewer

use std::path::Path;
use std::io::{self, Read, Seek};
use std::fs::File;

use super::types::{Map, MapInfo, Cell, CellType, Position};

/// Error type for map loading operations
#[derive(Debug)]
pub enum MapLoadError {
    IoError(io::Error),
    InvalidFormat(String),
    UnsupportedVersion(u16),
}

impl From<io::Error> for MapLoadError {
    fn from(err: io::Error) -> Self {
        MapLoadError::IoError(err)
    }
}

/// Attempts to load a map from the given file path
pub fn load_map(path: &Path) -> Result<Map, MapLoadError> {
    let mut file = File::open(path)?;
    let mut header = [0u8; 8];
    file.read_exact(&mut header)?;

    // Check magic number "FORM2" and version
    if &header[0..5] != b"FORM2" {
        return Err(MapLoadError::InvalidFormat("Not an OP2 map file".into()));
    }

    let version = u16::from_le_bytes([header[6], header[7]]);
    if version != 1 {
        return Err(MapLoadError::UnsupportedVersion(version));
    }

    // Read map dimensions
    let mut dim = [0u8; 8];
    file.read_exact(&mut dim)?;
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
    file.read_exact(&mut name_len)?;
    let mut name = vec![0u8; name_len[0] as usize];
    file.read_exact(&mut name)?;
    info.name = String::from_utf8_lossy(&name).into_owned();

    let mut desc_len = [0u8; 2];
    file.read_exact(&mut desc_len)?;
    let desc_len = u16::from_le_bytes(desc_len);
    let mut desc = vec![0u8; desc_len as usize];
    file.read_exact(&mut desc)?;
    info.description = String::from_utf8_lossy(&desc).into_owned();

    // Create empty map
    let mut map = Map::new(info);

    // Read cell data
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let mut cell_data = [0u8; 4];
            file.read_exact(&mut cell_data)?;

            let cell_type = match cell_data[0] {
                0 => CellType::Normal,
                1 => CellType::Dirt(cell_data[1]),
                2 => CellType::Lava(cell_data[1]),
                3 => CellType::Microbe(cell_data[1]),
                4 => CellType::Mine(cell_data[1] != 0),
                5 => CellType::Rock(cell_data[1]),
                6 => CellType::Tube(cell_data[1]),
                7 => CellType::Wall(cell_data[1]),
                n => return Err(MapLoadError::InvalidFormat(format!("Invalid cell type: {}", n))),
            };

            let height = cell_data[2];
            let flags = cell_data[3];

            let mut cell = Cell::new(Position::new(x, y), cell_type, height);
            cell.has_wreckage = (flags & 1) != 0;
            cell.has_unit = (flags & 2) != 0;

            if let Some(cell_ref) = map.get_cell_mut(x, y) {
                *cell_ref = cell;
            }
        }
    }

    Ok(map)
}

/// Attempts to load a preview thumbnail from a map file
pub fn load_map_preview(path: &Path) -> Result<(MapInfo, Vec<u8>), MapLoadError> {
    let mut file = File::open(path)?;

    // Skip to preview section (header + dimensions)
    file.seek(io::SeekFrom::Start(16))?;

    // Read map info first
    let mut name_len = [0u8; 1];
    file.read_exact(&mut name_len)?;
    let mut name = vec![0u8; name_len[0] as usize];
    file.read_exact(&mut name)?;
    let name = String::from_utf8_lossy(&name).into_owned();

    let mut desc_len = [0u8; 2];
    file.read_exact(&mut desc_len)?;
    let desc_len = u16::from_le_bytes(desc_len);
    let mut desc = vec![0u8; desc_len as usize];
    file.read_exact(&mut desc)?;
    let description = String::from_utf8_lossy(&desc).into_owned();

    // Read preview image size
    let mut preview_size = [0u8; 4];
    file.read_exact(&mut preview_size)?;
    let preview_size = u32::from_le_bytes(preview_size);

    // Read preview image data
    let mut preview_data = vec![0u8; preview_size as usize];
    file.read_exact(&mut preview_data)?;

    let info = MapInfo {
        name,
        description,
        ..Default::default()
    };

    Ok((info, preview_data))
}
