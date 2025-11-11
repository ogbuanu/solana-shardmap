# solana-shardmap

[![Crates.io](https://img.shields.io/crates/v/solana-shardmap.svg)](https://crates.io/crates/solana-shardmap)
[![Documentation](https://docs.rs/solana-shardmap/badge.svg)](https://docs.rs/solana-shardmap)
[![License](https://img.shields.io/crates/l/solana-shardmap.svg)](https://github.com/ogbuanu/solana-shardmap/blob/main/LICENSE)

A generic, efficient shard-based mapping primitive for Solana programs that enables horizontal scaling of key-value storage across multiple accounts.

## 🚀 Why solana-shardmap?

Solana programs face unique storage challenges:

- **No native HashMap**: Unlike EVM smart contracts, Solana doesn't provide built-in mapping types
- **Account size limits**: Individual accounts are limited to ~10KB, constraining large datasets
- **Performance bottlenecks**: Large single-account storage becomes inefficient
- **Horizontal scaling**: Need to distribute data across multiple accounts (PDAs) for scalability

`solana-shardmap` solves these problems by providing a production-ready, generic sharding solution that's fully compatible with the Anchor framework.

## ✨ Features

- 🔧 **Generic Design**: Works with any `AnchorSerialize + AnchorDeserialize` types
- 📊 **Batch Operations**: Efficient bulk insert, get, and remove operations
- 📈 **Capacity Management**: Built-in monitoring and optimization tools
- 🏗️ **PDA Helpers**: Utilities for deterministic shard account derivation
- ⚡ **Performance**: Optimized for Solana's account-based architecture
- 🧪 **Well Tested**: Comprehensive test suite covering all functionality
- 📖 **Anchor Compatible**: Seamless integration with Anchor programs

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
solana-shardmap = "0.1.0"
anchor-lang = "0.30.1"
```

## 🏃 Quick Start

### Basic Usage

```rust
use solana_shardmap::{MappingShard, ShardedMap};
use anchor_lang::prelude::Pubkey;

// Create a new shard with capacity for 1000 items
let mut shard = MappingShard::<Pubkey, u64>::new(0, 1000);

// Insert key-value pairs
let user = Pubkey::new_unique();
shard.insert(user, 42).unwrap();

// Retrieve values
assert_eq!(shard.get(&user), Some(42));

// Check capacity and utilization
println!("Utilization: {:.1}%", shard.utilization_percentage());
println!("Remaining capacity: {}", shard.remaining_capacity());
```

### Batch Operations

```rust
use solana_shardmap::{MappingShard, ShardedMap};

let mut shard = MappingShard::<u32, String>::new(0, 100);

// Efficient batch insertion
let items = vec![
    (1, "Alice".to_string()),
    (2, "Bob".to_string()),
    (3, "Charlie".to_string()),
];

let results = shard.insert_batch(items).unwrap();
assert!(results.iter().all(|r| r.is_ok()));

// Batch retrieval
let keys = [1, 2, 3, 999];
let values = shard.get_batch(&keys);
// values = [Some("Alice"), Some("Bob"), Some("Charlie"), None]
```

## 🔧 Anchor Program Integration

### 1. Define Your Account Structure

```rust
use anchor_lang::prelude::*;
use solana_shardmap::{MappingShard, estimate_shard_account_size};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UserProfile {
    pub username: String,
    pub level: u32,
    pub score: u64,
    pub last_active: i64,
}

// Wrapper account for your shard
#[account]
pub struct ProfileShardAccount {
    pub authority: Pubkey,
    pub shard: MappingShard<Pubkey, UserProfile>,
}
```

### 2. Initialize Shards

```rust
#[program]
pub mod my_program {
    use super::*;

    pub fn initialize_shard(
        ctx: Context<InitializeShard>,
        shard_index: u8,
    ) -> Result<()> {
        let shard_account = &mut ctx.accounts.profile_shard;
        shard_account.authority = ctx.accounts.authority.key();
        shard_account.shard = MappingShard::new(shard_index, 100);

        msg!("Initialized shard {} with capacity 100", shard_index);
        Ok(())
    }

    pub fn upsert_profile(
        ctx: Context<UpsertProfile>,
        username: String,
        level: u32,
        score: u64,
    ) -> Result<()> {
        let profile_shard = &mut ctx.accounts.profile_shard;
        let user_key = ctx.accounts.user.key();

        let profile = UserProfile {
            username,
            level,
            score,
            last_active: Clock::get()?.unix_timestamp,
        };

        profile_shard.shard.insert(user_key, profile)?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(shard_index: u8)]
pub struct InitializeShard<'info> {
    #[account(
        init,
        payer = authority,
        space = estimate_shard_account_size(32, 200, 100) + 8,
        seeds = [b"profile_shard", &[shard_index]],
        bump
    )]
    pub profile_shard: Account<'info, ProfileShardAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
```

### 3. Client-Side Integration (TypeScript)

```typescript
import { Program, web3 } from "@coral-xyz/anchor";

class ShardManager {
  constructor(private program: Program, private totalShards: number = 10) {}

  // Distribute users across shards using hash-based selection
  getUserShardIndex(userPubkey: web3.PublicKey): number {
    const hash = require("crypto")
      .createHash("sha256")
      .update(userPubkey.toBuffer())
      .digest();
    return hash.readUInt32BE(0) % this.totalShards;
  }

  getShardPDA(shardIndex: number): [web3.PublicKey, number] {
    return web3.PublicKey.findProgramAddressSync(
      [Buffer.from("profile_shard"), Buffer.from([shardIndex])],
      this.program.programId
    );
  }

  async createProfile(
    user: web3.Keypair,
    username: string,
    level: number,
    score: number
  ): Promise<string> {
    const shardIndex = this.getUserShardIndex(user.publicKey);
    const [shardPDA] = this.getShardPDA(shardIndex);

    return await this.program.methods
      .upsertProfile(username, level, score)
      .accounts({
        profileShard: shardPDA,
        user: user.publicKey,
      })
      .signers([user])
      .rpc();
  }
}
```

## 📊 Advanced Features

### Capacity Management

```rust
// Monitor shard health
let stats = shard.capacity_stats();
println!("Current items: {}/{}", stats.current_items, stats.max_capacity);
println!("Utilization: {:.1}%", stats.utilization_percentage);

// Check if near capacity
if shard.is_near_capacity(80.0) {
    println!("Shard is 80% full - consider creating new shard");
}

// Optimize memory usage
shard.shrink_to_fit();
shard.reserve(50); // Pre-allocate for 50 more items
```

### Batch Operations for Better Performance

```rust
// Instead of individual operations:
// for item in items { shard.insert(item.0, item.1)?; } // ❌ Inefficient

// Use batch operations:
let results = shard.insert_batch(items)?; // ✅ Efficient

// Check if batch would succeed before attempting
if shard.can_insert_batch(&items) {
    let count = shard.try_insert_batch(items)?;
    println!("Successfully inserted {} items", count);
}
```

## 🏗️ Architecture Patterns

### Simple Hash-Based Sharding

```rust
use solana_shardmap::derive_shard_pda;

fn get_user_shard_pda(user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    let hash = hash_pubkey(user);
    let shard_index = (hash % 10) as u8; // 10 shards
    derive_shard_pda(program_id, shard_index)
}
```

### Dynamic Shard Management

```rust
// Monitor and rebalance shards
async fn monitor_shard_health(shards: &[ShardAccount]) {
    for (index, shard) in shards.iter().enumerate() {
        let utilization = shard.shard.utilization_percentage();

        if utilization > 90.0 {
            println!("⚠️  Shard {} is {}% full", index, utilization);
            // Consider creating additional shards
        }
    }
}
```

## 🔍 API Reference

### Core Types

- **`MappingShard<K, V>`**: The main shard structure
- **`ShardedMap<K, V>`**: Trait defining core operations
- **`ShardKey`**: Trait bound for keys (`AnchorSerialize + AnchorDeserialize + Clone + PartialEq + Debug`)
- **`ShardValue`**: Trait bound for values (`AnchorSerialize + AnchorDeserialize + Clone`)

### Key Methods

| Method                     | Description                       |
| -------------------------- | --------------------------------- |
| `new(id, capacity)`        | Create a new shard                |
| `insert(key, value)`       | Insert or update a key-value pair |
| `get(key)`                 | Retrieve a value by key           |
| `remove(key)`              | Remove a key-value pair           |
| `insert_batch(items)`      | Batch insert operation            |
| `get_batch(keys)`          | Batch retrieval operation         |
| `utilization_percentage()` | Get current utilization           |
| `remaining_capacity()`     | Get available space               |
| `is_full()` / `is_empty()` | Check shard state                 |

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run with verbose output:

```bash
cargo test -- --nocapture
```

Check formatting and linting:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

## 📈 Performance Considerations

1. **Shard Size**: Aim for 50-200 items per shard for optimal performance
2. **Batch Operations**: Always prefer batch operations for multiple items
3. **Memory Management**: Use `shrink_to_fit()` and `reserve()` for memory optimization
4. **Monitoring**: Regularly check utilization to know when to create new shards

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built for the Solana ecosystem
- Inspired by the need for scalable storage solutions
- Compatible with the Anchor framework

---

**Made with ❤️ for the Solana developer community**
