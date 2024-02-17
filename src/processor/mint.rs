use {
    crate::{
        error::ProtocolError, instruction::CreateMintInstruction, state::validation::ValidationData,
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program::invoke,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    spl_transfer_hook_interface::get_extra_account_metas_address,
};

pub fn process_create_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: CreateMintInstruction,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Accounts expected by this instruction:
    // 0. [w]   Mint
    // 1. [w]   Validation Account
    // 2. [s]   Mint Authority
    // 3. []    Token-2022 Program
    let mint_info = next_account_info(accounts_iter)?;
    let validation_info = next_account_info(accounts_iter)?;
    let mint_authority_info = next_account_info(accounts_iter)?;
    let _token_2022_program_info = next_account_info(accounts_iter)?;

    let CreateMintInstruction {
        decimals,
        name,
        symbol,
        uri,
    } = data;

    // Assert the proper validation account was provided.
    if validation_info.key != &get_extra_account_metas_address(mint_info.key, program_id) {
        return Err(ProtocolError::IncorrectValidationAccount.into());
    }

    // Assert the mint authority is a signer.
    if !mint_authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Create the mint.
    invoke(
        &spl_token_2022::extension::transfer_hook::instruction::initialize(
            &spl_token_2022::id(),
            mint_info.key,
            None,
            Some(crate::id()),
        )
        .unwrap(),
        &[mint_info.clone()],
    )?;
    invoke(
        &spl_token_2022::extension::metadata_pointer::instruction::initialize(
            &spl_token_2022::id(),
            mint_info.key,
            None,
            Some(*mint_info.key),
        )
        .unwrap(),
        &[mint_info.clone()],
    )?;
    invoke(
        &spl_token_2022::instruction::initialize_mint2(
            &spl_token_2022::id(),
            mint_info.key,
            mint_authority_info.key,
            None,
            decimals,
        )
        .unwrap(),
        &[mint_info.clone()],
    )?;
    invoke(
        &spl_token_metadata_interface::instruction::initialize(
            &spl_token_2022::id(),
            mint_info.key,
            mint_authority_info.key,
            mint_info.key,
            mint_authority_info.key,
            name,
            symbol,
            uri,
        ),
        &[mint_info.clone(), mint_authority_info.clone()],
    )?;

    // Create the validation data.
    ValidationData::write_validation_data(&mut validation_info.try_borrow_mut_data()?)?;

    Ok(())
}
