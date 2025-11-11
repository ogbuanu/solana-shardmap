#[cfg(test)]
mod shardmap_tests {
    use anchor_lang::prelude::Pubkey;

    use crate::shard::MappingShard;
    use crate::traits::ShardedMap;

    #[test]
    fn insert_get_remove_pubkey_u64() {
        let mut shard = MappingShard::<Pubkey, u64>::new(0, 10);
        let key1 = Pubkey::new_unique();
        let key2 = Pubkey::new_unique();
        shard.insert(key1, 100).unwrap();
        shard.insert(key2, 200).unwrap();
        assert_eq!(shard.get(&key1), Some(100));
        assert_eq!(shard.get(&key2), Some(200));
        shard.remove(&key1).unwrap();
        assert_eq!(shard.get(&key1), None);
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

    #[test]
    fn test_batch_operations() {
        let mut shard = MappingShard::<u32, String>::new(0, 10);

        // Test batch insert
        let items = vec![
            (1, "Alice".to_string()),
            (2, "Bob".to_string()),
            (3, "Charlie".to_string()),
        ];
        let results = shard.insert_batch(items).unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));

        // Test batch get
        let keys = [1, 2, 3, 4];
        let values = shard.get_batch(&keys);
        assert_eq!(values[0], Some("Alice".to_string()));
        assert_eq!(values[1], Some("Bob".to_string()));
        assert_eq!(values[2], Some("Charlie".to_string()));
        assert_eq!(values[3], None);

        // Test batch remove
        let remove_keys = [1, 4]; // 1 exists, 4 doesn't
        let remove_results = shard.remove_batch(&remove_keys).unwrap();
        assert!(remove_results[0].is_ok());
        assert!(remove_results[1].is_err());
        assert_eq!(shard.len(), 2);
    }

    #[test]
    fn test_try_insert_batch() {
        let mut shard = MappingShard::<u8, u8>::new(0, 3);

        // Fill to capacity
        shard.insert(1, 10).unwrap();
        shard.insert(2, 20).unwrap();

        // Try to add more items than capacity allows
        let items = vec![(3, 30), (4, 40), (5, 50)];
        let result = shard.try_insert_batch(items);
        assert!(result.is_err()); // Should fail due to capacity

        // Try with items that fit
        let items = vec![(3, 30)];
        let inserted = shard.try_insert_batch(items).unwrap();
        assert_eq!(inserted, 1);
        assert_eq!(shard.len(), 3);
    }

    #[test]
    fn test_capacity_management() {
        let mut shard = MappingShard::<u8, String>::new(0, 5);

        // Test initial capacity
        assert_eq!(shard.remaining_capacity(), 5);
        assert!(shard.is_empty());
        assert!(!shard.is_full());
        assert_eq!(shard.utilization_percentage(), 0.0);
        assert_eq!(shard.load_factor(), 0.0);

        // Add some items
        shard.insert(1, "test1".to_string()).unwrap();
        shard.insert(2, "test2".to_string()).unwrap();

        assert_eq!(shard.remaining_capacity(), 3);
        assert!(!shard.is_empty());
        assert!(!shard.is_full());
        assert_eq!(shard.utilization_percentage(), 40.0);
        assert_eq!(shard.load_factor(), 0.4);

        // Fill to capacity
        shard.insert(3, "test3".to_string()).unwrap();
        shard.insert(4, "test4".to_string()).unwrap();
        shard.insert(5, "test5".to_string()).unwrap();

        assert_eq!(shard.remaining_capacity(), 0);
        assert!(shard.is_full());
        assert_eq!(shard.utilization_percentage(), 100.0);
        assert_eq!(shard.load_factor(), 1.0);

        // Test near capacity
        assert!(shard.is_near_capacity(80.0));
        assert!(shard.is_near_capacity(100.0));
    }

    #[test]
    fn test_resize_capacity() {
        let mut shard = MappingShard::<u8, u8>::new(0, 2);
        shard.insert(1, 10).unwrap();
        shard.insert(2, 20).unwrap();

        // Try to shrink below current size - should fail
        let result = shard.resize_capacity(1);
        assert!(result.is_err());

        // Expand capacity
        let result = shard.resize_capacity(5);
        assert!(result.is_ok());
        assert_eq!(shard.max_capacity(), 5);

        // Should now be able to add more items
        shard.insert(3, 30).unwrap();
        assert_eq!(shard.len(), 3);
    }

    #[test]
    fn test_space_for_new_items() {
        let mut shard = MappingShard::<u8, u8>::new(0, 5);
        shard.insert(1, 10).unwrap();
        shard.insert(2, 20).unwrap();

        // Test with mix of existing and new keys
        let keys = [1, 3, 4, 5, 6]; // 1 exists, others are new
        let space = shard.space_for_new_items(&keys);
        assert_eq!(space, 3); // 4 new keys, but only 3 space remaining

        // Test with all existing keys
        let existing_keys = [1, 2];
        let space = shard.space_for_new_items(&existing_keys);
        assert_eq!(space, 0);

        // Test with all new keys
        let new_keys = [3, 4];
        let space = shard.space_for_new_items(&new_keys);
        assert_eq!(space, 2);
    }

    #[test]
    fn test_clear_and_memory_management() {
        let mut shard = MappingShard::<u8, String>::new(0, 10);

        // Add items
        for i in 1..=5 {
            shard.insert(i, format!("value{}", i)).unwrap();
        }
        assert_eq!(shard.len(), 5);

        // Clear all items
        shard.clear();
        assert_eq!(shard.len(), 0);
        assert!(shard.is_empty());
        assert_eq!(shard.remaining_capacity(), 10);

        // Test reserve
        shard.reserve(5);
        // Note: We can't easily test if reserve worked without accessing private fields,
        // but we can ensure it doesn't crash

        // Test shrink_to_fit
        shard.shrink_to_fit();
        // Similarly, we can't easily verify shrinking worked, but ensure no crash
    }

    #[test]
    fn test_capacity_stats() {
        let mut shard = MappingShard::<u8, u8>::new(0, 10);
        shard.insert(1, 10).unwrap();
        shard.insert(2, 20).unwrap();

        let stats = shard.capacity_stats();
        assert_eq!(stats.current_items, 2);
        assert_eq!(stats.max_capacity, 10);
        assert_eq!(stats.remaining_capacity, 8);
        assert_eq!(stats.utilization_percentage, 20.0);
        assert_eq!(stats.load_factor, 0.2);
        assert!(!stats.is_full);
        assert!(!stats.is_empty);
    }

    #[test]
    fn test_can_insert_batch() {
        let mut shard = MappingShard::<u8, u8>::new(0, 3);
        shard.insert(1, 10).unwrap();

        // Test batch that would fit
        let items_fit = [(2, 20), (3, 30)];
        assert!(shard.can_insert_batch(&items_fit));

        // Test batch that would exceed capacity
        let items_exceed = [(2, 20), (3, 30), (4, 40)];
        assert!(!shard.can_insert_batch(&items_exceed));

        // Test batch with duplicate keys
        let items_duplicate = [(1, 15), (2, 20)]; // 1 already exists
        assert!(shard.can_insert_batch(&items_duplicate)); // Should fit because 1 is update, not new
    }
}
