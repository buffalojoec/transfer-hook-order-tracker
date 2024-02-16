use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{instruction::Instruction, pubkey::Pubkey, sysvar::Sysvar},
};

/// A user's profile on the protocol.
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Profile {
    pub wallet_address: Pubkey,
    pub username: String,
    pub order_volume: u64,
}

impl Profile {
    pub const MAX_LEN: usize = 140; // 140 characters

    pub fn seed<'s>() -> &'s [u8] {
        b"profile"
    }

    pub fn address_with_bump(wallet_address: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::seed(), wallet_address.as_ref()], &crate::id())
    }

    pub fn address(wallet_address: &Pubkey) -> Pubkey {
        Self::address_with_bump(wallet_address).0
    }

    pub fn new(wallet_address: &Pubkey, username: String) -> Self {
        let wallet_address = *wallet_address;
        Self {
            wallet_address,
            username,
            order_volume: 0,
        }
    }

    pub fn create_account_instruction(wallet_address: &Pubkey) -> Instruction {
        let lamports = solana_program::rent::Rent::get()
            .unwrap()
            .minimum_balance(Self::MAX_LEN);
        solana_program::system_instruction::create_account(
            wallet_address,
            &Self::address(wallet_address),
            lamports,
            Self::MAX_LEN as u64,
            &crate::id(),
        )
    }
}
