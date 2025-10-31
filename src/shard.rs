use crate::errors::ShardError;
use crate::traits::{ShardKey, ShardValue, ShardedMap};
use anchor_lang::prelude::*;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct CapacityStats {
    pub current_items: usize,
    pub max_capacity: usize,
    pub remaining_capacity: usize,
    pub utilization_percentage: f32,
    pub load_factor: f32,
    pub vec_capacity: usize,
    pub is_full: bool,
    pub is_empty: bool,
}
/// Generic in-memory/account-backed MappingShard.
/// Note: For Anchor account usage, wrap a concrete `MappingShard<K, V>` in a non-generic `#[account]` struct.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct MappingShard<K: ShardKey, V: ShardValue> {
    /// Optional shard id for debugging. If you use PDAs, you can omit storing this.
    pub shard_id: u8,
    /// Bounded list of key-value pairs.
    pub items: Vec<(K, V)>,
    /// cached count (kept as u16 to reduce serialized size)
    pub item_count: u16,
    /// Maximum allowed items in this shard (helps sizing accounts)
    pub max_items: u16,
    /// Phantom type marker so we can keep K,V generic
    _marker: PhantomData<(K, V)>,
}

impl<K: ShardKey, V: ShardValue> MappingShard<K, V> {
    /// Create a new shard instance in memory.
    pub fn new(shard_id: u8, max_items: u16) -> Self {
        let cap = max_items as usize;
        Self {
            shard_id,
            items: Vec::with_capacity(cap),
            item_count: 0,
            max_items,
            _marker: PhantomData,
        }
    }

    /// Convenience to check capacity.
    pub fn can_add_item(&self) -> bool {
        self.items.len() < self.max_items as usize
    }
    pub fn try_insert_batch(&mut self, items: Vec<(K, V)>) -> Result<usize> {
        let available_space = self.max_items as usize - self.items.len();
        let new_items_count = items
            .iter()
            .filter(|(key, _)| !self.items.iter().any(|(k, _)| k == key))
            .count();

        if new_items_count > available_space {
            return err!(ShardError::ShardFull);
        }

        let mut inserted_count = 0;
        for (key, value) in items {
            if self.insert(key, value).is_ok() {
                inserted_count += 1;
            }
        }

        Ok(inserted_count)
    }

    /// Check if batch insert would succeed without modifying the shard
    pub fn can_insert_batch(&self, items: &[(K, V)]) -> bool {
        let available_space = self.max_items as usize - self.items.len();
        let new_items_count = items
            .iter()
            .filter(|(key, _)| !self.items.iter().any(|(k, _)| k == key))
            .count();

        new_items_count <= available_space
    }
    /// Get the number of remaining slots available in this shard
    pub fn remaining_capacity(&self) -> usize {
        self.max_items as usize - self.items.len()
    }

    /// Check if the shard is at maximum capacity
    pub fn is_full(&self) -> bool {
        self.items.len() >= self.max_items as usize
    }

    /// Check if the shard contains no items
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the utilization percentage (0.0 to 100.0)
    pub fn utilization_percentage(&self) -> f32 {
        if self.max_items == 0 {
            return 0.0;
        }
        (self.items.len() as f32 / self.max_items as f32) * 100.0
    }

    /// Get the load factor (0.0 to 1.0) - useful for performance monitoring
    pub fn load_factor(&self) -> f32 {
        if self.max_items == 0 {
            return 0.0;
        }
        self.items.len() as f32 / self.max_items as f32
    }

    /// Check if the shard is near capacity (configurable threshold)
    pub fn is_near_capacity(&self, threshold_percentage: f32) -> bool {
        self.utilization_percentage() >= threshold_percentage
    }

    /// Calculate how many items can be added before hitting capacity
    pub fn space_for_new_items(&self, keys: &[K]) -> usize {
        let new_keys_count = keys
            .iter()
            .filter(|key| !self.items.iter().any(|(k, _)| k == *key))
            .count();

        std::cmp::min(new_keys_count, self.remaining_capacity())
    }

    /// Resize the maximum capacity (useful for account resizing)
    pub fn resize_capacity(&mut self, new_max_items: u16) -> Result<()> {
        if (new_max_items as usize) < self.items.len() {
            return err!(ShardError::InvalidCapacity);
        }

        self.max_items = new_max_items;

        // Adjust Vec capacity if growing significantly
        if new_max_items as usize > self.items.capacity() {
            self.items
                .reserve(new_max_items as usize - self.items.len());
        }

        Ok(())
    }

    /// Shrink the underlying Vec to fit current items (memory optimization)
    pub fn shrink_to_fit(&mut self) {
        self.items.shrink_to_fit();
    }

    /// Reserve space for additional items (performance optimization)
    pub fn reserve(&mut self, additional: usize) {
        let max_additional = self.remaining_capacity();
        let to_reserve = std::cmp::min(additional, max_additional);
        self.items.reserve(to_reserve);
    }

    /// Clear all items but maintain capacity allocation
    pub fn clear(&mut self) {
        self.items.clear();
        self.item_count = 0;
    }
    /// Get comprehensive capacity statistics
    pub fn capacity_stats(&self) -> CapacityStats {
        CapacityStats {
            current_items: self.len(),
            max_capacity: self.max_capacity(),
            remaining_capacity: self.remaining_capacity(),
            utilization_percentage: self.utilization_percentage(),
            load_factor: self.load_factor(),
            vec_capacity: self.items.capacity(),
            is_full: self.is_full(),
            is_empty: self.is_empty(),
        }
    }
}

impl<K, V> ShardedMap<K, V> for MappingShard<K, V>
where
    K: ShardKey,
    V: ShardValue,
{
    fn insert(&mut self, key: K, value: V) -> Result<()> {
        // If key exists, overwrite
        if let Some(pos) = self.items.iter().position(|(k, _)| *k == key) {
            self.items[pos].1 = value;
            return Ok(());
        }

        // Otherwise append if capacity allows
        if !self.can_add_item() {
            return err!(ShardError::ShardFull);
        }

        self.items.push((key, value));
        self.item_count = self.items.len() as u16;
        Ok(())
    }

    fn get(&self, key: &K) -> Option<V> {
        self.items
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    fn remove(&mut self, key: &K) -> Result<()> {
        if let Some(idx) = self.items.iter().position(|(k, _)| k == key) {
            self.items.remove(idx);
            self.item_count = self.items.len() as u16;
            Ok(())
        } else {
            err!(ShardError::KeyNotFound)
        }
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn max_capacity(&self) -> usize {
        self.max_items as usize
    }

    fn insert_batch(&mut self, items: Vec<(K, V)>) -> Result<Vec<Result<()>>> {
        let mut results = Vec::with_capacity(items.len());

        for (key, value) in items {
            let result = self.insert(key, value);
            let is_error = result.is_err();
            results.push(result);

            // Early exit if we hit capacity to avoid unnecessary processing
            if is_error && !self.can_add_item() {
                // Fill remaining results with capacity errors
                for _ in results.len()..results.capacity() {
                    results.push(err!(ShardError::ShardFull));
                }
                break;
            }
        }

        Ok(results)
    }

    fn get_batch(&self, keys: &[K]) -> Vec<Option<V>> {
        keys.iter().map(|key| self.get(key)).collect()
    }

    fn remove_batch(&mut self, keys: &[K]) -> Result<Vec<Result<()>>> {
        let mut results = Vec::with_capacity(keys.len());

        for key in keys {
            results.push(self.remove(key));
        }

        Ok(results)
    }
}
