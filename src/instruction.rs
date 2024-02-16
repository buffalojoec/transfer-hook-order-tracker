use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

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
    InitializeProfile {
        /// The user's username.
        username: String,
    },
}

impl ProtocolInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            ProtocolInstruction::InitializeProtocol => {
                buf.push(0);
            }
            ProtocolInstruction::InitializeProfile { username } => {
                buf.push(1);
                buf.extend_from_slice(username.as_bytes());
            }
        };
        buf
    }

    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let instruction = input.first().ok_or(ProgramError::InvalidInstructionData)?;
        match instruction {
            0 => Ok(ProtocolInstruction::InitializeProtocol),
            1 => {
                let username = String::from_utf8(input[1..].to_vec())
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(ProtocolInstruction::InitializeProfile { username })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
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
        data: ProtocolInstruction::InitializeProfile { username }.pack(),
    }
}
