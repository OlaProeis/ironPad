use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Type of lock held on a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockType {
    Editor,
    TaskView,
}

/// Information about a file lock
#[derive(Debug, Clone, Serialize)]
pub struct LockInfo {
    pub path: String,
    pub client_id: String,
    pub lock_type: LockType,
    pub acquired_at: DateTime<Utc>,
}

/// Error type for lock operations
#[derive(Debug, Clone, Serialize)]
pub enum LockError {
    AlreadyLocked { holder: String, lock_type: LockType },
    NotLocked,
    NotOwner,
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockError::AlreadyLocked { holder, lock_type } => {
                write!(f, "File already locked by {} ({:?})", holder, lock_type)
            }
            LockError::NotLocked => write!(f, "File is not locked"),
            LockError::NotOwner => write!(f, "You do not own this lock"),
        }
    }
}

/// Manages file locks across the application
#[derive(Debug, Clone)]
pub struct FileLockManager {
    locks: Arc<RwLock<HashMap<String, LockInfo>>>,
}

impl FileLockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Attempt to acquire a lock on a file
    pub async fn acquire(
        &self,
        path: &str,
        client_id: &str,
        lock_type: LockType,
    ) -> Result<LockInfo, LockError> {
        let mut locks = self.locks.write().await;

        // Check if already locked
        if let Some(existing) = locks.get(path) {
            if existing.client_id != client_id {
                return Err(LockError::AlreadyLocked {
                    holder: existing.client_id.clone(),
                    lock_type: existing.lock_type,
                });
            }
            // Same client - update lock type
        }

        let lock_info = LockInfo {
            path: path.to_string(),
            client_id: client_id.to_string(),
            lock_type,
            acquired_at: Utc::now(),
        };

        locks.insert(path.to_string(), lock_info.clone());
        Ok(lock_info)
    }

    /// Release a lock on a file
    pub async fn release(&self, path: &str, client_id: &str) -> Result<(), LockError> {
        let mut locks = self.locks.write().await;

        if let Some(existing) = locks.get(path) {
            if existing.client_id != client_id {
                return Err(LockError::NotOwner);
            }
            locks.remove(path);
            Ok(())
        } else {
            Err(LockError::NotLocked)
        }
    }

    /// Check if a file is locked
    pub async fn is_locked(&self, path: &str) -> Option<LockInfo> {
        let locks = self.locks.read().await;
        locks.get(path).cloned()
    }

    /// Check if a file is locked by someone other than the given client
    pub async fn is_locked_by_other(&self, path: &str, client_id: &str) -> Option<LockInfo> {
        let locks = self.locks.read().await;
        locks.get(path).and_then(|lock| {
            if lock.client_id != client_id {
                Some(lock.clone())
            } else {
                None
            }
        })
    }

    /// Release all locks held by a client (used on disconnect)
    pub async fn release_all_for_client(&self, client_id: &str) -> Vec<String> {
        let mut locks = self.locks.write().await;
        let paths_to_remove: Vec<String> = locks
            .iter()
            .filter(|(_, lock)| lock.client_id == client_id)
            .map(|(path, _)| path.clone())
            .collect();

        for path in &paths_to_remove {
            locks.remove(path);
        }

        paths_to_remove
    }

    /// Get all current locks (for debugging/monitoring)
    pub async fn get_all_locks(&self) -> Vec<LockInfo> {
        let locks = self.locks.read().await;
        locks.values().cloned().collect()
    }
}

impl Default for FileLockManager {
    fn default() -> Self {
        Self::new()
    }
}
