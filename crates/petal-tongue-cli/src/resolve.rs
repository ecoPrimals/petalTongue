// SPDX-License-Identifier: AGPL-3.0-or-later
//! Instance ID resolution (UUID and prefix matching).

use petal_tongue_core::{InstanceId, InstanceRegistry};

use crate::error::CliError;

/// Resolve instance ID from string (supports prefixes)
pub fn resolve_instance_id(id_str: &str) -> Result<InstanceId, CliError> {
    // Try to parse as UUID string and create InstanceId
    if let Ok(uuid) = uuid::Uuid::parse_str(id_str) {
        let id_string = uuid.to_string();
        return InstanceId::parse(&id_string)
            .map_err(|e| CliError::InvalidInstanceId(e.to_string()));
    }

    // Try prefix match
    let registry = InstanceRegistry::load()?;
    let instances = registry.list();

    let matches: Vec<_> = instances
        .iter()
        .filter(|i| i.id.as_str().starts_with(id_str))
        .collect();

    match matches.len() {
        0 => Err(CliError::NoInstanceFound(id_str.to_string())),
        1 => Ok(matches[0].id.clone()),
        _ => Err(CliError::AmbiguousInstanceId(
            id_str.to_string(),
            matches.len(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::Instance;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn temp_dir_path(temp_dir: &tempfile::TempDir) -> &str {
        temp_dir
            .path()
            .to_str()
            .expect("tempdir path should be valid UTF-8")
    }

    #[test]
    fn test_resolve_instance_id_valid_uuid() -> TestResult {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let resolved = resolve_instance_id(uuid_str)?;
        assert_eq!(resolved.as_str(), uuid_str);
        Ok(())
    }

    #[test]
    fn test_resolve_instance_id_invalid_uuid() {
        let result = resolve_instance_id("not-a-valid-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_instance_id_prefix_match() -> TestResult {
        let temp_dir = tempfile::TempDir::new()?;
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "XDG_DATA_HOME",
            temp_dir_path(&temp_dir),
            || -> TestResult {
                let mut registry = InstanceRegistry::new();
                let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440000")?;
                let instance = Instance::new(id, Some("test".to_owned()))?;
                registry.register(instance)?;

                let resolved = resolve_instance_id("550e")?;
                assert_eq!(
                    resolved.as_str(),
                    "550e8400-e29b-41d4-a716-446655440000"
                );
                Ok(())
            },
        )?;
        Ok(())
    }

    #[test]
    fn test_resolve_instance_id_prefix_ambiguous() -> TestResult {
        let temp_dir = tempfile::TempDir::new()?;
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "XDG_DATA_HOME",
            temp_dir_path(&temp_dir),
            || -> TestResult {
                let mut registry = InstanceRegistry::new();
                let id1 = InstanceId::parse("550e8400-e29b-41d4-a716-446655440001")?;
                let id2 = InstanceId::parse("550e8400-e29b-41d4-a716-446655440002")?;
                registry.register(Instance::new(id1, Some("a".to_owned()))?)?;
                registry.register(Instance::new(id2, Some("b".to_owned()))?)?;

                let result = resolve_instance_id("550e");
                assert!(result.is_err());
                let err = result.expect_err("expected ambiguous instance ID error");
                assert!(err.to_string().contains("Ambiguous"));
                assert!(err.to_string().contains('2'));
                Ok(())
            },
        )?;
        Ok(())
    }

    #[test]
    fn test_resolve_instance_id_prefix_no_match() -> TestResult {
        let temp_dir = tempfile::TempDir::new()?;
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "XDG_DATA_HOME",
            temp_dir_path(&temp_dir),
            || -> TestResult {
                let mut registry = InstanceRegistry::new();
                let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440000")?;
                registry.register(Instance::new(id, Some("test".to_owned()))?)?;

                let result = resolve_instance_id("ffff");
                assert!(result.is_err());
                assert!(
                    result
                        .expect_err("expected no instance found error")
                        .to_string()
                        .contains("No instance found")
                );
                Ok(())
            },
        )?;
        Ok(())
    }

    #[test]
    fn test_resolve_instance_id_error_message_invalid() -> TestResult {
        let temp_dir = tempfile::TempDir::new()?;
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "XDG_DATA_HOME",
            temp_dir_path(&temp_dir),
            || -> TestResult {
                let result = resolve_instance_id("not-a-uuid");
                assert!(result.is_err());
                let err = result
                    .expect_err("expected invalid instance ID error")
                    .to_string();
                assert!(
                    err.contains("Invalid") || err.contains("No instance found"),
                    "unexpected error: {err}"
                );
                Ok(())
            },
        )?;
        Ok(())
    }
}
