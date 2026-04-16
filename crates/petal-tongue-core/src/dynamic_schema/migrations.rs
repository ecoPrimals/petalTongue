// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::error::{PetalTongueError, Result};

use super::types::{DynamicData, DynamicValue, SchemaVersion};

/// Trait for schema migration
pub trait SchemaMigration {
    /// Check if this migration can handle the version upgrade
    fn can_migrate(&self, from: SchemaVersion, to: SchemaVersion) -> bool;

    /// Perform the migration
    ///
    /// # Errors
    ///
    /// Returns an error if the migration fails (e.g. required fields missing or
    /// incompatible data).
    fn migrate(&self, data: &mut DynamicData, from: SchemaVersion, to: SchemaVersion)
    -> Result<()>;
}

/// Concrete migration from schema v1.0.0 → v2.0.0 (test / reference implementation).
pub struct V1ToV2Migration;

impl SchemaMigration for V1ToV2Migration {
    fn can_migrate(&self, from: SchemaVersion, to: SchemaVersion) -> bool {
        from == SchemaVersion::new(1, 0, 0) && to == SchemaVersion::new(2, 0, 0)
    }

    fn migrate(
        &self,
        data: &mut DynamicData,
        _from: SchemaVersion,
        _to: SchemaVersion,
    ) -> Result<()> {
        data.set("migrated".to_string(), DynamicValue::Boolean(true));
        Ok(())
    }
}

/// Enum dispatch for [`SchemaMigration`] implementations.
pub enum SchemaMigrationImpl {
    /// v1 → v2 upgrade path.
    V1ToV2(V1ToV2Migration),
}

impl SchemaMigration for SchemaMigrationImpl {
    fn can_migrate(&self, from: SchemaVersion, to: SchemaVersion) -> bool {
        match self {
            Self::V1ToV2(m) => m.can_migrate(from, to),
        }
    }

    fn migrate(
        &self,
        data: &mut DynamicData,
        from: SchemaVersion,
        to: SchemaVersion,
    ) -> Result<()> {
        match self {
            Self::V1ToV2(m) => m.migrate(data, from, to),
        }
    }
}

/// Migration registry for managing schema upgrades
#[derive(Default)]
pub struct MigrationRegistry {
    migrations: Vec<SchemaMigrationImpl>,
}

impl MigrationRegistry {
    /// Create a new migration registry
    #[must_use]
    pub const fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Register a migration
    pub fn register(&mut self, migration: SchemaMigrationImpl) {
        self.migrations.push(migration);
    }

    /// Migrate data from one version to another
    ///
    /// # Errors
    ///
    /// Returns an error if no registered migration can handle the version upgrade
    /// from `from` to `to`, or if the selected migration fails.
    pub fn migrate(
        &self,
        data: &mut DynamicData,
        from: SchemaVersion,
        to: SchemaVersion,
    ) -> Result<()> {
        if from == to {
            return Ok(());
        }

        for migration in &self.migrations {
            if migration.can_migrate(from, to) {
                return migration.migrate(data, from, to);
            }
        }

        Err(PetalTongueError::NoMigration(
            from.to_string(),
            to.to_string(),
        ))
    }
}
