//! ToadStool Bridge for Python Tool Integration
//!
//! Connects petalTongue to ToadStool (compute primal) for running Python tools.
//! Maintains primal sovereignty: petalTongue NEVER runs Python directly!

use crate::tool_integration::{ToolCapability, ToolMetadata, ToolPanel};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Request to execute a Python tool
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteRequest {
    pub tool_name: String,
    pub input: serde_json::Value,
}

/// Response from Python tool execution
#[derive(Debug, Clone, Deserialize)]
pub struct ExecuteResponse {
    pub status: String, // "success" | "error"
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Tool listing from ToadStool
#[derive(Debug, Clone, Deserialize)]
pub struct ToadStoolToolMetadata {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool version
    pub version: String,
    /// Tool capabilities (as strings)
    pub capabilities: Vec<String>,
    /// Tool icon (emoji)
    pub icon: String,
}

/// Bridge to ToadStool compute primal
///
/// This is NOT a tool itself - it's a tool PROVIDER that discovers
/// Python tools via ToadStool's capabilities and presents them as ToolPanels.
pub struct ToadStoolBridge {
    toadstool_endpoint: String,
    http_client: reqwest::Client,
    discovered_tools: Arc<RwLock<Vec<ToadStoolToolMetadata>>>,
}

impl ToadStoolBridge {
    /// Create a new ToadStool bridge
    ///
    /// Attempts to connect to ToadStool and discover available Python tools.
    pub async fn new(endpoint: String) -> Result<Self, String> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let bridge = Self {
            toadstool_endpoint: endpoint,
            http_client,
            discovered_tools: Arc::new(RwLock::new(Vec::new())),
        };

        // Try to discover tools (non-fatal if it fails)
        if let Err(e) = bridge.refresh_available_tools().await {
            tracing::warn!("Could not discover ToadStool tools: {}", e);
        }

        Ok(bridge)
    }

    /// Refresh the list of available Python tools from ToadStool
    pub async fn refresh_available_tools(&self) -> Result<(), String> {
        let url = format!("{}/api/tools/list", self.toadstool_endpoint);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to ToadStool: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "ToadStool returned error: {}",
                response.status()
            ));
        }

        let tools: Vec<ToadStoolToolMetadata> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse ToadStool response: {}", e))?;

        *self.discovered_tools.write().await = tools.clone();

        tracing::info!(
            "Discovered {} Python tools from ToadStool",
            tools.len()
        );

        Ok(())
    }

    /// Execute a Python tool via ToadStool
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<ExecuteResponse, String> {
        let url = format!("{}/api/tools/execute", self.toadstool_endpoint);

        let request = ExecuteRequest {
            tool_name: tool_name.to_string(),
            input,
        };

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to ToadStool: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "ToadStool returned error: {}",
                response.status()
            ));
        }

        let result: ExecuteResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse ToadStool response: {}", e))?;

        Ok(result)
    }

    /// Get discovered tools
    pub async fn discovered_tools(&self) -> Vec<ToadStoolToolMetadata> {
        self.discovered_tools.read().await.clone()
    }

    /// Get ToadStool endpoint
    pub fn endpoint(&self) -> &str {
        &self.toadstool_endpoint
    }
}

/// Wrapper that presents a Python tool as a ToolPanel
///
/// This allows Python tools to be used exactly like Rust tools in petalTongue.
pub struct PythonToolPanel {
    metadata: ToolMetadata,
    bridge: Arc<ToadStoolBridge>,
    show_panel: bool,
    
    // Tool state
    input_text: String,
    last_output: Option<serde_json::Value>,
    last_error: Option<String>,
    is_executing: bool,
}

impl PythonToolPanel {
    /// Create a new Python tool panel
    pub fn new(tool_meta: ToadStoolToolMetadata, bridge: Arc<ToadStoolBridge>) -> Self {
        // Convert ToadStool metadata to ToolMetadata
        let capabilities: Vec<ToolCapability> = tool_meta
            .capabilities
            .iter()
            .map(|cap| match cap.as_str() {
                "visual" => ToolCapability::Visual,
                "audio" => ToolCapability::Audio,
                "export" => ToolCapability::Export,
                other => ToolCapability::Custom(other.to_string()),
            })
            .collect();

        let metadata = ToolMetadata {
            name: format!("🐍 {}", tool_meta.name), // Prefix with Python icon
            description: format!("{} (Python via ToadStool)", tool_meta.description),
            version: tool_meta.version,
            capabilities,
            icon: tool_meta.icon,
            source: Some(format!("Python tool via ToadStool @ {}", bridge.endpoint())),
        };

        Self {
            metadata,
            bridge,
            show_panel: false,
            input_text: String::new(),
            last_output: None,
            last_error: None,
            is_executing: false,
        }
    }

    /// Render Python tool UI
    fn render_python_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading(format!("{} {}", self.metadata.icon, self.metadata.name));
        ui.label(&self.metadata.description);
        ui.add_space(10.0);

        // Input section
        ui.group(|ui| {
            ui.label("Input (JSON):");
            ui.text_edit_multiline(&mut self.input_text);

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui.button("▶ Execute").clicked() && !self.is_executing {
                    // Parse input and execute
                    self.execute_tool();
                }

                if self.is_executing {
                    ui.spinner();
                    ui.label("Executing...");
                }
            });
        });

        ui.add_space(10.0);

        // Output section
        if let Some(error) = &self.last_error {
            ui.colored_label(egui::Color32::from_rgb(200, 50, 50), "Error:");
            ui.label(error);
        }

        if let Some(output) = &self.last_output {
            ui.separator();
            ui.heading("Output:");
            ui.add_space(5.0);

            // Display output (handle different types)
            if let Some(plot_data) = output.get("plot_data").and_then(|v| v.as_str()) {
                // Base64-encoded image
                ui.label("Plot:");
                ui.label(format!("(Base64 image: {} chars)", plot_data.len()));
                // TODO: Decode and display image
            } else {
                // JSON output
                let json_str = serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string());
                ui.label(json_str);
            }
        }
    }

    /// Execute the Python tool (async simulation for now)
    fn execute_tool(&mut self) {
        self.is_executing = true;
        self.last_error = None;

        // Parse input JSON
        let input: serde_json::Value = match serde_json::from_str(&self.input_text) {
            Ok(v) => v,
            Err(e) => {
                self.last_error = Some(format!("Invalid JSON: {}", e));
                self.is_executing = false;
                return;
            }
        };

        // In a real implementation, this would be async
        // For now, we'll just show a placeholder
        self.last_error = Some("ToadStool execution not yet implemented in this demo".to_string());
        self.is_executing = false;

        // TODO: Implement actual async execution
        // let tool_name = self.metadata.name.clone();
        // let bridge = self.bridge.clone();
        // tokio::spawn(async move {
        //     let result = bridge.execute_tool(&tool_name, input).await;
        //     // Update UI with result
        // });
    }
}

impl ToolPanel for PythonToolPanel {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    fn is_visible(&self) -> bool {
        self.show_panel
    }

    fn toggle_visibility(&mut self) {
        self.show_panel = !self.show_panel;
    }

    fn render_panel(&mut self, ui: &mut egui::Ui) {
        // Header
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(egui::RichText::new("🐍 Python Tool (via ToadStool)").size(24.0));
            ui.label(
                egui::RichText::new("Python tools run on ToadStool compute primal")
                    .size(14.0)
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(10.0);
        });

        ui.separator();
        ui.add_space(10.0);

        // Main content
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(30, 30, 35))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    self.render_python_ui(ui);
                });
        });
    }

    fn status_message(&self) -> Option<String> {
        if self.is_executing {
            Some("Executing...".to_string())
        } else if self.last_error.is_some() {
            Some("Error".to_string())
        } else if self.last_output.is_some() {
            Some("Ready".to_string())
        } else {
            Some("Idle".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_request_serialization() {
        let req = ExecuteRequest {
            tool_name: "test".to_string(),
            input: serde_json::json!({"x": [1, 2, 3]}),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("input"));
    }

    #[test]
    fn test_execute_response_deserialization() {
        let json = r#"{"status":"success","output":{"result":42},"error":null}"#;
        let resp: ExecuteResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "success");
        assert!(resp.output.is_some());
    }
}

