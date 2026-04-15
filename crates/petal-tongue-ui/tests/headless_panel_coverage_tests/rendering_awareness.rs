// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::PanelKind;
use petal_tongue_ui::headless_harness::HeadlessHarness;

#[test]
fn rendering_awareness_visible_panels_match_introspection() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    let intro_panels = h.visible_panels();
    let awareness_arc = h.app().rendering_awareness();
    let awareness = awareness_arc.read().unwrap();
    let awareness_panels = awareness.visible_panels();

    assert_eq!(
        intro_panels.len(),
        awareness_panels.len(),
        "RenderingAwareness and FrameIntrospection should agree on panel count"
    );
}

#[test]
fn rendering_awareness_is_showing_data_matches() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    let awareness_arc = h.app().rendering_awareness();
    let awareness = awareness_arc.read().unwrap();

    // Non-existent data should not be "showing"
    assert!(!awareness.is_showing_data("totally-fake-id-xyz"));
}

#[test]
fn rendering_awareness_is_panel_visible_works() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    let awareness_arc = h.app().rendering_awareness();
    let awareness = awareness_arc.read().unwrap();

    assert!(awareness.is_panel_visible(PanelKind::TopMenu));
    assert!(!awareness.is_panel_visible(PanelKind::Proprioception));
}
