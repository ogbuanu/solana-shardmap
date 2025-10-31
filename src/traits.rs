use anchor_lang::prelude::*;

/// Trait representing the operations expected from a sharded map.
pub trait ShardedMap<K, V> {
    fn insert(&mut self, key: K, value: V) -> Result<()>;
    fn get(&self, key: &K) -> Option<V>;
    fn remove(&mut self, key: &K) -> Result<()>;
    fn len(&self) -> usize;
    fn max_capacity(&self) -> usize;

    // New batch operations
    fn insert_batch(&mut self, items: Vec<(K, V)>) -> Result<Vec<Result<()>>>;
    fn get_batch(&self, keys: &[K]) -> Vec<Option<V>>;
    fn remove_batch(&mut self, keys: &[K]) -> Result<Vec<Result<()>>>;
}

/// Convenience trait bounds you can use when declaring generic MappingShard types.
/// AnchorSerialize/AnchorDeserialize are required so types can be stored in Anchor accounts.
pub trait ShardKey: AnchorSerialize + AnchorDeserialize + Clone + PartialEq {}
impl<T> ShardKey for T where T: AnchorSerialize + AnchorDeserialize + Clone + PartialEq {}

pub trait ShardValue: AnchorSerialize + AnchorDeserialize + Clone {}
impl<T> ShardValue for T where T: AnchorSerialize + AnchorDeserialize + Clone {}
