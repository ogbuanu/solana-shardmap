use anchor_lang::prelude::*;

#[error_code]
pub enum ShardError {
    #[msg("Shard is full.")]
    ShardFull,
    #[msg("Key not found.")]
    KeyNotFound,
    #[msg("Invalid shard index or PDA mismatch.")]
    InvalidShard,
    #[msg("Invalid capacity: new capacity cannot be smaller than current item count")]
    InvalidCapacity,
}
