mod execute;
mod init;
mod profile;

use {
    crate::instruction::ProtocolInstruction,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
        pubkey::Pubkey,
    },
    spl_transfer_hook_interface::instruction::TransferHookInstruction,
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    if let Ok(instruction) = ProtocolInstruction::unpack(input) {
        match instruction {
            ProtocolInstruction::InitializeProtocol => {
                msg!("Instruction: InitializeProtocol");
                init::process_init(program_id, accounts)
            }
            ProtocolInstruction::InitializeProfile { username } => {
                msg!("Instruction: InitializeProfile");
                profile::process_initialize_profile(program_id, accounts, username)
            }
        }
    } else if let Ok(instruction) = TransferHookInstruction::unpack(input) {
        match instruction {
            TransferHookInstruction::Execute { amount } => {
                msg!("Instruction: Execute");
                execute::process_execute(program_id, accounts, amount)
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    } else {
        Err(ProgramError::InvalidInstructionData)
    }
}
