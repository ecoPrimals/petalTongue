// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Headless integration tests: framebuffer dimensions and adaptive layouts.

use petal_tongue_ui::headless_harness::HeadlessHarness;

#[test]
fn viewport_small_640x480() {
    let mut harness = HeadlessHarness::with_screen_size(640.0, 480.0).unwrap();
    harness.run_frames(5);
    let (buffer, w, h) = harness.render_pixels().unwrap();
    assert_eq!(w, 640);
    assert_eq!(h, 480);
    assert_eq!(buffer.len(), (640 * 480 * 4) as usize);
}

#[test]
fn viewport_large_1920x1080() {
    let mut harness = HeadlessHarness::with_screen_size(1920.0, 1080.0).unwrap();
    harness.run_frames(3);
    let (buffer, w, h) = harness.render_pixels().unwrap();
    assert_eq!(w, 1920);
    assert_eq!(h, 1080);
    assert_eq!(buffer.len(), (1920 * 1080 * 4) as usize);
}

#[test]
fn viewport_adaptive_320x240() {
    let mut harness = HeadlessHarness::with_screen_size(320.0, 240.0).unwrap();
    harness.run_frames(5);
    let (buffer, w, h) = harness.render_pixels().unwrap();
    assert_eq!(w, 320);
    assert_eq!(h, 240);
    assert_eq!(buffer.len(), (320 * 240 * 4) as usize);
}

#[test]
fn viewport_adaptive_800x600() {
    let mut harness = HeadlessHarness::with_screen_size(800.0, 600.0).unwrap();
    harness.run_frames(5);
    assert!(!harness.visible_panels().is_empty());
    let _ = harness.tessellate();
}
