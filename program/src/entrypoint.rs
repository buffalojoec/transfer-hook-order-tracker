use {
    crate::{error::ProtocolError, processor},
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, program_error::PrintProgramError,
        pubkey::Pubkey,
    },
};

solana_program::entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = processor::process(program_id, accounts, instruction_data) {
        error.print::<ProtocolError>();
        return Err(error);
    }
    Ok(())
}
