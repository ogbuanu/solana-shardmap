# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-11-12

### Added

- Initial release of `solana-shardmap`
- Core `MappingShard<K, V>` struct for bounded key-value storage
- `ShardedMap<K, V>` trait defining standard operations
- Batch operations for efficient bulk operations:
  - `insert_batch()` - Bulk insertion with individual result tracking
  - `get_batch()` - Bulk retrieval operations
  - `remove_batch()` - Bulk removal operations
  - `try_insert_batch()` - All-or-nothing batch insertion
- Comprehensive capacity management:
  - `remaining_capacity()` - Available space calculation
  - `utilization_percentage()` - Usage percentage (0-100%)
  - `load_factor()` - Load factor (0.0-1.0)
  - `is_full()` / `is_empty()` - State checking
  - `is_near_capacity()` - Threshold-based capacity warning
  - `capacity_stats()` - Comprehensive capacity statistics
- Memory optimization utilities:
  - `shrink_to_fit()` - Reduce memory footprint
  - `reserve()` - Pre-allocate for performance
  - `clear()` - Remove all items while preserving capacity
  - `resize_capacity()` - Dynamic capacity adjustment
- PDA utilities:
  - `derive_shard_pda()` - Deterministic shard account derivation
  - `estimate_shard_account_size()` - Account space calculation
- Generic `AccountShard<K, V>` wrapper for Anchor account usage
- Comprehensive error handling with `ShardError` enum
- Full Anchor framework compatibility
- Generic trait bounds for any serializable types
- Extensive test suite covering all functionality

### Features

- **Generic Design**: Works with any `AnchorSerialize + AnchorDeserialize + Clone + PartialEq + Debug` key types and `AnchorSerialize + AnchorDeserialize + Clone` value types
- **Performance Optimized**: Batch operations provide significant performance improvements over individual operations
- **Memory Efficient**: Built-in capacity management and memory optimization utilities
- **Production Ready**: Comprehensive error handling and edge case management
- **Well Documented**: Extensive documentation with examples and best practices
- **Fully Tested**: Complete test coverage including edge cases and performance scenarios

### Dependencies

- `anchor-lang = "0.30.1"` - Anchor framework compatibility
- `solana-program = "1.18.0"` - Solana program development utilities

### Documentation

- Comprehensive README with quick start guide
- Detailed API documentation with examples
- Architecture patterns and best practices guide
- Performance optimization recommendations
- Complete TypeScript integration examples

### Technical Details

- Account-based storage model optimized for Solana's architecture
- Hash-based shard distribution for even load balancing
- Bounded capacity design prevents account size limit issues
- Efficient Vec-based storage with capacity management
- Zero-copy serialization through Anchor framework integration
