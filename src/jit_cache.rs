// src/jit_cache.rs
//! JIT compilation caching system for the Eä programming language.
//!
//! This module provides efficient caching of JIT compilation results to avoid
//! recompiling identical code. It uses content hashing to identify duplicate
//! compilations and stores compiled machine code for immediate reuse.

use crate::error::Result;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Serialization helper for Instant
mod instant_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(instant: &Instant, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert to SystemTime for serialization
        let duration_since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let instant_duration = instant.elapsed();
        let timestamp = duration_since_epoch.saturating_sub(instant_duration);
        serializer.serialize_u64(timestamp.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp_secs = u64::deserialize(deserializer)?;
        let timestamp_duration = Duration::from_secs(timestamp_secs);
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let elapsed = current_time.saturating_sub(timestamp_duration);
        Ok(Instant::now() - elapsed)
    }
}

/// Serialization helper for Duration
mod duration_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_nanos() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos = u64::deserialize(deserializer)?;
        Ok(Duration::from_nanos(nanos))
    }
}

/// A cached JIT compilation result containing compiled machine code and metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CachedJIT {
    /// Hash of the source code that was compiled
    pub source_hash: u64,
    /// Compiled machine code bytes
    pub machine_code: Vec<u8>,
    /// Entry point address for the main function
    pub main_entry_point: usize,
    /// Symbol table mapping function names to addresses
    pub symbol_table: HashMap<String, usize>,
    /// Compilation timestamp (serialized as seconds since Unix epoch)
    #[serde(with = "instant_serde")]
    pub compiled_at: Instant,
    /// Memory usage during compilation
    pub memory_usage: u64,
    /// Compilation time taken (serialized as nanoseconds)
    #[serde(with = "duration_serde")]
    pub compilation_time: Duration,
    /// Cache hit count for statistics
    pub hit_count: u64,
}

/// Configuration for JIT cache behavior
#[derive(Debug, Clone)]
pub struct JITCacheConfig {
    /// Maximum number of cached compilations
    pub max_cache_size: usize,
    /// Maximum age for cached entries (in seconds)
    pub max_cache_age_seconds: u64,
    /// Whether to enable cache statistics
    pub enable_statistics: bool,
    /// Whether to enable cache persistence to disk
    pub enable_persistence: bool,
    /// Cache directory for persistent storage
    pub cache_directory: std::path::PathBuf,
}

impl Default for JITCacheConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 1000,
            max_cache_age_seconds: 3600, // 1 hour
            enable_statistics: true,
            enable_persistence: true,
            cache_directory: std::path::PathBuf::from(".ea_jit_cache"),
        }
    }
}

/// Statistics for JIT cache performance
#[derive(Debug, Default, Clone)]
pub struct JITCacheStats {
    /// Total number of cache lookups
    pub total_lookups: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Total time saved by cache hits
    pub time_saved: Duration,
    /// Total memory saved by cache hits
    pub memory_saved: u64,
    /// Number of cache evictions
    pub evictions: u64,
}

impl JITCacheStats {
    /// Calculate cache hit ratio as a percentage
    pub fn hit_ratio(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / self.total_lookups as f64) * 100.0
        }
    }
}

/// Thread-safe JIT compilation cache
pub struct JITCache {
    /// Cache storage mapping source hash to cached JIT
    cache: Arc<RwLock<HashMap<u64, CachedJIT>>>,
    /// Cache configuration
    config: JITCacheConfig,
    /// Cache statistics
    stats: Arc<RwLock<JITCacheStats>>,
}

impl JITCache {
    /// Create a new JIT cache with default configuration
    pub fn new() -> Self {
        Self::with_config(JITCacheConfig::default())
    }

    /// Create a new JIT cache with custom configuration
    pub fn with_config(config: JITCacheConfig) -> Self {
        // Create cache directory if it doesn't exist
        if config.enable_persistence && !config.cache_directory.exists() {
            let _ = std::fs::create_dir_all(&config.cache_directory);
        }

        let cache = Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(JITCacheStats::default())),
        };

        // Load persisted cache if enabled
        if cache.config.enable_persistence {
            cache.load_from_disk();
        }

        cache
    }

    /// Hash source code for cache key generation
    fn hash_source(&self, source: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    /// Check if a cache entry is still valid based on age
    fn is_cache_entry_valid(&self, cached_jit: &CachedJIT) -> bool {
        let age = cached_jit.compiled_at.elapsed();
        age.as_secs() < self.config.max_cache_age_seconds
    }

    /// Evict old entries to make room for new ones
    fn evict_old_entries(&self) {
        // First, get a read lock to check if eviction is needed
        let cache_size = {
            let cache = self.cache.read().unwrap();
            cache.len()
        };

        if cache_size <= self.config.max_cache_size {
            return; // No eviction needed
        }

        // Now acquire write locks for eviction
        let mut cache = self.cache.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        // Remove expired entries
        let now = Instant::now();
        let max_age = Duration::from_secs(self.config.max_cache_age_seconds);

        cache.retain(|_, cached_jit| {
            // Fix: Use checked_duration_since to avoid panic on clock skew
            let keep = match cached_jit.compiled_at.checked_duration_since(now) {
                Some(_) => true, // compiled_at is in the future, keep it
                None => now.duration_since(cached_jit.compiled_at) < max_age,
            };
            if !keep {
                stats.evictions += 1;
            }
            keep
        });

        // If still over capacity, remove least recently used entries
        if cache.len() > self.config.max_cache_size {
            let entries: Vec<_> = cache.iter().map(|(k, v)| (*k, v.compiled_at)).collect();
            let mut sorted_entries = entries;
            sorted_entries.sort_by_key(|(_, compiled_at)| *compiled_at);

            let evict_count = cache.len() - self.config.max_cache_size;
            let to_remove: Vec<_> = sorted_entries
                .iter()
                .take(evict_count)
                .map(|(hash, _)| *hash)
                .collect();

            for hash in to_remove {
                cache.remove(&hash);
                stats.evictions += 1;
            }
        }
    }

    /// Look up cached JIT compilation result
    pub fn get(&self, source: &str) -> Option<CachedJIT> {
        let source_hash = self.hash_source(source);

        if self.config.enable_statistics {
            let mut stats = self.stats.write().unwrap();
            stats.total_lookups += 1;
        }

        let cache = self.cache.read().unwrap();

        if let Some(cached_jit) = cache.get(&source_hash) {
            if self.is_cache_entry_valid(cached_jit) {
                if self.config.enable_statistics {
                    let mut stats = self.stats.write().unwrap();
                    stats.cache_hits += 1;
                    stats.time_saved += cached_jit.compilation_time;
                    stats.memory_saved += cached_jit.memory_usage;
                }

                // Update hit count
                let mut cached_jit = cached_jit.clone();
                cached_jit.hit_count += 1;

                return Some(cached_jit);
            }
        }

        if self.config.enable_statistics {
            let mut stats = self.stats.write().unwrap();
            stats.cache_misses += 1;
        }

        None
    }

    /// Store a JIT compilation result in the cache
    pub fn put(
        &self,
        source: &str,
        machine_code: Vec<u8>,
        main_entry_point: usize,
        symbol_table: HashMap<String, usize>,
        memory_usage: u64,
        compilation_time: Duration,
    ) -> Result<()> {
        let source_hash = self.hash_source(source);

        let cached_jit = CachedJIT {
            source_hash,
            machine_code,
            main_entry_point,
            symbol_table,
            compiled_at: Instant::now(),
            memory_usage,
            compilation_time,
            hit_count: 0,
        };

        // Store the new entry
        let mut cache = self.cache.write().unwrap();
        cache.insert(source_hash, cached_jit);
        drop(cache); // Release the lock before eviction

        // Evict old entries if needed (after adding the new entry)
        self.evict_old_entries();

        // Save to disk if persistence is enabled
        if self.config.enable_persistence {
            self.save_to_disk();
        }

        Ok(())
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();

        if self.config.enable_statistics {
            let mut stats = self.stats.write().unwrap();
            *stats = JITCacheStats::default();
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> JITCacheStats {
        self.stats.read().unwrap().clone()
    }

    /// Get current cache size
    pub fn size(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.read().unwrap().is_empty()
    }

    /// Load cache from disk
    fn load_from_disk(&self) {
        if !self.config.enable_persistence {
            return;
        }

        let cache_file = self.config.cache_directory.join("jit_cache.json");
        if !cache_file.exists() {
            return;
        }

        match std::fs::read_to_string(&cache_file) {
            Ok(contents) => {
                match serde_json::from_str::<Vec<(u64, CachedJIT)>>(&contents) {
                    Ok(entries) => {
                        let mut cache = self.cache.write().unwrap();
                        for (hash, cached_jit) in entries {
                            // Only load entries that haven't expired
                            if self.is_cache_entry_valid(&cached_jit) {
                                cache.insert(hash, cached_jit);
                            }
                        }
                        eprintln!("✅ Loaded {} entries from JIT cache", cache.len());
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to parse JIT cache file: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to read JIT cache file: {}", e);
            }
        }
    }

    /// Save cache to disk
    fn save_to_disk(&self) {
        if !self.config.enable_persistence {
            return;
        }

        let cache_file = self.config.cache_directory.join("jit_cache.json");
        let cache = self.cache.read().unwrap();

        let entries: Vec<(u64, &CachedJIT)> = cache.iter().map(|(k, v)| (*k, v)).collect();

        match serde_json::to_string_pretty(&entries) {
            Ok(contents) => {
                if let Err(e) = std::fs::write(&cache_file, contents) {
                    eprintln!("❌ Failed to write JIT cache file: {}", e);
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to serialize JIT cache: {}", e);
            }
        }
    }
}

/// Global JIT cache instance
static mut GLOBAL_JIT_CACHE: Option<JITCache> = None;
static CACHE_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global JIT cache
pub fn initialize_jit_cache(config: JITCacheConfig) {
    CACHE_INIT.call_once(|| unsafe {
        GLOBAL_JIT_CACHE = Some(JITCache::with_config(config));
    });
}

/// Get reference to the global JIT cache
pub fn get_jit_cache() -> &'static JITCache {
    unsafe {
        GLOBAL_JIT_CACHE
            .as_ref()
            .expect("JIT cache not initialized")
    }
}

/// Initialize JIT cache with default configuration
pub fn initialize_default_jit_cache() {
    initialize_jit_cache(JITCacheConfig::default());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_jit_cache_basic_operations() {
        let config = JITCacheConfig {
            max_cache_size: 1000,
            max_cache_age_seconds: 3600,
            enable_statistics: true,
            enable_persistence: false, // Disable persistence for testing
            cache_directory: std::path::PathBuf::from(".test_cache"),
        };
        let cache = JITCache::with_config(config);

        // Test cache miss
        assert!(cache.get("test_code").is_none());

        // Store entry
        let machine_code = vec![0x48, 0x89, 0xE5]; // Sample machine code
        let symbol_table = HashMap::new();
        cache
            .put(
                "test_code",
                machine_code.clone(),
                0x1000,
                symbol_table.clone(),
                1024,
                Duration::from_millis(100),
            )
            .unwrap();

        // Test cache hit
        let cached = cache.get("test_code").unwrap();
        assert_eq!(cached.machine_code, machine_code);
        assert_eq!(cached.main_entry_point, 0x1000);
        assert_eq!(cached.memory_usage, 1024);
        assert_eq!(cached.hit_count, 1);
    }

    #[test]
    fn test_jit_cache_statistics() {
        let config = JITCacheConfig {
            max_cache_size: 1000,
            max_cache_age_seconds: 3600,
            enable_statistics: true,
            enable_persistence: false, // Disable persistence for testing
            cache_directory: std::path::PathBuf::from(".test_cache"),
        };
        let cache = JITCache::with_config(config);

        // Test miss
        cache.get("non_existent");
        let stats = cache.get_stats();
        assert_eq!(stats.total_lookups, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.hit_ratio(), 0.0);

        // Store and hit
        cache
            .put(
                "test",
                vec![0x90],
                0x1000,
                HashMap::new(),
                512,
                Duration::from_millis(50),
            )
            .unwrap();
        cache.get("test");

        let stats = cache.get_stats();
        assert_eq!(stats.total_lookups, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.hit_ratio(), 50.0);
    }

    #[test]
    fn test_jit_cache_eviction() {
        // DEVELOPMENT_PROCESS.md: Fix synchronization issues by using isolated cache
        // No global state, no complex timing, just simple cache behavior testing
        let config = JITCacheConfig {
            max_cache_size: 2,
            max_cache_age_seconds: 3600,
            enable_statistics: true,
            enable_persistence: false,
            cache_directory: std::path::PathBuf::from(".test_cache_eviction"),
        };
        let cache = JITCache::with_config(config);

        // Fill cache to capacity
        cache
            .put(
                "code1",
                vec![0x90],
                0x1000,
                HashMap::new(),
                512,
                Duration::from_millis(10),
            )
            .unwrap();
        
        cache
            .put(
                "code2",
                vec![0x91], // Different code
                0x2000,
                HashMap::new(),
                512,
                Duration::from_millis(20),
            )
            .unwrap();
        
        // Cache should be at capacity
        assert_eq!(cache.size(), 2);

        // Add one more - should evict oldest (code1)
        // REAL FIX: Remove problematic sleep that causes test hangs
        cache
            .put(
                "code3",
                vec![0x92], // Different code
                0x3000,
                HashMap::new(),
                512,
                Duration::from_millis(30),
            )
            .unwrap();
        
        // Cache should still be at max capacity
        assert_eq!(cache.size(), 2);

        // DEVELOPMENT_PROCESS.md: Test actual eviction behavior
        // The LRU eviction should have removed the oldest entry (code1)
        assert!(cache.get("code1").is_none(), "Oldest entry should be evicted");
        assert!(cache.get("code2").is_some(), "Second entry should remain");
        assert!(cache.get("code3").is_some(), "Newest entry should exist");
    }

    #[test]
    fn test_jit_cache_age_expiration() {
        let config = JITCacheConfig {
            max_cache_size: 100,
            max_cache_age_seconds: 0, // Immediate expiration
            enable_statistics: true,
            enable_persistence: false,
            cache_directory: std::path::PathBuf::from(".test_cache"),
        };
        let cache = JITCache::with_config(config);

        // Store entry
        cache
            .put(
                "test",
                vec![0x90],
                0x1000,
                HashMap::new(),
                512,
                Duration::from_millis(10),
            )
            .unwrap();

        // Should be expired immediately
        thread::sleep(Duration::from_millis(1));
        assert!(cache.get("test").is_none());
    }

    #[test]
    fn test_source_hashing() {
        let config = JITCacheConfig {
            max_cache_size: 1000,
            max_cache_age_seconds: 3600,
            enable_statistics: true,
            enable_persistence: false, // Disable persistence for testing
            cache_directory: std::path::PathBuf::from(".test_cache"),
        };
        let cache = JITCache::with_config(config);

        // Same source should have same hash
        let hash1 = cache.hash_source("fn main() { print(42); }");
        let hash2 = cache.hash_source("fn main() { print(42); }");
        assert_eq!(hash1, hash2);

        // Different source should have different hash
        let hash3 = cache.hash_source("fn main() { print(43); }");
        assert_ne!(hash1, hash3);
    }
}
