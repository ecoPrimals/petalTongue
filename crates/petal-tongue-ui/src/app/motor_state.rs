// SPDX-License-Identifier: AGPL-3.0-or-later
//! State backing for motor-driven panel updates and notifications.
//!
//! IPC compositions push content via `motor.panel.update` and
//! `motor.notification` — the UI reads from these stores each frame.
use petal_tongue_core::PanelId;
use std::collections::HashMap;
use std::time::Instant;

/// Per-panel content payload delivered by `motor.panel.update`.
#[derive(Debug, Clone)]
pub struct PanelContent {
    pub title: Option<String>,
    pub content: serde_json::Value,
    #[expect(dead_code, reason = "tracked for future staleness rendering")]
    pub updated_at: Instant,
}

/// Store for composition-driven panel content.
#[derive(Debug, Default)]
pub struct PanelContentStore {
    panels: HashMap<String, PanelContent>,
}

impl PanelContentStore {
    pub fn update(
        &mut self,
        panel: PanelId,
        title: Option<String>,
        content: serde_json::Value,
    ) {
        let key = format!("{panel:?}");
        self.panels.insert(
            key,
            PanelContent {
                title,
                content,
                updated_at: Instant::now(),
            },
        );
    }

    #[expect(dead_code, reason = "public API for per-panel lookup, used in tests")]
    #[must_use]
    pub fn get(&self, panel: &PanelId) -> Option<&PanelContent> {
        let key = format!("{panel:?}");
        self.panels.get(&key)
    }

    #[expect(dead_code, reason = "public API for store size, used in tests")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.panels.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }

    /// Iterate over all stored panel content entries.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &PanelContent)> {
        self.panels.iter().map(|(k, v)| (k.as_str(), v))
    }
}

/// A single notification entry.
#[derive(Debug, Clone)]
pub struct NotificationEntry {
    pub level: String,
    pub message: String,
    pub created_at: Instant,
    pub duration_ms: Option<u64>,
}

impl NotificationEntry {
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.duration_ms
            .is_some_and(|d| self.created_at.elapsed().as_millis() > u128::from(d))
    }
}

/// FIFO queue for UI notifications from `motor.notification`.
#[derive(Debug, Default)]
pub struct NotificationQueue {
    entries: Vec<NotificationEntry>,
}

impl NotificationQueue {
    pub fn push(&mut self, level: &str, message: &str, duration_ms: Option<u64>) {
        self.entries.push(NotificationEntry {
            level: level.to_string(),
            message: message.to_string(),
            created_at: Instant::now(),
            duration_ms,
        });
    }

    pub fn drain_expired(&mut self) {
        self.entries.retain(|n| !n.is_expired());
    }

    #[must_use]
    pub fn active(&self) -> &[NotificationEntry] {
        &self.entries
    }

    #[expect(dead_code, reason = "public API for queue size, used in tests")]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_content_store_update_and_get() {
        let mut store = PanelContentStore::default();
        store.update(
            PanelId::SystemDashboard,
            Some("Health".to_string()),
            serde_json::json!({"cpu": 42}),
        );
        assert_eq!(store.len(), 1);
        let entry = store.get(&PanelId::SystemDashboard).expect("present");
        assert_eq!(entry.title.as_deref(), Some("Health"));
        assert_eq!(entry.content["cpu"], 42);
    }

    #[test]
    fn panel_content_store_overwrite() {
        let mut store = PanelContentStore::default();
        store.update(PanelId::LeftSidebar, None, serde_json::json!("v1"));
        store.update(PanelId::LeftSidebar, None, serde_json::json!("v2"));
        assert_eq!(store.len(), 1);
        assert_eq!(
            store.get(&PanelId::LeftSidebar).expect("present").content,
            "v2"
        );
    }

    #[test]
    fn notification_queue_push_and_drain() {
        let mut q = NotificationQueue::default();
        q.push("info", "hello", Some(0));
        assert_eq!(q.len(), 1);
        std::thread::sleep(std::time::Duration::from_millis(5));
        q.drain_expired();
        assert!(q.is_empty());
    }

    #[test]
    fn sticky_notification_never_expires() {
        let mut q = NotificationQueue::default();
        q.push("error", "permanent", None);
        std::thread::sleep(std::time::Duration::from_millis(5));
        q.drain_expired();
        assert_eq!(q.len(), 1);
    }
}
