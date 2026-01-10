//! Capability taxonomy for ecoPrimals ecosystem
//!
//! Defines the standard capability taxonomy that enables capability-based
//! primal discovery and routing following the biomeOS convention.
//!
//! # TRUE PRIMAL Principles
//!
//! - **Capability-Based**: Primals are discovered by capability, not by name
//! - **Runtime Detection**: Capabilities are detected at runtime, not hardcoded
//! - **Agnostic Design**: No assumptions about which primals provide which capabilities
//!
//! # biomeOS Integration
//!
//! This taxonomy aligns with biomeOS capability naming conventions, enabling
//! zero-configuration discovery and routing across the ecosystem.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Capability taxonomy following biomeOS convention
///
/// Capabilities use dot-notation for hierarchical organization:
/// `<domain>.<category>.<specific>`
///
/// # Examples
///
/// ```
/// use petal_tongue_core::capability_taxonomy::CapabilityTaxonomy;
///
/// let cap = CapabilityTaxonomy::UIRender;
/// assert_eq!(cap.as_str(), "ui.render");
///
/// let parsed: CapabilityTaxonomy = "ui.graph".parse().unwrap();
/// assert_eq!(parsed, CapabilityTaxonomy::UIGraph);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityTaxonomy {
    // ===== UI Capabilities =====
    
    /// General UI rendering capability
    ///
    /// Can render various content types (graphs, text, images, etc.)
    #[serde(rename = "ui.render")]
    UIRender,
    
    /// Data visualization capability
    ///
    /// Can create visual representations of data
    #[serde(rename = "ui.visualization")]
    UIVisualization,
    
    /// Graph/network visualization
    ///
    /// Can render node-edge graphs and network topologies
    #[serde(rename = "ui.graph")]
    UIGraph,
    
    /// Terminal/console UI
    ///
    /// Can render to text-based terminal interfaces
    #[serde(rename = "ui.terminal")]
    UITerminal,
    
    /// Audio/sound capability
    ///
    /// Can generate or play audio output
    #[serde(rename = "ui.audio")]
    UIAudio,
    
    /// Framebuffer rendering
    ///
    /// Can render directly to Linux framebuffer (/dev/fb0)
    #[serde(rename = "ui.framebuffer")]
    UIFramebuffer,
    
    // ===== Input Capabilities =====
    
    /// Keyboard input capability
    ///
    /// Can receive and process keyboard input
    #[serde(rename = "ui.input.keyboard")]
    UIInputKeyboard,
    
    /// Mouse/pointer input capability
    ///
    /// Can receive and process mouse/pointer input
    #[serde(rename = "ui.input.mouse")]
    UIInputMouse,
    
    /// Touch input capability
    ///
    /// Can receive and process touch screen input
    #[serde(rename = "ui.input.touch")]
    UIInputTouch,
    
    // ===== Discovery Capabilities =====
    
    /// mDNS-based discovery
    ///
    /// Can discover services via mDNS/Bonjour
    #[serde(rename = "discovery.mdns")]
    DiscoveryMDNS,
    
    /// HTTP-based discovery
    ///
    /// Can discover services via HTTP endpoints
    #[serde(rename = "discovery.http")]
    DiscoveryHTTP,
    
    // ===== Storage Capabilities =====
    
    /// Persistent data storage
    ///
    /// Can store and retrieve persistent data
    #[serde(rename = "storage.persistent")]
    StoragePersistent,
    
    /// Caching capability
    ///
    /// Can cache frequently accessed data
    #[serde(rename = "storage.cache")]
    StorageCache,
    
    // ===== Communication Capabilities =====
    
    /// tarpc RPC protocol support
    ///
    /// Can communicate via high-performance tarpc binary protocol
    #[serde(rename = "ipc.tarpc")]
    IpcTarpc,
    
    /// JSON-RPC protocol support
    ///
    /// Can communicate via JSON-RPC 2.0 protocol
    #[serde(rename = "ipc.json-rpc")]
    IpcJsonRpc,
    
    /// Unix socket support
    ///
    /// Can communicate via Unix domain sockets
    #[serde(rename = "ipc.unix-socket")]
    IpcUnixSocket,
}

impl CapabilityTaxonomy {
    /// Convert capability to string representation
    ///
    /// Returns the dot-notation string (e.g., "ui.render")
    pub fn as_str(&self) -> &'static str {
        match self {
            // UI Capabilities
            Self::UIRender => "ui.render",
            Self::UIVisualization => "ui.visualization",
            Self::UIGraph => "ui.graph",
            Self::UITerminal => "ui.terminal",
            Self::UIAudio => "ui.audio",
            Self::UIFramebuffer => "ui.framebuffer",
            
            // Input Capabilities
            Self::UIInputKeyboard => "ui.input.keyboard",
            Self::UIInputMouse => "ui.input.mouse",
            Self::UIInputTouch => "ui.input.touch",
            
            // Discovery Capabilities
            Self::DiscoveryMDNS => "discovery.mdns",
            Self::DiscoveryHTTP => "discovery.http",
            
            // Storage Capabilities
            Self::StoragePersistent => "storage.persistent",
            Self::StorageCache => "storage.cache",
            
            // Communication Capabilities
            Self::IpcTarpc => "ipc.tarpc",
            Self::IpcJsonRpc => "ipc.json-rpc",
            Self::IpcUnixSocket => "ipc.unix-socket",
        }
    }
    
    /// Get all UI-related capabilities
    ///
    /// Returns capabilities in the "ui.*" domain
    pub fn ui_capabilities() -> Vec<Self> {
        vec![
            Self::UIRender,
            Self::UIVisualization,
            Self::UIGraph,
            Self::UITerminal,
            Self::UIAudio,
            Self::UIFramebuffer,
        ]
    }
    
    /// Get all input-related capabilities
    ///
    /// Returns capabilities in the "ui.input.*" domain
    pub fn input_capabilities() -> Vec<Self> {
        vec![
            Self::UIInputKeyboard,
            Self::UIInputMouse,
            Self::UIInputTouch,
        ]
    }
    
    /// Get all discovery-related capabilities
    ///
    /// Returns capabilities in the "discovery.*" domain
    pub fn discovery_capabilities() -> Vec<Self> {
        vec![
            Self::DiscoveryMDNS,
            Self::DiscoveryHTTP,
        ]
    }
    
    /// Get all storage-related capabilities
    ///
    /// Returns capabilities in the "storage.*" domain
    pub fn storage_capabilities() -> Vec<Self> {
        vec![
            Self::StoragePersistent,
            Self::StorageCache,
        ]
    }
    
    /// Get all IPC-related capabilities
    ///
    /// Returns capabilities in the "ipc.*" domain
    pub fn ipc_capabilities() -> Vec<Self> {
        vec![
            Self::IpcTarpc,
            Self::IpcJsonRpc,
            Self::IpcUnixSocket,
        ]
    }
    
    /// Check if this is a UI capability
    pub fn is_ui(&self) -> bool {
        matches!(
            self,
            Self::UIRender
                | Self::UIVisualization
                | Self::UIGraph
                | Self::UITerminal
                | Self::UIAudio
                | Self::UIFramebuffer
        )
    }
    
    /// Check if this is an input capability
    pub fn is_input(&self) -> bool {
        matches!(
            self,
            Self::UIInputKeyboard | Self::UIInputMouse | Self::UIInputTouch
        )
    }
}

impl fmt::Display for CapabilityTaxonomy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CapabilityTaxonomy {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // UI Capabilities
            "ui.render" => Ok(Self::UIRender),
            "ui.visualization" => Ok(Self::UIVisualization),
            "ui.graph" => Ok(Self::UIGraph),
            "ui.terminal" => Ok(Self::UITerminal),
            "ui.audio" => Ok(Self::UIAudio),
            "ui.framebuffer" => Ok(Self::UIFramebuffer),
            
            // Input Capabilities
            "ui.input.keyboard" => Ok(Self::UIInputKeyboard),
            "ui.input.mouse" => Ok(Self::UIInputMouse),
            "ui.input.touch" => Ok(Self::UIInputTouch),
            
            // Discovery Capabilities
            "discovery.mdns" => Ok(Self::DiscoveryMDNS),
            "discovery.http" => Ok(Self::DiscoveryHTTP),
            
            // Storage Capabilities
            "storage.persistent" => Ok(Self::StoragePersistent),
            "storage.cache" => Ok(Self::StorageCache),
            
            // Communication Capabilities
            "ipc.tarpc" => Ok(Self::IpcTarpc),
            "ipc.json-rpc" => Ok(Self::IpcJsonRpc),
            "ipc.unix-socket" => Ok(Self::IpcUnixSocket),
            
            _ => Err(format!("Unknown capability: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_as_str_conversion() {
        assert_eq!(CapabilityTaxonomy::UIRender.as_str(), "ui.render");
        assert_eq!(CapabilityTaxonomy::UIGraph.as_str(), "ui.graph");
        assert_eq!(CapabilityTaxonomy::UITerminal.as_str(), "ui.terminal");
        assert_eq!(CapabilityTaxonomy::UIAudio.as_str(), "ui.audio");
    }
    
    #[test]
    fn test_from_str_parsing() {
        let cap: CapabilityTaxonomy = "ui.render".parse().unwrap();
        assert_eq!(cap, CapabilityTaxonomy::UIRender);
        
        let cap: CapabilityTaxonomy = "ui.graph".parse().unwrap();
        assert_eq!(cap, CapabilityTaxonomy::UIGraph);
        
        let cap: CapabilityTaxonomy = "ui.terminal".parse().unwrap();
        assert_eq!(cap, CapabilityTaxonomy::UITerminal);
    }
    
    #[test]
    fn test_from_str_invalid() {
        let result: Result<CapabilityTaxonomy, _> = "invalid.capability".parse();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_display_formatting() {
        let cap = CapabilityTaxonomy::UIRender;
        assert_eq!(format!("{}", cap), "ui.render");
    }
    
    #[test]
    fn test_round_trip() {
        let original = CapabilityTaxonomy::UIVisualization;
        let string = original.as_str();
        let parsed: CapabilityTaxonomy = string.parse().unwrap();
        assert_eq!(original, parsed);
    }
    
    #[test]
    fn test_ui_capabilities() {
        let caps = CapabilityTaxonomy::ui_capabilities();
        assert!(caps.len() >= 6);
        assert!(caps.contains(&CapabilityTaxonomy::UIRender));
        assert!(caps.contains(&CapabilityTaxonomy::UIGraph));
    }
    
    #[test]
    fn test_is_ui() {
        assert!(CapabilityTaxonomy::UIRender.is_ui());
        assert!(CapabilityTaxonomy::UIGraph.is_ui());
        assert!(!CapabilityTaxonomy::DiscoveryMDNS.is_ui());
        assert!(!CapabilityTaxonomy::IpcTarpc.is_ui());
    }
    
    #[test]
    fn test_is_input() {
        assert!(CapabilityTaxonomy::UIInputKeyboard.is_input());
        assert!(CapabilityTaxonomy::UIInputMouse.is_input());
        assert!(!CapabilityTaxonomy::UIRender.is_input());
    }
    
    #[test]
    fn test_serde_serialization() {
        let cap = CapabilityTaxonomy::UIRender;
        let json = serde_json::to_string(&cap).unwrap();
        assert!(json.contains("ui.render"));
    }
    
    #[test]
    fn test_serde_deserialization() {
        let json = r#""ui.render""#;
        let cap: CapabilityTaxonomy = serde_json::from_str(json).unwrap();
        assert_eq!(cap, CapabilityTaxonomy::UIRender);
    }
}

