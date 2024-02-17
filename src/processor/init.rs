use {
    crate::state::{OrderTracker, Soulbound},
    borsh::BorshSerialize,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program::invoke,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

pub fn process_init(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Accounts expected by this instruction:
    //
    // 0. [w]   Soulbound Mint
    // 1. [w]   Order Tracker
    // 2. [w+s] Payer
    // 3. []    Token-2022 Program
    let soulbound_mint_info = next_account_info(accounts_iter)?;
    let order_tracker_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let _token_2022_program_info = next_account_info(accounts_iter)?;

    if !payer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Create the soulbound mint.
    invoke(
        &Soulbound::initialize_non_transferrable_instruction(),
        &[soulbound_mint_info.clone(), payer_info.clone()],
    )?;
    invoke(
        &Soulbound::initialize_mint_instruction(),
        &[soulbound_mint_info.clone(), payer_info.clone()],
    )?;

    // Create the order tracker.
    let order_tracker = OrderTracker::default();
    order_tracker.serialize(&mut &mut order_tracker_info.data.borrow_mut()[..])?;

    Ok(())
}
