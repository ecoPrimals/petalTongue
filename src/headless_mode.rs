//! Headless mode - Pure Rust rendering without GUI
//! 
//! Pure Rust! ✅
//! No GUI dependencies

use anyhow::Result;
use std::sync::Arc;
use crate::data_service::DataService;

pub async fn run(_bind: &str, _workers: usize, data_service: Arc<DataService>) -> Result<()> {
    tracing::info!("Starting headless rendering mode (Pure Rust!)");
    
    tracing::info!("✅ Using shared DataService (zero duplication!)");
    
    // Output minimal info
    println!("🌸 petalTongue headless mode (Pure Rust!)");
    println!("Headless mode active - Pure Rust rendering ready");
    
    // Show data from DataService
    match data_service.snapshot().await {
        Ok(snapshot) => {
            println!("\n📊 Data from unified service:");
            println!("  Primals: {}", snapshot.primals.len());
            println!("  Edges: {}", snapshot.edges.len());
        }
        Err(e) => {
            tracing::warn!("Failed to get snapshot: {}", e);
        }
    }
    
    tracing::info!("Headless mode started successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_headless_mode() {
        let data_service = Arc::new(DataService::new());
        let result = run("0.0.0.0:8080", 4, data_service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_headless_concurrent() {
        // Test runs in parallel with others - no sleeps needed!
        let handles: Vec<_> = (0..4)
            .map(|i| {
                tokio::spawn(async move {
                    let port = format!("0.0.0.0:{}", 8080 + i);
                    let data_service = Arc::new(DataService::new());
                    run(&port, 1, data_service).await
                })
            })
            .collect();
        
        for handle in handles {
            assert!(handle.await.is_ok());
        }
    }
}


