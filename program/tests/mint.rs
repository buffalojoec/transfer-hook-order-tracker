#![cfg(feature = "test-sbf")]
mod context;

use {
    context::{
        setup, setup_empty_protocol_mint_account, setup_empty_protocol_validation_account,
        setup_wallet, ProtocolTestContext,
    },
    order_tracker::error::ProtocolError,
    solana_program::{program_error::ProgramError, pubkey::Pubkey},
    solana_program_test::tokio,
    solana_sdk::signer::Signer,
};

const DECIMALS: u8 = 0;
const NAME: &'static str = "Joe Token";
const SYMBOL: &'static str = "JOE";
const URI: &'static str = "https://www.joetoken.com";

// Fail incorrect validation account
#[tokio::test]
async fn fail_incorrect_validation_account() {
    let mut context = setup().await;

    let wallet = setup_wallet(&mut context);
    let mint = setup_empty_protocol_mint_account(&mut context);
    setup_empty_protocol_validation_account(&mut context, &mint.pubkey());

    let mut instruction = order_tracker::instruction::create_mint(
        &mint.pubkey(),
        &wallet.pubkey(),
        DECIMALS,
        NAME,
        SYMBOL,
        URI,
    );
    instruction.accounts[1].pubkey = Pubkey::new_unique();

    context
        .expect_error(
            &[instruction],
            &[&wallet],
            (0, ProtocolError::IncorrectValidationAccount),
        )
        .await;
}

// Fail payer not signer
#[tokio::test]
async fn fail_payer_not_signer() {
    let mut context = setup().await;

    let wallet = setup_wallet(&mut context);
    let mint = setup_empty_protocol_mint_account(&mut context);
    setup_empty_protocol_validation_account(&mut context, &mint.pubkey());

    let mut instruction = order_tracker::instruction::create_mint(
        &mint.pubkey(),
        &wallet.pubkey(),
        DECIMALS,
        NAME,
        SYMBOL,
        URI,
    );
    instruction.accounts[2].is_signer = false;

    context
        .expect_error(
            &[instruction],
            &[],
            (0, ProgramError::MissingRequiredSignature),
        )
        .await;
}

// Success
#[tokio::test]
async fn success() {
    let mut context = setup().await;

    let wallet = setup_wallet(&mut context);
    let mint = setup_empty_protocol_mint_account(&mut context);
    setup_empty_protocol_validation_account(&mut context, &mint.pubkey());

    let instruction = order_tracker::instruction::create_mint(
        &mint.pubkey(),
        &wallet.pubkey(),
        DECIMALS,
        NAME,
        SYMBOL,
        URI,
    );

    context.expect_success(&[instruction], &[&wallet]).await;
}
