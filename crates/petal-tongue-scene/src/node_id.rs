// SPDX-License-Identifier: AGPL-3.0-only
//! Stable node identifier for the scene graph.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;
use std::sync::Arc;

/// Stable identifier for a scene node.
///
/// Uses `Arc<str>` for zero-copy sharing: the same ID can appear as a map key,
/// in node.children, and in parent references without cloning the string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(Arc<str>);

impl Serialize for NodeId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for NodeId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        Ok(Self(Arc::from(s.into_boxed_str())))
    }
}

impl NodeId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(Arc::from(s.into_boxed_str()))
    }
}

impl From<&String> for NodeId {
    fn from(s: &String) -> Self {
        Self(Arc::from(s.as_str()))
    }
}

impl Borrow<str> for NodeId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for NodeId {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref() == other
    }
}

impl PartialEq<&str> for NodeId {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref() == *other
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
