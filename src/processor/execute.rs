#[allow(deprecated)]
use solana_program::borsh0_10::get_instance_packed_len;
use {
    crate::{
        error::ProtocolError,
        state::{OrderTracker, Profile, Soulbound},
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    spl_token_2022::{
        extension::{
            transfer_hook::TransferHookAccount, BaseStateWithExtensions, StateWithExtensions,
        },
        state::Account as TokenAccount,
    },
    spl_transfer_hook_interface::error::TransferHookError,
};

fn get_owner_from_token_account(token_account_info: &AccountInfo) -> Result<Pubkey, ProgramError> {
    let token_account_data = token_account_info.data.borrow();
    let token_account = StateWithExtensions::<TokenAccount>::unpack(&token_account_data)?;
    Ok(token_account.base.owner)
}

fn check_soulbound_token_account(
    token_account_info: &AccountInfo,
    expected_owner: &Pubkey,
) -> Result<(), ProgramError> {
    let token_account_data = token_account_info.data.borrow();
    let token_account = StateWithExtensions::<TokenAccount>::unpack(&token_account_data)?;
    let TokenAccount { amount, owner, .. } = token_account.base;
    if token_account_info.key != &Soulbound::token_account(expected_owner)
        || token_account_info.key != &Soulbound::token_account(&owner)
    {
        return Err(ProtocolError::IncorrectSoulboundTokenAccount.into());
    }
    if amount < 1 {
        return Err(ProtocolError::SoulboundTokenAccountIsEmpty.into());
    }
    Ok(())
}

fn check_token_account_is_transferring(account_info: &AccountInfo) -> Result<(), ProgramError> {
    let account_data = account_info.try_borrow_data()?;
    let token_account = StateWithExtensions::<TokenAccount>::unpack(&account_data)?;
    let extension = token_account.get_extension::<TransferHookAccount>()?;
    if bool::from(extension.transferring) {
        Ok(())
    } else {
        Err(TransferHookError::ProgramCalledOutsideOfTransfer.into())
    }
}

/// `spl_transfer_hook_interface::execute`
pub fn process_execute(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Accounts expected by this hook:
    // 0. []  Source
    // 1. []  Mint
    // 2. []  Destination
    // 3. []  Source Owner
    // 4. []  Validation Account
    // 5. []  Token-2022 Program
    // 6. []  Associated Token Program
    // 7. []  Soulbound Mint
    // 8. []  Source Soulbound Token Account
    // 9. [w] Source Profile
    // 10. []  Destination Soulbound Token Account
    // 11. []  Destination Profile
    // 12. [w] Order Tracker
    let source_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let destination_info = next_account_info(accounts_iter)?;
    let _source_owner_info = next_account_info(accounts_iter)?;
    let _validation_account_info = next_account_info(accounts_iter)?;
    let _token_2022_program_info = next_account_info(accounts_iter)?;
    let _associated_token_program_info = next_account_info(accounts_iter)?;
    let soulbound_mint_info = next_account_info(accounts_iter)?;
    let source_soulbound_token_account_info = next_account_info(accounts_iter)?;
    let source_profile_info = next_account_info(accounts_iter)?;
    let destination_soulbound_token_account_info = next_account_info(accounts_iter)?;
    let destination_profile_info = next_account_info(accounts_iter)?;
    let order_tracker_info = next_account_info(accounts_iter)?;

    // Assert the correct soulbound mint was provided.
    if soulbound_mint_info.key != &Soulbound::address() {
        return Err(ProtocolError::IncorrectSoulboundMint.into());
    }

    // For the source, assert the correct soulbound token account was provided,
    // and that the soulbound token account has one token.
    check_soulbound_token_account(
        source_soulbound_token_account_info,
        &get_owner_from_token_account(source_info)?,
    )?;

    // For the destination, assert the correct soulbound token account was provided,
    // and that the soulbound token account has one token.
    check_soulbound_token_account(
        destination_soulbound_token_account_info,
        &get_owner_from_token_account(destination_info)?,
    )?;

    // Assert the source owner's profile exists.
    if source_profile_info.lamports() == 0 {
        return Err(ProtocolError::ProfileNotInitialized.into());
    }

    // Assert the destination owner's profile exists.
    if destination_profile_info.lamports() == 0 {
        return Err(ProtocolError::ProfileNotInitialized.into());
    }

    // Assert the token accounts are set to transferring.
    // This protects against unwanted invoking of this instruction.
    check_token_account_is_transferring(source_info)?;
    check_token_account_is_transferring(destination_info)?;

    // Update the user's profile volume.
    let mut profile = Profile::try_from_slice(&source_profile_info.data.borrow())?;
    profile.order_volume += amount;
    profile.serialize(&mut &mut source_profile_info.data.borrow_mut()[..])?;

    // Update the order tracker.
    let mut order_tracker = OrderTracker::try_from_slice(&order_tracker_info.data.borrow())?;
    order_tracker.increment(mint_info.key, amount);

    order_tracker_info.realloc(
        #[allow(deprecated)]
        get_instance_packed_len(&order_tracker)?,
        true,
    )?;

    order_tracker.serialize(&mut &mut order_tracker_info.data.borrow_mut()[..])?;

    Ok(())
}
