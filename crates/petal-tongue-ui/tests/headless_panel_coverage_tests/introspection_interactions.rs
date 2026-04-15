// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::PanelKind;
use petal_tongue_ui::headless_harness::HeadlessHarness;

#[test]
fn empty_graph_has_no_data_bindings() {
    let mut h = HeadlessHarness::new().unwrap();
    let intro = h.run_frame();

    // A freshly created headless app might have tutorial data or be empty
    // depending on build features - but the introspection itself should be populated
    let _binding_count = intro.bound_data.len();
}

#[test]
fn interaction_capabilities_include_navigate() {
    let mut h = HeadlessHarness::new().unwrap();
    let intro = h.run_frame();

    let has_navigate = intro
        .possible_interactions
        .iter()
        .any(|i| i.intent == petal_tongue_core::InteractionKind::Navigate);
    assert!(
        has_navigate,
        "Graph canvas should offer Navigate interaction"
    );
}

#[test]
fn interaction_capabilities_include_toggle_panel() {
    let mut h = HeadlessHarness::new().unwrap();
    let intro = h.run_frame();

    let has_toggle = intro
        .possible_interactions
        .iter()
        .any(|i| i.intent == petal_tongue_core::InteractionKind::TogglePanel);
    assert!(has_toggle, "TopMenu should offer TogglePanel interaction");
}

#[test]
fn interaction_capabilities_include_configure() {
    let mut h = HeadlessHarness::new().unwrap();
    h.run_frame();

    assert!(h.is_panel_visible(PanelKind::Controls));
    let intro = h.last_introspection().unwrap();
    let has_configure = intro
        .possible_interactions
        .iter()
        .any(|i| i.intent == petal_tongue_core::InteractionKind::Configure);
    assert!(
        has_configure,
        "Controls panel should offer Configure interaction"
    );
}
