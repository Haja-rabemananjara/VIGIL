use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ResourceKey {
    pub resource_type: String,
    pub resource_id: Uuid,
}

#[derive(Clone)]
pub struct PresenceTracker {
    inner: Arc<DashMap<ResourceKey, HashMap<Uuid, usize>>>,
}

impl PresenceTracker {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    pub fn watch(&self, user_id: Uuid, resource_type: String, resource_id: Uuid) -> Vec<Uuid> {
        let key = ResourceKey { resource_type, resource_id };
        let mut entry = self.inner.entry(key).or_default();
        *entry.entry(user_id).or_insert(0) += 1;
        entry.keys().copied().collect()
    }

    pub fn unwatch(&self, user_id: Uuid, resource_type: String, resource_id: Uuid) -> Vec<Uuid> {
        let key = ResourceKey { resource_type, resource_id };
        if let Some(mut entry) = self.inner.get_mut(&key) {
            if let Some(count) = entry.get_mut(&user_id) {
                *count -= 1;
                if *count == 0 {
                    entry.remove(&user_id);
                }
            }
            let watchers: Vec<Uuid> = entry.keys().copied().collect();
            if watchers.is_empty() {
                drop(entry);
                self.inner.remove(&key);
            }
            watchers
        } else {
            vec![]
        }
    }

    pub fn disconnect(&self, user_id: Uuid) -> Vec<(String, Uuid, Vec<Uuid>)> {
        let mut affected = vec![];

        let keys: Vec<ResourceKey> = self
            .inner
            .iter()
            .filter(|entry| entry.value().contains_key(&user_id))
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys {
            if let Some(mut entry) = self.inner.get_mut(&key) {
                entry.remove(&user_id);
                let watchers: Vec<Uuid> = entry.keys().copied().collect();
                let rt = key.resource_type.clone();
                let rid = key.resource_id;
                if watchers.is_empty() {
                    drop(entry);
                    self.inner.remove(&key);
                    affected.push((rt, rid, vec![]));
                } else {
                    affected.push((rt, rid, watchers));
                }
            }
        }

        affected
    }

    pub fn watchers(&self, resource_type: &str, resource_id: Uuid) -> Vec<Uuid> {
        let key = ResourceKey {
            resource_type: resource_type.to_string(),
            resource_id,
        };
        self.inner
            .get(&key)
            .map(|entry| entry.keys().copied().collect())
            .unwrap_or_default()
    }
}
