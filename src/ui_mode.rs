//! UI mode - Desktop GUI
//! 
//! Platform dependencies: wayland-sys, x11-sys (acceptable for ecoBud)
//! This is the 1 mode (out of 5) that has platform dependencies

use anyhow::Result;

pub async fn run(scenario: Option<String>, no_audio: bool) -> Result<()> {
    tracing::info!(
        scenario = ?scenario,
        no_audio,
        "Starting desktop GUI mode"
    );
    
    // TODO: Integrate with existing petal-tongue-ui
    // For now, return error indicating work in progress
    anyhow::bail!("UI mode implementation in progress - use existing petal-tongue-ui binary for now")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ui_mode_signature() {
        // Test that the function signature is correct
        // Will implement full tests once integrated
        let result = run(None, false).await;
        assert!(result.is_err()); // Expected while unimplemented
    }
}

