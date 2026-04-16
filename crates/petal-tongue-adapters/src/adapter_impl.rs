// SPDX-License-Identifier: AGPL-3.0-or-later
//! Enum dispatch for [`PropertyAdapter`] (replaces `Box<dyn PropertyAdapter>`).

use crate::adapter_trait::{NodeDecoration, PropertyAdapter};
use crate::ecoprimal::{EcoPrimalCapabilityAdapter, EcoPrimalFamilyAdapter, EcoPrimalTrustAdapter};
use egui::Ui;
use petal_tongue_core::property::{Properties, PropertyValue};

/// Concrete adapter implementations for registry storage.
pub enum PropertyAdapterImpl {
    /// ecoPrimals family / lineage.
    Family(EcoPrimalFamilyAdapter),
    /// ecoPrimals trust levels.
    Trust(EcoPrimalTrustAdapter),
    /// ecoPrimals capability icons.
    Capability(EcoPrimalCapabilityAdapter),
    #[cfg(test)]
    TestNamed(test_support::TestNamedAdapter),
    #[cfg(test)]
    TestHighPriority(test_support::HighPriorityStub),
    #[cfg(test)]
    TestLowPriority(test_support::LowPriorityStub),
    #[cfg(test)]
    TestBadge(test_support::BadgeDecorationAdapter),
    #[cfg(test)]
    TestColor(test_support::ColorDecorationAdapter),
}

impl PropertyAdapter for PropertyAdapterImpl {
    fn name(&self) -> &str {
        match self {
            Self::Family(a) => a.name(),
            Self::Trust(a) => a.name(),
            Self::Capability(a) => a.name(),
            #[cfg(test)]
            Self::TestNamed(a) => a.name(),
            #[cfg(test)]
            Self::TestHighPriority(a) => a.name(),
            #[cfg(test)]
            Self::TestLowPriority(a) => a.name(),
            #[cfg(test)]
            Self::TestBadge(a) => a.name(),
            #[cfg(test)]
            Self::TestColor(a) => a.name(),
        }
    }

    fn handles(&self, property_key: &str) -> bool {
        match self {
            Self::Family(a) => a.handles(property_key),
            Self::Trust(a) => a.handles(property_key),
            Self::Capability(a) => a.handles(property_key),
            #[cfg(test)]
            Self::TestNamed(a) => a.handles(property_key),
            #[cfg(test)]
            Self::TestHighPriority(a) => a.handles(property_key),
            #[cfg(test)]
            Self::TestLowPriority(a) => a.handles(property_key),
            #[cfg(test)]
            Self::TestBadge(a) => a.handles(property_key),
            #[cfg(test)]
            Self::TestColor(a) => a.handles(property_key),
        }
    }

    fn render(&self, property_key: &str, value: &PropertyValue, ui: &mut Ui) {
        match self {
            Self::Family(a) => a.render(property_key, value, ui),
            Self::Trust(a) => a.render(property_key, value, ui),
            Self::Capability(a) => a.render(property_key, value, ui),
            #[cfg(test)]
            Self::TestNamed(a) => a.render(property_key, value, ui),
            #[cfg(test)]
            Self::TestHighPriority(a) => a.render(property_key, value, ui),
            #[cfg(test)]
            Self::TestLowPriority(a) => a.render(property_key, value, ui),
            #[cfg(test)]
            Self::TestBadge(a) => a.render(property_key, value, ui),
            #[cfg(test)]
            Self::TestColor(a) => a.render(property_key, value, ui),
        }
    }

    fn node_decoration(&self, properties: &Properties) -> Option<NodeDecoration> {
        match self {
            Self::Family(a) => a.node_decoration(properties),
            Self::Trust(a) => a.node_decoration(properties),
            Self::Capability(a) => a.node_decoration(properties),
            #[cfg(test)]
            Self::TestNamed(a) => a.node_decoration(properties),
            #[cfg(test)]
            Self::TestHighPriority(a) => a.node_decoration(properties),
            #[cfg(test)]
            Self::TestLowPriority(a) => a.node_decoration(properties),
            #[cfg(test)]
            Self::TestBadge(a) => a.node_decoration(properties),
            #[cfg(test)]
            Self::TestColor(a) => a.node_decoration(properties),
        }
    }

    fn priority(&self) -> i32 {
        match self {
            Self::Family(a) => a.priority(),
            Self::Trust(a) => a.priority(),
            Self::Capability(a) => a.priority(),
            #[cfg(test)]
            Self::TestNamed(a) => a.priority(),
            #[cfg(test)]
            Self::TestHighPriority(a) => a.priority(),
            #[cfg(test)]
            Self::TestLowPriority(a) => a.priority(),
            #[cfg(test)]
            Self::TestBadge(a) => a.priority(),
            #[cfg(test)]
            Self::TestColor(a) => a.priority(),
        }
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use egui::Color32;

    pub struct TestNamedAdapter {
        pub name: String,
    }

    impl PropertyAdapter for TestNamedAdapter {
        fn name(&self) -> &str {
            &self.name
        }

        fn handles(&self, property_key: &str) -> bool {
            property_key == "test"
        }

        fn render(&self, _key: &str, _value: &PropertyValue, _ui: &mut Ui) {}
    }

    pub struct HighPriorityStub;

    impl PropertyAdapter for HighPriorityStub {
        fn name(&self) -> &'static str {
            "high"
        }

        fn handles(&self, _: &str) -> bool {
            false
        }

        fn render(&self, _: &str, _: &PropertyValue, _: &mut Ui) {}

        fn priority(&self) -> i32 {
            100
        }
    }

    pub struct LowPriorityStub;

    impl PropertyAdapter for LowPriorityStub {
        fn name(&self) -> &'static str {
            "low"
        }

        fn handles(&self, _: &str) -> bool {
            false
        }

        fn render(&self, _: &str, _: &PropertyValue, _: &mut Ui) {}

        fn priority(&self) -> i32 {
            1
        }
    }

    pub struct BadgeDecorationAdapter;

    impl PropertyAdapter for BadgeDecorationAdapter {
        fn name(&self) -> &'static str {
            "badge"
        }

        fn handles(&self, _: &str) -> bool {
            false
        }

        fn render(&self, _: &str, _: &PropertyValue, _: &mut Ui) {}

        fn node_decoration(&self, properties: &Properties) -> Option<NodeDecoration> {
            if properties.contains_key("badge") {
                Some(NodeDecoration {
                    badge: Some("B".to_string()),
                    fill_color: None,
                    ring_color: None,
                    tooltip: None,
                })
            } else {
                None
            }
        }
    }

    pub struct ColorDecorationAdapter;

    impl PropertyAdapter for ColorDecorationAdapter {
        fn name(&self) -> &'static str {
            "color"
        }

        fn handles(&self, _: &str) -> bool {
            false
        }

        fn render(&self, _: &str, _: &PropertyValue, _: &mut Ui) {}

        fn node_decoration(&self, properties: &Properties) -> Option<NodeDecoration> {
            if properties.contains_key("color") {
                Some(NodeDecoration {
                    badge: None,
                    fill_color: Some(Color32::GREEN),
                    ring_color: None,
                    tooltip: None,
                })
            } else {
                None
            }
        }
    }
}
