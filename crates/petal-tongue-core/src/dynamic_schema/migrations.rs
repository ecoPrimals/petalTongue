// SPDX-License-Identifier: AGPL-3.0-only

use crate::error::{PetalTongueError, Result};

use super::types::{DynamicData, SchemaVersion};

/// Trait for schema migration
pub trait SchemaMigration {
    /// Check if this migration can handle the version upgrade
    fn can_migrate(&self, from: SchemaVersion, to: SchemaVersion) -> bool;

    /// Perform the migration
    fn migrate(&self, data: &mut DynamicData, from: SchemaVersion, to: SchemaVersion)
    -> Result<()>;
}

/// Migration registry for managing schema upgrades
#[derive(Default)]
pub struct MigrationRegistry {
    migrations: Vec<Box<dyn SchemaMigration>>,
}

impl MigrationRegistry {
    /// Create a new migration registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Register a migration
    pub fn register(&mut self, migration: Box<dyn SchemaMigration>) {
        self.migrations.push(migration);
    }

    /// Migrate data from one version to another
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
