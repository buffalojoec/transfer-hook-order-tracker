use {
    crate::{
        error::ProtocolError,
        state::{MintAuthority, Profile, Soulbound},
    },
    borsh::BorshSerialize,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    spl_token_2022::{extension::StateWithExtensions, state::Account as TokenAccount},
};

pub fn process_initialize_profile(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    username: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Accounts expected by this instruction:
    // 0. [w]   Soulbound Mint
    // 1. [w]   User's Soulbound Token Account
    // 2. [w]   User's Profile
    // 3. [w+s] User's Wallet
    // 4. []    Souldbound Mint Authority
    // 5. []    Token-2022 Program
    // 6. []    System Program

    let soulbound_mint_info = next_account_info(accounts_iter)?;
    let soulbound_token_account_info = next_account_info(accounts_iter)?;
    let profile_info = next_account_info(accounts_iter)?;
    let wallet_info = next_account_info(accounts_iter)?;
    let soulbound_mint_authority_info = next_account_info(accounts_iter)?;
    let _token_2022_program_info = next_account_info(accounts_iter)?;
    let _system_program_info = next_account_info(accounts_iter)?;

    // Assert the correct soulbound mint was provided.
    if soulbound_mint_info.key != &Soulbound::address() {
        return Err(ProtocolError::IncorrectSoulboundMint.into());
    }

    // Assert the correct soulbound token account was provided.
    if soulbound_token_account_info.key != &Soulbound::token_account(wallet_info.key) {
        return Err(ProtocolError::IncorrectSoulboundTokenAccount.into());
    }

    // Assert the soulbound token account does not have any tokens.
    {
        let token_account_data = soulbound_token_account_info.data.borrow();
        let token_account = StateWithExtensions::<TokenAccount>::unpack(&token_account_data)?;
        if token_account.base.amount != 0 {
            return Err(ProtocolError::SoulboundTokenAccountHasTokens.into());
        }
    }

    // Assert the user's profile does not exist.
    if profile_info.lamports() != 0 {
        return Err(ProtocolError::ProfileAlreadyInitialized.into());
    }

    // Assert the username isn't too long.
    if username.len() > Profile::MAX_LEN {
        return Err(ProtocolError::UsernameTooLong.into());
    }

    // Assert the user's wallet is the signer.
    if !wallet_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Mint the user's soulbound token.
    {
        let seed = MintAuthority::seed();
        let bump = MintAuthority::address_with_bump().1;
        let signer_seeds = &[seed, &[bump]];
        invoke_signed(
            &Soulbound::mint_to_instruction(wallet_info.key),
            &[
                soulbound_mint_info.clone(),
                soulbound_token_account_info.clone(),
                soulbound_mint_authority_info.clone(),
            ],
            &[signer_seeds],
        )?;
    }

    // Initialize the user's profile.
    {
        let seed = Profile::seed();
        let bump = Profile::address_with_bump(wallet_info.key).1;
        let signer_seeds = &[seed, wallet_info.key.as_ref(), &[bump]];
        invoke_signed(
            &Profile::create_account_instruction(wallet_info.key),
            &[profile_info.clone(), wallet_info.clone()],
            &[signer_seeds],
        )?;
    }

    let profile = Profile::new(wallet_info.key, username);
    profile.serialize(&mut &mut profile_info.data.borrow_mut()[..])?;

    Ok(())
}
