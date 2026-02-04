//! WAD file loading and parsing - Pure Rust implementation!
//!
//! Phase 1.1: Load and parse Doom WAD files to extract map geometry.
//!
//! # Evolution Note
//!
//! We discovered the external `wad` crate didn't fit our needs, so we
//! implemented our own Pure Rust parser. This aligns with TRUE PRIMAL
//! principles: zero unnecessary external dependencies!
//!
//! The WAD format is simple:
//! - Header (12 bytes): type, lump count, directory offset
//! - Directory: array of lump entries (name, offset, size)
//! - Data: raw lump data

use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use tracing::{debug, info};

/// WAD file header
#[derive(Debug)]
struct WadHeader {
    wad_type: WadType,
    num_lumps: i32,
    dir_offset: i32,
}

/// WAD type (Iwad or Pwad)
///
/// Idiomatic Rust: Acronyms in type names use lowercase except initial letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WadType {
    Iwad, // Official game data (IWAD)
    Pwad, // Patch WAD (custom levels) (PWAD)
}

/// A lump (data chunk) in the WAD
///
/// Lumps contain raw data that can be accessed for resource loading.
#[derive(Debug, Clone)]
pub struct Lump {
    name: String,
    offset: i32,
    size: i32,
    data: Vec<u8>,
}

impl Lump {
    /// Get the lump name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the lump data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// A loaded WAD file with parsed data
pub struct WadData {
    /// Internal lumps storage for accessing raw WAD data
    /// Used by map parsing and resource loading
    lumps: Vec<Lump>,
    /// Parsed map data ready for rendering
    pub maps: Vec<MapData>,
}

impl WadData {
    /// Access a lump by name (used for resource loading)
    pub fn get_lump(&self, name: &str) -> Option<&Lump> {
        self.lumps.iter().find(|lump| lump.name == name)
    }

    /// Access all lumps (used for advanced WAD manipulation)
    pub fn lumps(&self) -> &[Lump] {
        &self.lumps
    }
}

/// Map geometry data
#[derive(Debug, Clone)]
pub struct MapData {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub linedefs: Vec<LineDef>,
    pub sectors: Vec<Sector>,
    pub things: Vec<Thing>,
}

/// 2D vertex
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub x: i16,
    pub y: i16,
}

/// Line definition (wall segment)
#[derive(Debug, Clone)]
pub struct LineDef {
    pub start_vertex: usize,
    pub end_vertex: usize,
    pub flags: u16,
    pub line_type: u16,
    pub sector_tag: u16,
}

/// Sector (floor/ceiling area)
#[derive(Debug, Clone)]
pub struct Sector {
    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_texture: String,
    pub ceiling_texture: String,
    pub light_level: u16,
}

/// Thing (player start, enemy, item, etc.)
#[derive(Debug, Clone)]
pub struct Thing {
    pub x: i16,
    pub y: i16,
    pub angle: u16,
    pub thing_type: u16,
    pub flags: u16,
}

impl WadData {
    /// Load a WAD file from disk - Pure Rust implementation!
    ///
    /// # Phase 1.1 Implementation
    ///
    /// We parse the binary WAD format directly:
    /// 1. Read header (type, lump count, directory offset)
    /// 2. Read directory (lump names, offsets, sizes)
    /// 3. Read lump data on demand
    /// 4. Parse map geometry from relevant lumps
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!("Loading WAD file: {}", path.display());

        let mut file = File::open(path)
            .with_context(|| format!("Failed to open WAD file: {}", path.display()))?;

        // Read header
        let header = Self::read_header(&mut file)?;
        info!(
            "WAD loaded successfully - Type: {:?}, Lumps: {}",
            header.wad_type, header.num_lumps
        );

        // Read directory
        let lumps = Self::read_directory(&mut file, &header)?;
        debug!("Read {} lump entries", lumps.len());

        // Parse maps
        let mut maps = Vec::new();

        for (i, lump) in lumps.iter().enumerate() {
            if is_map_marker(&lump.name) {
                info!("Found map: {}", lump.name);

                match parse_map(&lumps, i, &lump.name) {
                    Ok(map) => {
                        info!(
                            "Parsed map {} with {} vertices, {} linedefs",
                            map.name,
                            map.vertices.len(),
                            map.linedefs.len()
                        );
                        maps.push(map);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse map {}: {}", lump.name, e);
                    }
                }
            }
        }

        if maps.is_empty() {
            bail!("No valid maps found in WAD file");
        }

        info!("Loaded {} maps from WAD", maps.len());

        Ok(Self { lumps, maps })
    }

    /// Read the WAD header (12 bytes)
    fn read_header(file: &mut File) -> Result<WadHeader> {
        let mut header_bytes = [0u8; 12];
        file.read_exact(&mut header_bytes)
            .context("Failed to read WAD header")?;

        // Parse type (4 bytes ASCII)
        let type_str = std::str::from_utf8(&header_bytes[0..4]).context("Invalid WAD type")?;

        let wad_type = match type_str {
            "IWAD" => WadType::Iwad,
            "PWAD" => WadType::Pwad,
            _ => bail!("Unknown WAD type: {}", type_str),
        };

        // Parse lump count (4 bytes, little-endian i32)
        let num_lumps = i32::from_le_bytes([
            header_bytes[4],
            header_bytes[5],
            header_bytes[6],
            header_bytes[7],
        ]);

        // Parse directory offset (4 bytes, little-endian i32)
        let dir_offset = i32::from_le_bytes([
            header_bytes[8],
            header_bytes[9],
            header_bytes[10],
            header_bytes[11],
        ]);

        Ok(WadHeader {
            wad_type,
            num_lumps,
            dir_offset,
        })
    }

    /// Read the lump directory
    fn read_directory(file: &mut File, header: &WadHeader) -> Result<Vec<Lump>> {
        // Seek to directory
        file.seek(SeekFrom::Start(header.dir_offset as u64))
            .context("Failed to seek to directory")?;

        let mut lumps = Vec::new();

        // Each directory entry is 16 bytes
        for _ in 0..header.num_lumps {
            let mut entry_bytes = [0u8; 16];
            file.read_exact(&mut entry_bytes)
                .context("Failed to read directory entry")?;

            // Parse offset (4 bytes)
            let offset = i32::from_le_bytes([
                entry_bytes[0],
                entry_bytes[1],
                entry_bytes[2],
                entry_bytes[3],
            ]);

            // Parse size (4 bytes)
            let size = i32::from_le_bytes([
                entry_bytes[4],
                entry_bytes[5],
                entry_bytes[6],
                entry_bytes[7],
            ]);

            // Parse name (8 bytes, null-terminated ASCII)
            let name_end = entry_bytes[8..16].iter().position(|&b| b == 0).unwrap_or(8);
            let name = String::from_utf8_lossy(&entry_bytes[8..8 + name_end]).to_string();

            lumps.push(Lump {
                name,
                offset,
                size,
                data: Vec::new(), // Load on demand
            });
        }

        // Now load the data for each lump
        for lump in &mut lumps {
            if lump.size > 0 {
                file.seek(SeekFrom::Start(lump.offset as u64))
                    .context("Failed to seek to lump data")?;

                lump.data.resize(lump.size as usize, 0);
                file.read_exact(&mut lump.data)
                    .context("Failed to read lump data")?;
            }
        }

        Ok(lumps)
    }

    /// Get a map by name (e.g. "E1M1")
    pub fn get_map(&self, name: &str) -> Option<&MapData> {
        self.maps.iter().find(|m| m.name == name)
    }

    /// Get the first map (usually the starting map)
    pub fn first_map(&self) -> Option<&MapData> {
        self.maps.first()
    }
}

/// Check if a lump name is a map marker
fn is_map_marker(name: &str) -> bool {
    // Episode format: E#M# (e.g. E1M1)
    if name.len() == 4 && name.starts_with('E') && name.chars().nth(2) == Some('M') {
        return true;
    }

    // Doom 2 format: MAP## (e.g. MAP01)
    if name.len() == 5 && name.starts_with("MAP") {
        return true;
    }

    false
}

/// Parse a map from the lumps
fn parse_map(lumps: &[Lump], map_index: usize, map_name: &str) -> Result<MapData> {
    // Map data follows the map marker in this order:
    // THINGS, LINEDEFS, SIDEDEFS, VERTEXES, SEGS, SSECTORS, NODES, SECTORS, REJECT, BLOCKMAP

    let vertices = parse_vertices(lumps, map_index)?;
    let linedefs = parse_linedefs(lumps, map_index)?;
    let sectors = parse_sectors(lumps, map_index)?;
    let things = parse_things(lumps, map_index)?;

    Ok(MapData {
        name: map_name.to_string(),
        vertices,
        linedefs,
        sectors,
        things,
    })
}

/// Parse VERTEXES lump
fn parse_vertices(lumps: &[Lump], map_index: usize) -> Result<Vec<Vertex>> {
    let lump = find_map_lump(lumps, map_index, "VERTEXES")?;
    let data = &lump.data;

    let mut vertices = Vec::new();

    // Each vertex is 4 bytes: x (i16), y (i16)
    for chunk in data.chunks_exact(4) {
        let x = i16::from_le_bytes([chunk[0], chunk[1]]);
        let y = i16::from_le_bytes([chunk[2], chunk[3]]);
        vertices.push(Vertex { x, y });
    }

    Ok(vertices)
}

/// Parse LINEDEFS lump
fn parse_linedefs(lumps: &[Lump], map_index: usize) -> Result<Vec<LineDef>> {
    let lump = find_map_lump(lumps, map_index, "LINEDEFS")?;
    let data = &lump.data;

    let mut linedefs = Vec::new();

    // Each linedef is 14 bytes
    for chunk in data.chunks_exact(14) {
        let start_vertex = u16::from_le_bytes([chunk[0], chunk[1]]) as usize;
        let end_vertex = u16::from_le_bytes([chunk[2], chunk[3]]) as usize;
        let flags = u16::from_le_bytes([chunk[4], chunk[5]]);
        let line_type = u16::from_le_bytes([chunk[6], chunk[7]]);
        let sector_tag = u16::from_le_bytes([chunk[8], chunk[9]]);

        linedefs.push(LineDef {
            start_vertex,
            end_vertex,
            flags,
            line_type,
            sector_tag,
        });
    }

    Ok(linedefs)
}

/// Parse SECTORS lump
fn parse_sectors(lumps: &[Lump], map_index: usize) -> Result<Vec<Sector>> {
    let lump = find_map_lump(lumps, map_index, "SECTORS")?;
    let data = &lump.data;

    let mut sectors = Vec::new();

    // Each sector is 26 bytes
    for chunk in data.chunks_exact(26) {
        let floor_height = i16::from_le_bytes([chunk[0], chunk[1]]);
        let ceiling_height = i16::from_le_bytes([chunk[2], chunk[3]]);

        // Texture names are 8 bytes, null-terminated
        let floor_texture = parse_texture_name(&chunk[4..12]);
        let ceiling_texture = parse_texture_name(&chunk[12..20]);

        let light_level = u16::from_le_bytes([chunk[20], chunk[21]]);

        sectors.push(Sector {
            floor_height,
            ceiling_height,
            floor_texture,
            ceiling_texture,
            light_level,
        });
    }

    Ok(sectors)
}

/// Parse THINGS lump
fn parse_things(lumps: &[Lump], map_index: usize) -> Result<Vec<Thing>> {
    let lump = find_map_lump(lumps, map_index, "THINGS")?;
    let data = &lump.data;

    let mut things = Vec::new();

    // Each thing is 10 bytes
    for chunk in data.chunks_exact(10) {
        let x = i16::from_le_bytes([chunk[0], chunk[1]]);
        let y = i16::from_le_bytes([chunk[2], chunk[3]]);
        let angle = u16::from_le_bytes([chunk[4], chunk[5]]);
        let thing_type = u16::from_le_bytes([chunk[6], chunk[7]]);
        let flags = u16::from_le_bytes([chunk[8], chunk[9]]);

        things.push(Thing {
            x,
            y,
            angle,
            thing_type,
            flags,
        });
    }

    Ok(things)
}

/// Find a lump within a map
fn find_map_lump<'a>(lumps: &'a [Lump], map_index: usize, lump_name: &str) -> Result<&'a Lump> {
    // Map lumps follow the map marker
    for offset in 1..12 {
        let idx = map_index + offset;
        if idx < lumps.len() && lumps[idx].name == lump_name {
            return Ok(&lumps[idx]);
        }
    }

    bail!("Lump {} not found after map marker", lump_name)
}

/// Parse a texture name from 8 bytes
fn parse_texture_name(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_map_marker() {
        assert!(is_map_marker("E1M1"));
        assert!(is_map_marker("E2M5"));
        assert!(is_map_marker("MAP01"));
        assert!(is_map_marker("MAP32"));

        assert!(!is_map_marker("THINGS"));
        assert!(!is_map_marker("VERTEXES"));
        assert!(!is_map_marker("E1"));
        assert!(!is_map_marker("MAP"));
    }
}
