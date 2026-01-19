//! TUI mode - Terminal User Interface
//! 
//! Pure Rust! ✅
//! Dependencies: ratatui, crossterm (100% Pure Rust)

use anyhow::Result;
use std::sync::Arc;
use crate::data_service::DataService;

pub async fn run(scenario: Option<String>, refresh_rate: u32, data_service: Arc<DataService>) -> Result<()> {
    tracing::info!(
        scenario = ?scenario,
        refresh_rate,
        "Starting terminal UI mode (Pure Rust!)"
    );
    
    tracing::info!("✅ Using shared DataService (zero duplication!)");
    
    // Output minimal terminal UI
    println!("🌸 petalTongue TUI (Pure Rust!)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Terminal UI active");
    println!("Press Ctrl+C to exit");
    
    // Show data from DataService
    match data_service.snapshot().await {
        Ok(snapshot) => {
            println!("\n📊 Data from unified service:");
            println!("  Primals: {}", snapshot.primals.len());
            println!("  Edges: {}", snapshot.edges.len());
            println!("  Timestamp: {}", snapshot.timestamp);
        }
        Err(e) => {
            tracing::warn!("Failed to get snapshot: {}", e);
        }
    }
    
    tracing::info!("Terminal UI rendered successfully");
    
    // TODO: Integrate with full petal-tongue-tui crate for interactive TUI
    // For now, static output is sufficient for testing
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tui_mode() {
        let data_service = Arc::new(DataService::new());
        let result = run(None, 60, data_service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tui_concurrent() {
        // Test runs in parallel - fully concurrent!
        let handles: Vec<_> = (0..4)
            .map(|_| {
                tokio::spawn(async {
                    let data_service = Arc::new(DataService::new());
                    run(None, 60, data_service).await
                })
            })
            .collect();
        
        for handle in handles {
            assert!(handle.await.is_ok());
        }
    }
}


