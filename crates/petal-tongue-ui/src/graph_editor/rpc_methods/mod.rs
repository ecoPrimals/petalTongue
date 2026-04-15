// SPDX-License-Identifier: AGPL-3.0-or-later

mod edges;
mod graph;
mod nodes;
mod templates;
mod types;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::graph_editor::graph::Graph;

pub struct GraphEditorService {
    pub(super) graphs: Arc<RwLock<HashMap<String, Graph>>>,
    pub(super) templates: Arc<RwLock<HashMap<String, types::GraphTemplate>>>,
}

impl GraphEditorService {
    #[must_use]
    pub fn new() -> Self {
        Self {
            graphs: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for GraphEditorService {
    fn default() -> Self {
        Self::new()
    }
}

pub use types::{
    AddEdgeRequest, AddNodeRequest, ApplyTemplateRequest, EditorOpenRequest, GetPreviewRequest,
    GraphTemplate, ModifyNodeRequest, RemoveNodeRequest, SaveTemplateRequest,
};

#[cfg(test)]
mod tests;
