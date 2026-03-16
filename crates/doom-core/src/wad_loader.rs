// SPDX-License-Identifier: AGPL-3.0-or-later
//! WAD file loading and parsing - Pure Rust implementation
//!
//! Phase 1.1: Load and parse Doom WAD files to extract map geometry.
//!
//! The WAD format is simple:
//! - Header (12 bytes): type, lump count, directory offset
//! - Directory: array of lump entries (name, offset, size)
//! - Data: raw lump data

use crate::error::DoomError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use tracing::{debug, info};

#[derive(Debug)]
struct WadHeader {
    wad_type: WadType,
    num_lumps: i32,
    dir_offset: i32,
}

/// Idiomatic Rust: Acronyms in type names use lowercase except initial letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WadType {
    Iwad,
    Pwad,
}

/// A lump (data chunk) in the WAD.
#[derive(Debug, Clone)]
pub struct Lump {
    name: String,
    offset: i32,
    size: i32,
    data: Vec<u8>,
}

impl Lump {
    /// Get the lump name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the lump data.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// A loaded WAD file with parsed data.
pub struct WadData {
    lumps: Vec<Lump>,
    /// Parsed map data ready for rendering.
    pub maps: Vec<MapData>,
}

impl WadData {
    /// Access a lump by name (used for resource loading).
    #[must_use]
    pub fn get_lump(&self, name: &str) -> Option<&Lump> {
        self.lumps.iter().find(|lump| lump.name == name)
    }

    /// Access all lumps.
    #[must_use]
    pub fn lumps(&self) -> &[Lump] {
        &self.lumps
    }
}

/// Map geometry data.
#[derive(Debug, Clone)]
pub struct MapData {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub linedefs: Vec<LineDef>,
    pub sectors: Vec<Sector>,
    pub things: Vec<Thing>,
}

/// 2D vertex.
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub x: i16,
    pub y: i16,
}

/// Line definition (wall segment).
#[derive(Debug, Clone)]
pub struct LineDef {
    pub start_vertex: usize,
    pub end_vertex: usize,
    pub flags: u16,
    pub line_type: u16,
    pub sector_tag: u16,
}

/// Sector (floor/ceiling area).
#[derive(Debug, Clone)]
pub struct Sector {
    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_texture: String,
    pub ceiling_texture: String,
    pub light_level: u16,
}

/// Thing (player start, enemy, item, etc.).
#[derive(Debug, Clone)]
pub struct Thing {
    pub x: i16,
    pub y: i16,
    pub angle: u16,
    pub thing_type: u16,
    pub flags: u16,
}

impl WadData {
    /// Load a WAD file from disk.
    ///
    /// Parses the binary WAD format directly:
    /// 1. Read header (type, lump count, directory offset)
    /// 2. Read directory (lump names, offsets, sizes)
    /// 3. Read lump data
    /// 4. Parse map geometry from relevant lumps
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or contains no valid maps.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, DoomError> {
        let path = path.as_ref();
        info!("Loading WAD file: {}", path.display());

        let mut file = File::open(path).map_err(|e| {
            DoomError::WadNotFound(format!("Failed to open WAD file: {}: {e}", path.display()))
        })?;

        let header = Self::read_header(&mut file)?;
        info!(
            "WAD loaded successfully - Type: {:?}, Lumps: {}",
            header.wad_type, header.num_lumps
        );

        let lumps = Self::read_directory(&mut file, &header)?;
        debug!("Read {} lump entries", lumps.len());

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
                        tracing::warn!("Failed to parse map {}: {e}", lump.name);
                    }
                }
            }
        }

        if maps.is_empty() {
            return Err(DoomError::InvalidWad(
                "No valid maps found in WAD file".to_string(),
            ));
        }

        info!("Loaded {} maps from WAD", maps.len());

        Ok(Self { lumps, maps })
    }

    fn read_header(file: &mut File) -> Result<WadHeader, DoomError> {
        let mut header_bytes = [0u8; 12];
        file.read_exact(&mut header_bytes)
            .map_err(|e| DoomError::InvalidWad(format!("Failed to read WAD header: {e}")))?;

        let type_str = std::str::from_utf8(&header_bytes[0..4])
            .map_err(|e| DoomError::InvalidWad(format!("Invalid WAD type: {e}")))?;

        let wad_type = match type_str {
            "IWAD" => WadType::Iwad,
            "PWAD" => WadType::Pwad,
            _ => {
                return Err(DoomError::InvalidWad(format!(
                    "Unknown WAD type: {type_str}"
                )));
            }
        };

        let num_lumps = i32::from_le_bytes([
            header_bytes[4],
            header_bytes[5],
            header_bytes[6],
            header_bytes[7],
        ]);

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

    #[expect(
        clippy::cast_sign_loss,
        reason = "WAD format uses i32 for offsets/sizes that are always non-negative"
    )]
    fn read_directory(file: &mut File, header: &WadHeader) -> Result<Vec<Lump>, DoomError> {
        file.seek(SeekFrom::Start(header.dir_offset as u64))
            .map_err(|e| DoomError::InvalidWad(format!("Failed to seek to directory: {e}")))?;

        let mut lumps = Vec::new();

        for _ in 0..header.num_lumps {
            let mut entry_bytes = [0u8; 16];
            file.read_exact(&mut entry_bytes).map_err(|e| {
                DoomError::InvalidWad(format!("Failed to read directory entry: {e}"))
            })?;

            let offset = i32::from_le_bytes([
                entry_bytes[0],
                entry_bytes[1],
                entry_bytes[2],
                entry_bytes[3],
            ]);

            let size = i32::from_le_bytes([
                entry_bytes[4],
                entry_bytes[5],
                entry_bytes[6],
                entry_bytes[7],
            ]);

            let name_end = entry_bytes[8..16].iter().position(|&b| b == 0).unwrap_or(8);
            let name = String::from_utf8_lossy(&entry_bytes[8..8 + name_end]).to_string();

            lumps.push(Lump {
                name,
                offset,
                size,
                data: Vec::new(),
            });
        }

        for lump in &mut lumps {
            if lump.size > 0 {
                file.seek(SeekFrom::Start(lump.offset as u64))
                    .map_err(|e| {
                        DoomError::InvalidWad(format!("Failed to seek to lump data: {e}"))
                    })?;

                lump.data.resize(lump.size as usize, 0);
                file.read_exact(&mut lump.data)
                    .map_err(|e| DoomError::InvalidWad(format!("Failed to read lump data: {e}")))?;
            }
        }

        Ok(lumps)
    }

    /// Get a map by name (e.g. `"E1M1"`).
    #[must_use]
    pub fn get_map(&self, name: &str) -> Option<&MapData> {
        self.maps.iter().find(|m| m.name == name)
    }

    /// Get the first map (usually the starting map).
    #[must_use]
    pub fn first_map(&self) -> Option<&MapData> {
        self.maps.first()
    }
}

fn is_map_marker(name: &str) -> bool {
    if name.len() == 4 && name.starts_with('E') && name.chars().nth(2) == Some('M') {
        return true;
    }
    if name.len() == 5 && name.starts_with("MAP") {
        return true;
    }
    false
}

fn parse_map(lumps: &[Lump], map_index: usize, map_name: &str) -> Result<MapData, DoomError> {
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

fn parse_vertices(lumps: &[Lump], map_index: usize) -> Result<Vec<Vertex>, DoomError> {
    let lump = find_map_lump(lumps, map_index, "VERTEXES")?;
    let data = &lump.data;

    let mut vertices = Vec::new();

    for chunk in data.chunks_exact(4) {
        let x = i16::from_le_bytes([chunk[0], chunk[1]]);
        let y = i16::from_le_bytes([chunk[2], chunk[3]]);
        vertices.push(Vertex { x, y });
    }

    Ok(vertices)
}

fn parse_linedefs(lumps: &[Lump], map_index: usize) -> Result<Vec<LineDef>, DoomError> {
    let lump = find_map_lump(lumps, map_index, "LINEDEFS")?;
    let data = &lump.data;

    let mut linedefs = Vec::new();

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

fn parse_sectors(lumps: &[Lump], map_index: usize) -> Result<Vec<Sector>, DoomError> {
    let lump = find_map_lump(lumps, map_index, "SECTORS")?;
    let data = &lump.data;

    let mut sectors = Vec::new();

    for chunk in data.chunks_exact(26) {
        let floor_height = i16::from_le_bytes([chunk[0], chunk[1]]);
        let ceiling_height = i16::from_le_bytes([chunk[2], chunk[3]]);
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

fn parse_things(lumps: &[Lump], map_index: usize) -> Result<Vec<Thing>, DoomError> {
    let lump = find_map_lump(lumps, map_index, "THINGS")?;
    let data = &lump.data;

    let mut things = Vec::new();

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

fn find_map_lump<'a>(
    lumps: &'a [Lump],
    map_index: usize,
    lump_name: &str,
) -> Result<&'a Lump, DoomError> {
    for offset in 1..12 {
        let idx = map_index + offset;
        if idx < lumps.len() && lumps[idx].name == lump_name {
            return Ok(&lumps[idx]);
        }
    }

    Err(DoomError::InvalidWad(format!(
        "Lump {lump_name} not found after map marker"
    )))
}

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

    #[expect(
        clippy::cast_possible_wrap,
        reason = "WAD test offsets are small and fit in i32"
    )]
    fn dir_entry(off: u32, size: i32, name: &str) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&(off as i32).to_le_bytes());
        bytes[4..8].copy_from_slice(&size.to_le_bytes());
        let name_bytes = name.as_bytes();
        bytes[8..8 + name_bytes.len().min(8)].copy_from_slice(name_bytes);
        bytes
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        reason = "WAD test data uses small known sizes that fit in all integer widths"
    )]
    fn create_minimal_wad_bytes() -> Vec<u8> {
        let mut wad = Vec::new();

        let data_start = 12u32;

        let e1m1_offset = data_start;
        let e1m1_size = 0i32;

        let vertex_data = [0i16, 0i16, 100i16, 100i16];
        let vertex_bytes: Vec<u8> = vertex_data.iter().flat_map(|v| v.to_le_bytes()).collect();
        let vertex_offset = data_start;
        let vertex_size = vertex_bytes.len() as i32;

        let linedef_data: [u8; 14] = [
            0, 0, 1, 0, // start_vertex=0, end_vertex=1
            0, 0, // flags
            0, 0, // line_type
            0, 0, // sector_tag
            0, 0, // right_sidedef
            0, 0, // left_sidedef
        ];
        let linedef_offset = data_start + vertex_size as u32;
        let linedef_size = 14i32;

        let mut sector_data = [0u8; 26];
        sector_data[0..2].copy_from_slice(&0i16.to_le_bytes());
        sector_data[2..4].copy_from_slice(&128i16.to_le_bytes());
        sector_data[4..12].copy_from_slice(b"FLOOR4_6");
        sector_data[12..20].copy_from_slice(b"CEIL3_5 ");
        sector_data[20..22].copy_from_slice(&160u16.to_le_bytes());
        let sector_offset = linedef_offset + linedef_size as u32;
        let sector_size = 26i32;

        let thing_data: [u8; 10] = [50, 0, 50, 0, 0, 0, 1, 0, 0, 0];
        let thing_offset = sector_offset + sector_size as u32;
        let thing_size = 10i32;

        wad.extend_from_slice(b"IWAD");
        let dir_offset = thing_offset + thing_size as u32;
        wad.extend_from_slice(&5i32.to_le_bytes());
        wad.extend_from_slice(&dir_offset.to_le_bytes());

        wad.extend_from_slice(&vertex_bytes);
        wad.extend_from_slice(&linedef_data);
        wad.extend_from_slice(&sector_data);
        wad.extend_from_slice(&thing_data);

        wad.extend_from_slice(&dir_entry(e1m1_offset, e1m1_size, "E1M1"));
        wad.extend_from_slice(&dir_entry(vertex_offset, vertex_size, "VERTEXES"));
        wad.extend_from_slice(&dir_entry(linedef_offset, linedef_size, "LINEDEFS"));
        wad.extend_from_slice(&dir_entry(sector_offset, sector_size, "SECTORS"));
        wad.extend_from_slice(&dir_entry(thing_offset, thing_size, "THINGS"));

        wad
    }

    fn write_wad(bytes: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.wad");
        std::fs::write(&path, bytes).unwrap();
        (dir, path)
    }

    #[test]
    fn test_wad_load_minimal() {
        let wad_bytes = create_minimal_wad_bytes();
        let (_dir, path) = write_wad(&wad_bytes);

        let wad = WadData::load(&path).expect("Should load minimal WAD");
        assert!(!wad.maps.is_empty(), "Should have at least one map");
        let map = &wad.maps[0];
        assert_eq!(map.name, "E1M1");
        assert_eq!(map.vertices.len(), 2);
        assert_eq!(map.linedefs.len(), 1);
        assert_eq!(map.sectors.len(), 1);
        assert_eq!(map.things.len(), 1);
    }

    #[test]
    fn test_wad_lump_accessors() {
        let wad_bytes = create_minimal_wad_bytes();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_lump.wad");
        std::fs::write(&path, &wad_bytes).unwrap();

        let wad = WadData::load(&path).unwrap();

        let vertex_lump = wad.get_lump("VERTEXES").expect("Should have VERTEXES");
        assert_eq!(vertex_lump.name(), "VERTEXES");
        assert_eq!(vertex_lump.data().len(), 8);

        assert!(wad.get_lump("NONEXISTENT").is_none());
    }

    #[test]
    fn test_wad_get_map_by_name() {
        let wad_bytes = create_minimal_wad_bytes();
        let (_dir, path) = write_wad(&wad_bytes);

        let wad = WadData::load(&path).unwrap();

        let map = wad.get_map("E1M1").expect("Should find E1M1");
        assert_eq!(map.name, "E1M1");

        assert!(wad.get_map("E2M1").is_none());
    }

    #[test]
    fn test_wad_first_map() {
        let wad_bytes = create_minimal_wad_bytes();
        let (_dir, path) = write_wad(&wad_bytes);

        let wad = WadData::load(&path).unwrap();

        let map = wad.first_map().expect("Should have first map");
        assert_eq!(map.name, "E1M1");
    }

    #[test]
    fn test_vertex_coordinates() {
        let wad_bytes = create_minimal_wad_bytes();
        let (_dir, path) = write_wad(&wad_bytes);

        let wad = WadData::load(&path).unwrap();

        let map = wad.first_map().unwrap();
        assert_eq!(map.vertices[0].x, 0);
        assert_eq!(map.vertices[0].y, 0);
        assert_eq!(map.vertices[1].x, 100);
        assert_eq!(map.vertices[1].y, 100);
    }

    #[test]
    fn test_wad_invalid_header_type() {
        let mut bad_wad = vec![0u8; 12];
        bad_wad[0..4].copy_from_slice(b"XXXX");
        bad_wad[4..8].copy_from_slice(&0i32.to_le_bytes());
        bad_wad[8..12].copy_from_slice(&12i32.to_le_bytes());
        let (_dir, path) = write_wad(&bad_wad);
        let result = WadData::load(&path);
        if let Err(e) = result {
            assert!(e.to_string().contains("Unknown WAD type"));
        } else {
            panic!("expected error");
        }
    }

    #[test]
    fn test_wad_truncated_header() {
        let bad_wad = vec![0u8; 8];
        let (_dir, path) = write_wad(&bad_wad);
        let result = WadData::load(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_wad_pwad_type() {
        let mut wad = create_minimal_wad_bytes();
        wad[0..4].copy_from_slice(b"PWAD");
        let (_dir, path) = write_wad(&wad);
        let result = WadData::load(&path);
        assert!(result.is_ok());
        assert!(!result.unwrap().maps.is_empty());
    }

    #[test]
    fn test_wad_no_maps() {
        let mut wad = Vec::new();
        wad.extend_from_slice(b"IWAD");
        wad.extend_from_slice(&1i32.to_le_bytes());
        wad.extend_from_slice(&12i32.to_le_bytes());
        let mut entry = [0u8; 16];
        entry[0..4].copy_from_slice(&12i32.to_le_bytes());
        entry[4..8].copy_from_slice(&0i32.to_le_bytes());
        entry[8..16].copy_from_slice(b"DATA    ");
        wad.extend_from_slice(&entry);
        let (_dir, path) = write_wad(&wad);
        let result = WadData::load(&path);
        if let Err(e) = result {
            assert!(e.to_string().contains("No valid maps"));
        } else {
            panic!("expected error");
        }
    }

    #[test]
    fn test_map_data_structures() {
        let wad_bytes = create_minimal_wad_bytes();
        let (_dir, path) = write_wad(&wad_bytes);
        let wad = WadData::load(&path).unwrap();
        let map = wad.first_map().unwrap();
        assert_eq!(map.linedefs[0].start_vertex, 0);
        assert_eq!(map.linedefs[0].end_vertex, 1);
        assert_eq!(map.linedefs[0].flags, 0);
        assert_eq!(map.linedefs[0].line_type, 0);
        assert_eq!(map.linedefs[0].sector_tag, 0);
        assert_eq!(map.sectors[0].floor_height, 0);
        assert_eq!(map.sectors[0].ceiling_height, 128);
        assert_eq!(map.sectors[0].floor_texture, "FLOOR4_6");
        assert_eq!(map.sectors[0].ceiling_texture, "CEIL3_5 ");
        assert_eq!(map.sectors[0].light_level, 160);
        assert_eq!(map.things[0].x, 50);
        assert_eq!(map.things[0].y, 50);
        assert_eq!(map.things[0].angle, 0);
        assert_eq!(map.things[0].thing_type, 1);
        assert_eq!(map.things[0].flags, 0);
    }

    #[test]
    fn test_lump_extraction() {
        let wad_bytes = create_minimal_wad_bytes();
        let (_dir, path) = write_wad(&wad_bytes);
        let wad = WadData::load(&path).unwrap();
        let lumps = wad.lumps();
        assert_eq!(lumps.len(), 5);
        assert_eq!(lumps[0].name(), "E1M1");
        assert_eq!(lumps[0].data().len(), 0);
        assert_eq!(lumps[1].name(), "VERTEXES");
        assert_eq!(lumps[1].data().len(), 8);
    }
}
