//! Account-backed helpers for shards: PDA derivation and account sizing helpers.

use crate::shard::MappingShard;
use crate::traits::{ShardKey, ShardValue};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;

/// Seed prefix used for deriving shard PDAs.
/// You can change this to a more specific prefix in your application.
pub const SHARD_SEED_PREFIX: &[u8] = b"mapping_shard";

/// Derive a shard PDA for a given program and shard index (u8).
pub fn derive_shard_pda(program_id: &Pubkey, shard_index: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SHARD_SEED_PREFIX, &[shard_index]], program_id)
}

/// Conservative estimate of required account space for a MappingShard<K, V>.
/// This is only an estimate: for production, compute accurate size for the concrete K/V types.
/// - 8 bytes: account discriminator
/// - Anchor/Borsh overhead for vec length and entries is included approximately
pub fn estimate_shard_account_size(k_size: usize, v_size: usize, max_items: usize) -> usize {
    // 8: account discriminator
    // 4: vector length prefix (borsh)
    // each item: k_size + v_size + 4 (for possible length prefixes for variable types)
    8 + 4 + (max_items * (k_size + v_size + 4)) + 32
}

/// Wrapper struct you can use *in your program* for a concrete shard account.
/// Example in your Anchor program:
/// ```ignore
/// #[account]
/// pub struct UserPubkeyShard {
///     pub shard: MappingShard<Pubkey, u64>
/// }
/// ```
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AccountShard<K: ShardKey, V: ShardValue> {
    pub authority: Pubkey,
    pub shard: MappingShard<K, V>,
}

impl<K: ShardKey, V: ShardValue> AccountShard<K, V> {
    pub fn new(authority: Pubkey, shard_id: u8, max_items: u16) -> Self {
        Self {
            authority,
            shard: MappingShard::new(shard_id, max_items),
        }
    }
}
