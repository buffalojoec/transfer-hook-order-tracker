mod context;

use {
    crate::context::{setup_soulbound_token_account, setup_wallet_with_soulbound_token_account},
    context::{setup, setup_wallet, ProtocolTestContext},
    order_tracker::{error::ProtocolError, state::Profile},
    solana_program::{program_error::ProgramError, pubkey::Pubkey},
    solana_program_test::tokio,
    solana_sdk::{account::Account, signer::Signer},
};

const USERNAME: &str = "joec";

#[tokio::test]
async fn fail_incorrect_soulbound_mint() {
    let mut context = setup().await;

    let wallet = setup_wallet(&mut context);

    let mut instruction =
        order_tracker::instruction::initialize_profile(&wallet.pubkey(), USERNAME);
    instruction.accounts[0].pubkey = Pubkey::new_unique();

    context
        .expect_error(
            &[instruction],
            &[&wallet],
            (0, ProtocolError::IncorrectSoulboundMint),
        )
        .await;
}

#[tokio::test]
async fn fail_incorrect_soulbound_token_account() {
    let mut context = setup().await;

    let wallet = setup_wallet(&mut context);

    let mut instruction =
        order_tracker::instruction::initialize_profile(&wallet.pubkey(), USERNAME);
    instruction.accounts[1].pubkey = Pubkey::new_unique();

    context
        .expect_error(
            &[instruction],
            &[&wallet],
            (0, ProtocolError::IncorrectSoulboundTokenAccount),
        )
        .await;
}

#[tokio::test]
async fn fail_soulbound_token_account_has_tokens() {
    let mut context = setup().await;

    let wallet = setup_wallet(&mut context);
    setup_soulbound_token_account(&mut context, &wallet.pubkey(), 1);

    context
        .expect_error(
            &[order_tracker::instruction::initialize_profile(
                &wallet.pubkey(),
                USERNAME,
            )],
            &[&wallet],
            (0, ProtocolError::SoulboundTokenAccountHasTokens),
        )
        .await;
}

#[tokio::test]
async fn fail_profile_exists() {
    let mut context = setup().await;

    let wallet = setup_wallet_with_soulbound_token_account(&mut context);

    context.set_account(
        &Profile::address(&wallet.pubkey()),
        &Account {
            lamports: 1_000_000_000,
            owner: order_tracker::id(),
            ..Account::default()
        }
        .into(),
    );

    context
        .expect_error(
            &[order_tracker::instruction::initialize_profile(
                &wallet.pubkey(),
                USERNAME,
            )],
            &[&wallet],
            (0, ProtocolError::ProfileAlreadyInitialized),
        )
        .await;
}

#[tokio::test]
async fn fail_username_too_long() {
    let mut context = setup().await;
    let super_long_username = "a".repeat(Profile::MAX_LEN + 1);

    let wallet = setup_wallet_with_soulbound_token_account(&mut context);

    context
        .expect_error(
            &[order_tracker::instruction::initialize_profile(
                &wallet.pubkey(),
                &super_long_username,
            )],
            &[&wallet],
            (0, ProtocolError::UsernameTooLong),
        )
        .await;
}

#[tokio::test]
async fn fail_wallet_not_signer() {
    let mut context = setup().await;

    let wallet = setup_wallet_with_soulbound_token_account(&mut context);

    let mut instruction =
        order_tracker::instruction::initialize_profile(&wallet.pubkey(), USERNAME);
    instruction.accounts[3].is_signer = false;

    context
        .expect_error(
            &[instruction],
            &[],
            (0, ProgramError::MissingRequiredSignature),
        )
        .await;
}

#[tokio::test]
async fn success() {
    let mut context = setup().await;

    let wallet = setup_wallet_with_soulbound_token_account(&mut context);

    context
        .expect_success(
            &[order_tracker::instruction::initialize_profile(
                &wallet.pubkey(),
                USERNAME,
            )],
            &[&wallet],
        )
        .await;
}