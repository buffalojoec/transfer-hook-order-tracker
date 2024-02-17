use {
    crate::{
        error::ProtocolError,
        state::{OrderTracker, Profile, Soulbound},
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_pack::Pack,
        pubkey::Pubkey,
    },
    spl_token_2022::state::Account as TokenAccount,
};

/// `spl_transfer_hook_interface::execute`
pub fn process_execute(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Accounts expected by this hook:
    // 0. [] Source
    // 1. [] Mint
    // 2. [] Destination
    // 3. [] Source Owner
    // 4. [] Validation Account
    // 5. [] Token-2022 Program
    // 6. [] Associated Token Program
    // 7. [] Soulbound Mint
    // 8. [] Source Owner Soulbound Token Account
    // 9. [w] Source Owner Profile
    // 10. [w] Order Tracker
    let _source_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let _destination_info = next_account_info(accounts_iter)?;
    let source_owner_info = next_account_info(accounts_iter)?;
    let _validation_account_info = next_account_info(accounts_iter)?;
    let _token_2022_program_info = next_account_info(accounts_iter)?;
    let _associated_token_program_info = next_account_info(accounts_iter)?;
    let source_owner_soulbound_token_account_info = next_account_info(accounts_iter)?;
    let source_owner_profile_info = next_account_info(accounts_iter)?;
    let order_tracker_info = next_account_info(accounts_iter)?;

    // Assert the correct soulbound token account was provided.
    if source_owner_soulbound_token_account_info.key
        != &Soulbound::token_account(source_owner_info.key)
    {
        return Err(ProtocolError::IncorrectSoulboundTokenAccount.into());
    }

    // Assert the soulbound token account has one token.
    let token_account =
        TokenAccount::unpack(&source_owner_soulbound_token_account_info.data.borrow())?;
    if token_account.amount != 1 {
        return Err(ProtocolError::SoulboundTokenAccountIsEmpty.into());
    }

    // Assert the source owner's profile exists.
    if source_owner_profile_info.lamports() == 0 {
        return Err(ProtocolError::ProfileNotInitialized.into());
    }

    // Update the user's profile volume.
    let mut profile = Profile::try_from_slice(&source_owner_profile_info.data.borrow())?;
    profile.order_volume += amount;
    profile.serialize(&mut &mut source_owner_profile_info.data.borrow_mut()[..])?;

    // Update the order tracker.
    let mut order_tracker = OrderTracker::try_from_slice(&order_tracker_info.data.borrow())?;
    order_tracker.increment(mint_info.key, amount);
    order_tracker.serialize(&mut &mut order_tracker_info.data.borrow_mut()[..])?;

    Ok(())
}
