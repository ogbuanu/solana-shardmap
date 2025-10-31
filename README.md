# solana-shardmap

Generic shard-based mapping primitive for Solana programs (Anchor-compatible).

## Overview

Solana programs do not have a native `mapping`/`HashMap` type as in EVM smart contracts. Large in-account maps quickly hit size and performance limits. `solana-shardmap` provides a generic, Anchor-compatible primitive to shard key-value mappings across multiple accounts (PDAs), allowing your program to scale its state horizontally.

## Features

- `MappingShard<K, V>`: generic shard struct (bounded capacity).
- `ShardedMap` trait: insert/get/remove primitives.
- PDA helper for deterministic shard accounts.
- Anchor-serializable types.
- Unit tests demonstrating `Pubkey -> u64`.

## Quick example

```rust
use solana_shardmap::MappingShard;
use solana_program::pubkey::Pubkey;

let mut shard = MappingShard::<Pubkey, u64>::new(0, 1000);
let user = Pubkey::new_unique();
shard.insert(user, 42).unwrap();
assert_eq!(shard.get(&user), Some(42));
