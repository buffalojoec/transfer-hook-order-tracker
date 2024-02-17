use {
    solana_program::{instruction::Instruction, pubkey::Pubkey},
    spl_associated_token_account::get_associated_token_address_with_program_id,
    spl_token_2022::ID as TOKEN_2022_PROGRAM_ID,
};

/// A soulbound token.
pub struct Soulbound;

impl Soulbound {
    pub const DECIMALS: u8 = 0;

    pub fn address() -> Pubkey {
        Pubkey::find_program_address(&[b"soulbound"], &crate::id()).0
    }

    /// Get an associated token account address for the soulbound token.
    pub fn token_account(wallet_address: &Pubkey) -> Pubkey {
        get_associated_token_address_with_program_id(
            wallet_address,
            &Self::address(),
            &TOKEN_2022_PROGRAM_ID,
        )
    }

    pub fn initialize_non_transferrable_instruction() -> Instruction {
        spl_token_2022::instruction::initialize_non_transferable_mint(
            &spl_token_2022::id(),
            &Self::address(),
        )
        .unwrap()
    }

    pub fn initialize_mint_instruction() -> Instruction {
        spl_token_2022::instruction::initialize_mint2(
            &spl_token_2022::id(),
            &Self::address(),
            &MintAuthority::address(),
            None,
            Self::DECIMALS,
        )
        .unwrap()
    }

    pub fn mint_to_instruction(wallet_address: &Pubkey) -> Instruction {
        spl_token_2022::instruction::mint_to_checked(
            &spl_token_2022::id(),
            &Self::address(),
            &Self::token_account(wallet_address),
            &MintAuthority::address(),
            &[],
            1,
            Self::DECIMALS,
        )
        .unwrap()
    }
}

/// The protocol's soulbound token Mint Authority.
pub struct MintAuthority;

impl MintAuthority {
    pub fn seed<'s>() -> &'s [u8] {
        b"mint_authority"
    }

    pub fn address_with_bump() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::seed()], &crate::id())
    }

    pub fn address() -> Pubkey {
        Self::address_with_bump().0
    }
}
