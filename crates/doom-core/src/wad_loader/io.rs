// SPDX-License-Identifier: AGPL-3.0-or-later
//! WAD header and directory I/O.

use crate::error::DoomError;
use crate::wad_loader::endian::i32_le_at;
use crate::wad_loader::types::Lump;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// Idiomatic Rust: Acronyms in type names use lowercase except initial letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WadType {
    Iwad,
    Pwad,
}

pub(super) struct WadHeader {
    pub(super) wad_type: WadType,
    pub(super) num_lumps: i32,
    pub(super) dir_offset: i32,
}

pub(super) fn read_header(file: &mut File) -> Result<WadHeader, DoomError> {
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

    let num_lumps = i32_le_at(&header_bytes, 4);
    let dir_offset = i32_le_at(&header_bytes, 8);

    Ok(WadHeader {
        wad_type,
        num_lumps,
        dir_offset,
    })
}

struct DirEntry {
    name: String,
    offset: i32,
    size: i32,
}

fn read_directory_entries(file: &mut File, num_lumps: i32) -> Result<Vec<DirEntry>, DoomError> {
    let mut entries = Vec::new();

    for _ in 0..num_lumps {
        let mut entry_bytes = [0u8; 16];
        file.read_exact(&mut entry_bytes)
            .map_err(|e| DoomError::InvalidWad(format!("Failed to read directory entry: {e}")))?;

        let offset = i32_le_at(&entry_bytes, 0);
        let size = i32_le_at(&entry_bytes, 4);

        let name_end = entry_bytes[8..16].iter().position(|&b| b == 0).unwrap_or(8);
        let name = String::from_utf8_lossy(&entry_bytes[8..8 + name_end]).to_string();

        entries.push(DirEntry { name, offset, size });
    }

    Ok(entries)
}

fn read_lump_payloads(file: &mut File, lumps: &mut [Lump]) -> Result<(), DoomError> {
    for lump in lumps {
        if lump.size > 0 {
            let lump_pos = u64::try_from(lump.offset).map_err(|_| {
                DoomError::InvalidWad(format!(
                    "lump offset for {} must be non-negative",
                    lump.name
                ))
            })?;
            file.seek(SeekFrom::Start(lump_pos))
                .map_err(|e| DoomError::InvalidWad(format!("Failed to seek to lump data: {e}")))?;

            let lump_len = usize::try_from(lump.size).map_err(|_| {
                DoomError::InvalidWad(format!("lump size for {} must be non-negative", lump.name))
            })?;
            lump.data.resize(lump_len, 0);
            file.read_exact(&mut lump.data)
                .map_err(|e| DoomError::InvalidWad(format!("Failed to read lump data: {e}")))?;
        }
    }
    Ok(())
}

pub(super) fn read_directory(file: &mut File, header: &WadHeader) -> Result<Vec<Lump>, DoomError> {
    let dir_pos = u64::try_from(header.dir_offset)
        .map_err(|_| DoomError::InvalidWad("directory offset must be non-negative".into()))?;
    file.seek(SeekFrom::Start(dir_pos))
        .map_err(|e| DoomError::InvalidWad(format!("Failed to seek to directory: {e}")))?;

    let entries = read_directory_entries(file, header.num_lumps)?;
    let mut lumps: Vec<Lump> = entries
        .into_iter()
        .map(|e| Lump {
            name: e.name,
            offset: e.offset,
            size: e.size,
            data: Vec::new(),
        })
        .collect();

    read_lump_payloads(file, &mut lumps)?;
    Ok(lumps)
}
