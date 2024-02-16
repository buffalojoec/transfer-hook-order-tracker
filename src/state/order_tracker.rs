use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::pubkey::Pubkey,
    std::collections::HashMap,
};

/// Tracks the volume of orders on mints.
#[derive(BorshDeserialize, BorshSerialize, Debug, Default)]
pub struct OrderTracker {
    /// The volume of orders for each mint.
    pub volume: HashMap<Pubkey, u64>,
}

impl OrderTracker {
    pub fn address() -> Pubkey {
        Pubkey::find_program_address(&[b"order_tracker"], &crate::id()).0
    }

    /// Increment the volume of orders for a mint.
    pub fn increment(&mut self, mint: &Pubkey, amount: u64) {
        let volume = self.volume.entry(*mint).or_insert(0);
        *volume += amount;
    }
}
