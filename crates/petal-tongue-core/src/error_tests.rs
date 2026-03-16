// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tests for error module

#[cfg(test)]
mod tests {
    use super::super::error::*;

    #[test]
    fn test_config_error() {
        let err = PetalTongueError::Config("test config error".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("configuration error"));
        assert!(msg.contains("test config error"));
    }

    #[test]
    fn test_graph_engine_error() {
        let err = PetalTongueError::GraphEngine("test graph error".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("graph engine error"));
        assert!(msg.contains("test graph error"));
    }

    #[test]
    fn test_renderer_error() {
        let err = PetalTongueError::Renderer("test render error".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("renderer error"));
        assert!(msg.contains("test render error"));
    }

    #[test]
    fn test_discovery_error() {
        let err = PetalTongueError::Discovery("test discovery error".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("primal discovery failed"));
        assert!(msg.contains("test discovery error"));
    }

    #[test]
    fn test_lock_poisoned_error() {
        let err = PetalTongueError::LockPoisoned("test lock error".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("lock poisoned"));
        assert!(msg.contains("test lock error"));
    }

    #[test]
    fn test_audio_error() {
        let err = PetalTongueError::Audio("test audio error".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("audio system error"));
        assert!(msg.contains("test audio error"));
    }

    #[test]
    fn test_telemetry_error() {
        let err = PetalTongueError::Telemetry("test telemetry error".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("telemetry error"));
        assert!(msg.contains("test telemetry error"));
    }

    #[test]
    fn test_internal_error() {
        let err = PetalTongueError::Internal("test internal error".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("internal error"));
        assert!(msg.contains("test internal error"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: PetalTongueError = io_err.into();
        let msg = format!("{err}");
        assert!(msg.contains("IO error"));
    }

    #[test]
    fn test_error_debug() {
        let err = PetalTongueError::Config("test".to_string());
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("Config"));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap(), &42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(PetalTongueError::Config("test".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_all_error_variants_display() {
        let errors = vec![
            PetalTongueError::Config("config".to_string()),
            PetalTongueError::GraphEngine("graph".to_string()),
            PetalTongueError::Renderer("render".to_string()),
            PetalTongueError::Discovery("discovery".to_string()),
            PetalTongueError::LockPoisoned("lock".to_string()),
            PetalTongueError::Audio("audio".to_string()),
            PetalTongueError::Telemetry("telemetry".to_string()),
            PetalTongueError::Internal("internal".to_string()),
        ];

        for err in errors {
            let msg = format!("{err}");
            assert!(!msg.is_empty());
        }
    }

    #[test]
    #[allow(clippy::significant_drop_tightening)]
    fn test_poison_error_conversion() {
        use std::sync::{Arc, RwLock};

        // Create a poisoned lock
        let lock = Arc::new(RwLock::new(0));
        let lock_clone = Arc::clone(&lock);

        // Poison the lock by panicking while holding it (guard must stay for poison)
        let _ = std::panic::catch_unwind(|| {
            let mut data = lock_clone.write().unwrap();
            *data = 1;
            panic!("intentional panic to poison lock");
        });

        // Try to acquire the poisoned lock
        #[allow(clippy::significant_drop_tightening)]
        let result = lock.write();
        assert!(result.is_err());
        let err: PetalTongueError = result.unwrap_err().into();
        let msg = format!("{err}");
        assert!(msg.contains("lock poisoned"));
    }
}
