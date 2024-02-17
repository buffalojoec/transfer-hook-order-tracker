use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    spl_transfer_hook_interface::get_extra_account_metas_address,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct CreateMintInstruction {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InitializeProfileInstruction {
    pub username: String,
}

pub enum ProtocolInstruction {
    /// Initializes the protocol.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. [w]   Soulbound Mint
    /// 1. [w]   Order Tracker
    /// 2. [w+s] Payer
    /// 3. []    Token-2022 Program
    InitializeProtocol,
    /// Creates a new protocol mint.
    ///
    /// Accounts expected by this instruction:
    /// 0. [w]   Mint
    /// 1. [w]   Validation Account
    /// 2. [s]   Mint Authority
    /// 3. []    Token-2022 Program
    CreateMint(CreateMintInstruction),
    /// Initializes a profile for a user and mints a soulbound token.
    ///
    /// Accounts expected by this instruction:
    /// 0. [w]   Soulbound Mint
    /// 1. [w]   User's Soulbound Token Account
    /// 2. [w]   User's Profile
    /// 3. [w+s] User's Wallet
    /// 4. []    Souldbound Mint Authority
    /// 5. []    Token-2022 Program
    /// 6. []    System Program
    InitializeProfile(InitializeProfileInstruction),
}

impl ProtocolInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = vec![];
        match self {
            Self::InitializeProtocol => {
                buf.push(0);
            }
            Self::CreateMint(data) => {
                buf.push(1);
                buf.append(&mut data.try_to_vec().unwrap());
            }
            Self::InitializeProfile(data) => {
                buf.push(2);
                buf.append(&mut data.try_to_vec().unwrap());
            }
        }
        buf
    }

    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        if input.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let (discriminator, rest) = input.split_first().unwrap();
        Ok(match discriminator {
            0 => Self::InitializeProtocol,
            1 => {
                let data = CreateMintInstruction::try_from_slice(rest)?;
                Self::CreateMint(data)
            }
            2 => {
                let data = InitializeProfileInstruction::try_from_slice(rest)?;
                Self::InitializeProfile(data)
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}

pub fn initialize_protocol(payer_address: &Pubkey) -> Instruction {
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(crate::state::Soulbound::address(), false),
            AccountMeta::new(crate::state::OrderTracker::address(), false),
            AccountMeta::new(*payer_address, true),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
        ],
        data: ProtocolInstruction::InitializeProtocol.pack(),
    }
}

pub fn create_mint(
    mint_address: &Pubkey,
    mint_authority: &Pubkey,
    decimals: u8,
    name: &str,
    symbol: &str,
    uri: &str,
) -> Instruction {
    let mint_authority = *mint_authority;
    let name = name.to_string();
    let symbol = symbol.to_string();
    let uri = uri.to_string();
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(*mint_address, false),
            AccountMeta::new(
                get_extra_account_metas_address(mint_address, &crate::id()),
                false,
            ),
            AccountMeta::new(mint_authority, true),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
        ],
        data: ProtocolInstruction::CreateMint(CreateMintInstruction {
            decimals,
            name,
            symbol,
            uri,
        })
        .pack(),
    }
}

pub fn initialize_profile(wallet_address: &Pubkey, username: &str) -> Instruction {
    let username = username.to_string();
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(crate::state::Soulbound::address(), false),
            AccountMeta::new(
                crate::state::Soulbound::token_account(wallet_address),
                false,
            ),
            AccountMeta::new(crate::state::Profile::address(wallet_address), false),
            AccountMeta::new(*wallet_address, true),
            AccountMeta::new_readonly(crate::state::MintAuthority::address(), false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: ProtocolInstruction::InitializeProfile(InitializeProfileInstruction { username })
            .pack(),
    }
}
