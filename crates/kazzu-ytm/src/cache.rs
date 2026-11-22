//! Metadata cache on disk (JSON files with a timestamp).
//!
//! Lets pages open instantly and keeps the app partly usable offline: a
//! stale entry is served when the network fails. Only parsed metadata is
//! stored — never stream URLs (they embed the user's IP) nor visitor data.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Bump when cached types change shape, so old files are ignored.
const SCHEMA: u32 = 1;

/// Entries older than this are deleted by [`DiskCache::prune`].
pub const MAX_AGE: Duration = Duration::from_secs(30 * 24 * 3600);

/// Time-to-live of each kind of page.
#[derive(Debug, Clone, Copy)]
pub struct Ttl;

impl Ttl {
    pub const HOME: Duration = Duration::from_secs(30 * 60);
    pub const CHARTS: Duration = Duration::from_secs(3 * 3600);
    pub const SEARCH: Duration = Duration::from_secs(6 * 3600);
    pub const ALBUM: Duration = Duration::from_secs(7 * 24 * 3600);
    pub const ARTIST: Duration = Duration::from_secs(24 * 3600);
    pub const PLAYLIST: Duration = Duration::from_secs(6 * 3600);
    pub const TRACK: Duration = Duration::from_secs(7 * 24 * 3600);
    pub const LYRICS: Duration = Duration::from_secs(30 * 24 * 3600);
    pub const RADIO: Duration = Duration::from_secs(3600);
    pub const CATEGORIES: Duration = Duration::from_secs(7 * 24 * 3600);
}

/// A cache hit.
#[derive(Debug, Clone)]
pub struct Cached<T> {
    pub value: T,
    pub age: Duration,
}

#[derive(Debug, Clone)]
pub struct DiskCache {
    dir: Option<PathBuf>,
}

/// FNV-1a 64: stable across Rust versions (unlike `DefaultHasher`).
fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl DiskCache {
    /// Cache in `dir` (created on first write).
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: Some(dir.into()) }
    }

    /// A cache that stores nothing.
    pub fn disabled() -> Self {
        Self { dir: None }
    }

    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    fn path(&self, key: &str) -> Option<PathBuf> {
        Some(self.dir.as_ref()?.join(format!("{:016x}.json", fnv1a(key))))
    }

    /// Entry of any age (the caller decides whether it is fresh enough).
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<Cached<T>> {
        let raw = fs::read_to_string(self.path(key)?).ok()?;
        let v: Value = serde_json::from_str(&raw).ok()?;
        if v.get("schema")?.as_u64()? != u64::from(SCHEMA) || v.get("key")?.as_str()? != key {
            return None;
        }
        let stored = v.get("stored_at")?.as_u64()?;
        let value = serde_json::from_value(v.get("data")?.clone()).ok()?;
        Some(Cached {
            value,
            age: Duration::from_secs(now_secs().saturating_sub(stored)),
        })
    }

    /// Stores `value`. Errors are logged and ignored: the cache is optional.
    pub fn put<T: Serialize>(&self, key: &str, value: &T) {
        let Some(path) = self.path(key) else { return };
        let result = (|| -> std::io::Result<()> {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let doc = serde_json::json!({
                "schema": SCHEMA,
                "key": key,
                "stored_at": now_secs(),
                "data": value,
            });
            let tmp = path.with_extension("tmp");
            fs::write(&tmp, serde_json::to_vec(&doc)?)?;
            fs::rename(&tmp, &path)
        })();
        if let Err(e) = result {
            log::warn!("ytm cache: cannot write {}: {e}", path.display());
        }
    }

    pub fn remove(&self, key: &str) {
        if let Some(p) = self.path(key) {
            let _ = fs::remove_file(p);
        }
    }

    /// Deletes entries older than `max_age`. Returns how many were removed.
    pub fn prune(&self, max_age: Duration) -> usize {
        let Some(dir) = &self.dir else { return 0 };
        let Ok(entries) = fs::read_dir(dir) else { return 0 };
        let mut removed = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let old = entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .is_some_and(|age| age > max_age);
            if old && fs::remove_file(&path).is_ok() {
                removed += 1;
            }
        }
        removed
    }

    /// Deletes every entry.
    pub fn clear(&self) {
        self.prune(Duration::ZERO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_key_check() {
        let dir = tempfile::tempdir().unwrap();
        let cache = DiskCache::new(dir.path().join("c"));
        assert!(cache.get::<Vec<u32>>("k").is_none());
        cache.put("k", &vec![1u32, 2, 3]);
        let hit = cache.get::<Vec<u32>>("k").unwrap();
        assert_eq!(hit.value, vec![1, 2, 3]);
        assert!(hit.age < Duration::from_secs(5));
        assert!(cache.get::<String>("k").is_none(), "wrong type is a miss");
        cache.remove("k");
        assert!(cache.get::<Vec<u32>>("k").is_none());
        cache.put("a", &1u8);
        cache.clear();
        assert!(cache.get::<u8>("a").is_none());
        let off = DiskCache::disabled();
        off.put("k", &1u8);
        assert!(off.get::<u8>("k").is_none());
    }
}
