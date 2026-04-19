// SPDX-License-Identifier: AGPL-3.0-or-later
//! Map marker detection and map lump parsing (vertices, linedefs, sectors, things).

use crate::error::DoomError;
use crate::wad_loader::endian::{i16_le, u16_le};
use crate::wad_loader::types::{LineDef, Lump, MapData, Sector, Thing, Vertex};

pub(super) fn is_map_marker(name: &str) -> bool {
    if name.len() == 4 && name.starts_with('E') && name.chars().nth(2) == Some('M') {
        return true;
    }
    name.len() == 5 && name.starts_with("MAP")
}

pub(super) fn parse_map(
    lumps: &[Lump],
    map_index: usize,
    map_name: &str,
) -> Result<MapData, DoomError> {
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
        vertices.push(Vertex {
            x: i16_le(&chunk[0..2]),
            y: i16_le(&chunk[2..4]),
        });
    }

    Ok(vertices)
}

fn parse_linedefs(lumps: &[Lump], map_index: usize) -> Result<Vec<LineDef>, DoomError> {
    let lump = find_map_lump(lumps, map_index, "LINEDEFS")?;
    let data = &lump.data;

    let mut linedefs = Vec::new();
    for chunk in data.chunks_exact(14) {
        linedefs.push(LineDef {
            start_vertex: u16_le(&chunk[0..2]) as usize,
            end_vertex: u16_le(&chunk[2..4]) as usize,
            flags: u16_le(&chunk[4..6]),
            line_type: u16_le(&chunk[6..8]),
            sector_tag: u16_le(&chunk[8..10]),
        });
    }

    Ok(linedefs)
}

fn parse_sectors(lumps: &[Lump], map_index: usize) -> Result<Vec<Sector>, DoomError> {
    let lump = find_map_lump(lumps, map_index, "SECTORS")?;
    let data = &lump.data;

    let mut sectors = Vec::new();
    for chunk in data.chunks_exact(26) {
        sectors.push(Sector {
            floor_height: i16_le(&chunk[0..2]),
            ceiling_height: i16_le(&chunk[2..4]),
            floor_texture: parse_texture_name(&chunk[4..12]),
            ceiling_texture: parse_texture_name(&chunk[12..20]),
            light_level: u16_le(&chunk[20..22]),
        });
    }

    Ok(sectors)
}

fn parse_things(lumps: &[Lump], map_index: usize) -> Result<Vec<Thing>, DoomError> {
    let lump = find_map_lump(lumps, map_index, "THINGS")?;
    let data = &lump.data;

    let mut things = Vec::new();
    for chunk in data.chunks_exact(10) {
        things.push(Thing {
            x: i16_le(&chunk[0..2]),
            y: i16_le(&chunk[2..4]),
            angle: u16_le(&chunk[4..6]),
            thing_type: u16_le(&chunk[6..8]),
            flags: u16_le(&chunk[8..10]),
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
    use super::is_map_marker;

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
