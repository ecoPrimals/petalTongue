// SPDX-License-Identifier: AGPL-3.0-or-later
//! WAD container and map geometry types.

/// A lump (data chunk) in the WAD.
#[derive(Debug, Clone)]
pub struct Lump {
    pub(crate) name: String,
    pub(crate) offset: i32,
    pub(crate) size: i32,
    pub(crate) data: Vec<u8>,
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
    pub(crate) lumps: Vec<Lump>,
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

    pub(crate) fn new(lumps: Vec<Lump>, maps: Vec<MapData>) -> Self {
        Self { lumps, maps }
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
