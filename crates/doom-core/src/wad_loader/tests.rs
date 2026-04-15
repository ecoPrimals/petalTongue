// SPDX-License-Identifier: AGPL-3.0-or-later

use super::WadData;

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
