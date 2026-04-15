// SPDX-License-Identifier: AGPL-3.0-or-later
//! WAD file loading and parsing - Pure Rust implementation
//!
//! Phase 1.1: Load and parse Doom WAD files to extract map geometry.
//!
//! The WAD format is simple:
//! - Header (12 bytes): type, lump count, directory offset
//! - Directory: array of lump entries (name, offset, size)
//! - Data: raw lump data

mod endian;
mod io;
mod map_parse;
mod types;

#[cfg(test)]
mod tests;

use crate::error::DoomError;
use std::fs::File;
use std::path::Path;
use tracing::{debug, info};

pub use types::{LineDef, Lump, MapData, Sector, Thing, Vertex, WadData};

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

        let header = io::read_header(&mut file)?;
        info!(
            "WAD loaded successfully - Type: {:?}, Lumps: {}",
            header.wad_type, header.num_lumps
        );

        let lumps = io::read_directory(&mut file, &header)?;
        debug!("Read {} lump entries", lumps.len());

        let mut maps = Vec::new();

        for (i, lump) in lumps.iter().enumerate() {
            if map_parse::is_map_marker(&lump.name) {
                info!("Found map: {}", lump.name);

                match map_parse::parse_map(&lumps, i, &lump.name) {
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

        Ok(Self::new(lumps, maps))
    }
}
