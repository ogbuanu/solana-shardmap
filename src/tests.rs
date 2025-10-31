#[cfg(test)]
mod tests {
    use super::*;
    use crate::shard::MappingShard;
    use crate::traits::ShardedMap;
    use solana_program::pubkey::Pubkey;

    #[test]
    fn insert_get_remove_pubkey_u64() {
        let mut shard = MappingShard::<Pubkey, u64>::new(1, 5);
        let pk1 = Pubkey::new_unique();
        let pk2 = Pubkey::new_unique();

        // Insert
        shard.insert(pk1, 100).unwrap();
        assert_eq!(shard.get(&pk1), Some(100));

        // Overwrite
        shard.insert(pk1, 200).unwrap();
        assert_eq!(shard.get(&pk1), Some(200));

        // Insert second
        shard.insert(pk2, 300).unwrap();
        assert_eq!(shard.len(), 2);

        // Remove
        shard.remove(&pk1).unwrap();
        assert_eq!(shard.get(&pk1), None);
        assert_eq!(shard.len(), 1);
    }

    #[test]
    fn shard_capacity_enforced() {
        let mut shard = MappingShard::<u8, u8>::new(0, 2);
        shard.insert(1u8, 10u8).unwrap();
        shard.insert(2u8, 20u8).unwrap();
        let err = shard.insert(3u8, 30u8).err();
        assert!(err.is_some());
    }
}
