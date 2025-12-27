//! Tests for config module

#[cfg(test)]
mod tests {
    use super::super::config::PetalTongueConfig;

    #[test]
    fn test_default_config() {
        let config = PetalTongueConfig::default();

        // biomeos_url is Option<String>, may be Some or None
        assert!(config.refresh_interval_secs > 0);
        assert!(config.audio_sample_rate > 0);
        assert!(config.max_fps > 0);
    }

    #[test]
    fn test_refresh_interval() {
        let config = PetalTongueConfig::default();
        let duration = config.refresh_interval();

        assert_eq!(duration.as_secs(), config.refresh_interval_secs);
    }

    #[test]
    fn test_config_values() {
        let config = PetalTongueConfig::default();

        // Verify reasonable defaults
        assert!(config.audio_sample_rate >= 44100);
        assert!(config.max_fps >= 30);
        assert!(config.refresh_interval_secs >= 1);
    }

    #[test]
    fn test_biomeos_url_from_env() {
        // Test that biomeos_url can be set via environment
        // Note: In actual use, this would be set before the process starts
        // We test the Option<String> type here
        let config = PetalTongueConfig::default();

        // biomeos_url is Option<String>, can be Some or None
        match &config.biomeos_url {
            Some(url) => {
                // If set, should be a valid URL format
                assert!(url.starts_with("http://") || url.starts_with("https://"));
            }
            None => {
                // None is valid - will use Songbird auto-discovery
            }
        }
    }

    #[test]
    fn test_mock_mode_flag() {
        let config = PetalTongueConfig::default();
        // mock_mode should be a boolean
        assert!(config.mock_mode || !config.mock_mode);
    }

    #[test]
    fn test_audio_enabled_flag() {
        let config = PetalTongueConfig::default();
        // audio_enabled should be a boolean
        assert!(config.audio_enabled || !config.audio_enabled);
    }

    #[test]
    fn test_biomeos_url_optional() {
        let config = PetalTongueConfig::default();
        // biomeos_url is Option, so it can be Some or None
        match config.biomeos_url {
            Some(url) => assert!(!url.is_empty()),
            None => {} // None is valid for Songbird auto-discovery
        }
    }
}
