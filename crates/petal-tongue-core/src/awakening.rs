//! # Awakening Experience
//! 
//! The default touchpoint: flower opening to sunrise, leading to tutorial.

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::engine::UniversalRenderingEngine;

/// Awakening Stage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwakeningStage {
    /// Stage 1: Awakening (0-3s)
    Awakening,
    
    /// Stage 2: Self-Knowledge (3-6s)
    SelfKnowledge,
    
    /// Stage 3: Discovery (6-10s)
    Discovery,
    
    /// Stage 4: Tutorial Invitation (10-12s)
    Tutorial,
    
    /// Complete
    Complete,
}

/// Awakening Experience
/// 
/// Coordinates the multi-modal awakening sequence.
pub struct AwakeningExperience {
    /// Reference to engine
    engine: Arc<UniversalRenderingEngine>,
    
    /// Current stage
    stage: Arc<RwLock<AwakeningStage>>,
    
    /// Configuration
    config: AwakeningConfig,
}

/// Awakening Configuration
#[derive(Debug, Clone)]
pub struct AwakeningConfig {
    /// Enable awakening experience
    pub enabled: bool,
    
    /// Stage 1 duration (seconds)
    pub stage_1_duration: u64,
    
    /// Stage 2 duration (seconds)
    pub stage_2_duration: u64,
    
    /// Stage 3 duration (seconds)
    pub stage_3_duration: u64,
    
    /// Stage 4 duration (seconds)
    pub stage_4_duration: u64,
    
    /// Auto-start tutorial after awakening
    pub auto_tutorial: bool,
    
    /// Enable visual animation
    pub visual_enabled: bool,
    
    /// Enable audio
    pub audio_enabled: bool,
    
    /// Enable text descriptions
    pub text_enabled: bool,
}

impl Default for AwakeningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            stage_1_duration: 3,
            stage_2_duration: 3,
            stage_3_duration: 4,
            stage_4_duration: 2,
            auto_tutorial: true,
            visual_enabled: true,
            audio_enabled: true,
            text_enabled: true,
        }
    }
}

impl AwakeningExperience {
    /// Create new awakening experience
    pub fn new(engine: Arc<UniversalRenderingEngine>) -> Self {
        Self {
            engine,
            stage: Arc::new(RwLock::new(AwakeningStage::Awakening)),
            config: AwakeningConfig::default(),
        }
    }
    
    /// Create with custom config
    pub fn with_config(engine: Arc<UniversalRenderingEngine>, config: AwakeningConfig) -> Self {
        Self {
            engine,
            stage: Arc::new(RwLock::new(AwakeningStage::Awakening)),
            config,
        }
    }
    
    /// Get current stage
    pub async fn current_stage(&self) -> AwakeningStage {
        *self.stage.read().await
    }
    
    /// Run the complete awakening experience
    pub async fn run(&self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("Awakening experience disabled, skipping");
            return Ok(());
        }
        
        tracing::info!("🌸 Starting awakening experience...");
        
        // Stage 1: Awakening
        self.run_stage_1().await?;
        
        // Stage 2: Self-Knowledge
        self.run_stage_2().await?;
        
        // Stage 3: Discovery
        self.run_stage_3().await?;
        
        // Stage 4: Tutorial
        self.run_stage_4().await?;
        
        // Mark complete
        {
            let mut stage = self.stage.write().await;
            *stage = AwakeningStage::Complete;
        }
        
        tracing::info!("✅ Awakening experience complete");
        
        Ok(())
    }
    
    /// Stage 1: Awakening (0-3s)
    async fn run_stage_1(&self) -> Result<()> {
        {
            let mut stage = self.stage.write().await;
            *stage = AwakeningStage::Awakening;
        }
        
        tracing::info!("🌸 Stage 1: Awakening...");
        
        // TODO: Coordinate modalities
        // - Visual: Flower opening animation
        // - Audio: Startup tones + embedded music
        // - Text: "Awakening..."
        
        tokio::time::sleep(Duration::from_secs(self.config.stage_1_duration)).await;
        
        Ok(())
    }
    
    /// Stage 2: Self-Knowledge (3-6s)
    async fn run_stage_2(&self) -> Result<()> {
        {
            let mut stage = self.stage.write().await;
            *stage = AwakeningStage::SelfKnowledge;
        }
        
        tracing::info!("🌸 Stage 2: Self-Knowledge...");
        
        // TODO: Coordinate modalities
        // - Visual: Flower fully open, glowing
        // - Audio: Heartbeat harmonics
        // - Text: "I am petalTongue. I know myself."
        
        tokio::time::sleep(Duration::from_secs(self.config.stage_2_duration)).await;
        
        Ok(())
    }
    
    /// Stage 3: Discovery (6-10s)
    async fn run_stage_3(&self) -> Result<()> {
        {
            let mut stage = self.stage.write().await;
            *stage = AwakeningStage::Discovery;
        }
        
        tracing::info!("🌸 Stage 3: Discovery...");
        
        // TODO: Coordinate modalities
        // - Visual: Tendrils reaching out
        // - Audio: Discovery chimes
        // - Text: "Found: Songbird, Toadstool..."
        
        // TODO: Actually discover other primals
        self.engine.discover_compute().await?;
        
        tokio::time::sleep(Duration::from_secs(self.config.stage_3_duration)).await;
        
        Ok(())
    }
    
    /// Stage 4: Tutorial Invitation (10-12s)
    async fn run_stage_4(&self) -> Result<()> {
        {
            let mut stage = self.stage.write().await;
            *stage = AwakeningStage::Tutorial;
        }
        
        tracing::info!("🌸 Stage 4: Tutorial Invitation...");
        
        // TODO: Coordinate modalities
        // - Visual: Tutorial invitation panel
        // - Audio: Completion harmony
        // - Text: "Ready. Let me show you."
        
        tokio::time::sleep(Duration::from_secs(self.config.stage_4_duration)).await;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_awakening_stages() {
        let engine = Arc::new(UniversalRenderingEngine::new().unwrap());
        let awakening = AwakeningExperience::new(engine);
        
        assert_eq!(awakening.current_stage().await, AwakeningStage::Awakening);
    }
    
    #[tokio::test]
    async fn test_awakening_disabled() {
        let engine = Arc::new(UniversalRenderingEngine::new().unwrap());
        let mut config = AwakeningConfig::default();
        config.enabled = false;
        
        let awakening = AwakeningExperience::with_config(engine, config);
        
        let result = awakening.run().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_custom_durations() {
        let engine = Arc::new(UniversalRenderingEngine::new().unwrap());
        let mut config = AwakeningConfig::default();
        config.stage_1_duration = 1; // Fast for testing
        config.stage_2_duration = 1;
        config.stage_3_duration = 1;
        config.stage_4_duration = 1;
        
        let awakening = AwakeningExperience::with_config(engine, config);
        
        let start = std::time::Instant::now();
        awakening.run().await.unwrap();
        let elapsed = start.elapsed();
        
        // Should complete in ~4 seconds
        assert!(elapsed < Duration::from_secs(5));
        assert_eq!(awakening.current_stage().await, AwakeningStage::Complete);
    }
}

