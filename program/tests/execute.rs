#![cfg(feature = "test-sbf")]
mod context;

use {
    context::{
        setup, setup_empty_protocol_mint_account, setup_empty_protocol_validation_account,
        setup_soulbound_token_account, setup_wallet_with_soulbound_token_account,
        ProtocolTestContext,
    },
    order_tracker::{
        error::ProtocolError,
        state::{Profile, Soulbound},
    },
    solana_program_test::{tokio, ProgramTestContext},
    solana_sdk::{
        account::AccountSharedData, instruction::Instruction, pubkey::Pubkey, signature::Keypair,
        signer::Signer,
    },
    spl_associated_token_account::get_associated_token_address_with_program_id,
    spl_token_2022::offchain::{AccountDataResult, AccountFetchError},
    spl_transfer_hook_interface::{error::TransferHookError, get_extra_account_metas_address},
    std::cell::RefCell,
};

const DECIMALS: u8 = 0;
const NAME: &'static str = "Joe Token";
const SYMBOL: &'static str = "JOE";
const URI: &'static str = "https://www.joetoken.com";

struct ExecuteTestContext {
    context: ProgramTestContext,
    source: Pubkey,
    mint: Keypair,
    destination: Pubkey,
    source_owner: Keypair,
    destination_owner: Keypair,
}
impl ExecuteTestContext {
    async fn get_account_data(&self, pubkey: Pubkey) -> AccountDataResult {
        let client = RefCell::new(self.context.banks_client.clone());
        let mut client = client.borrow_mut();
        client
            .get_account(pubkey)
            .await
            .map_err(AccountFetchError::from)
            .map(|opt| opt.map(|acct| acct.data))
    }

    async fn create_execute_instruction(&self, amount: u64) -> Instruction {
        let mut instruction = spl_transfer_hook_interface::instruction::execute(
            &order_tracker::id(),
            &self.source,
            &self.mint.pubkey(),
            &self.destination,
            &self.source_owner.pubkey(),
            &get_extra_account_metas_address(&self.mint.pubkey(), &order_tracker::id()),
            amount,
        );

        spl_transfer_hook_interface::offchain::add_extra_account_metas_for_execute(
            &mut instruction,
            &order_tracker::id(),
            &self.source,
            &self.mint.pubkey(),
            &self.destination,
            &self.source_owner.pubkey(),
            amount,
            |pubkey| self.get_account_data(pubkey),
        )
        .await
        .unwrap();

        instruction
    }

    async fn create_transfer_checked_instruction(&self, amount: u64) -> Instruction {
        spl_token_2022::offchain::create_transfer_checked_instruction_with_extra_metas(
            &spl_token_2022::id(),
            &self.source,
            &self.mint.pubkey(),
            &self.destination,
            &self.source_owner.pubkey(),
            &[&self.source_owner.pubkey()],
            amount,
            DECIMALS,
            |pubkey| self.get_account_data(pubkey),
        )
        .await
        .unwrap()
    }
}

async fn setup_execute() -> ExecuteTestContext {
    let mut context = setup().await;

    let mint = setup_empty_protocol_mint_account(&mut context);
    setup_empty_protocol_validation_account(&mut context, &mint.pubkey());

    let source_owner = setup_wallet_with_soulbound_token_account(&mut context);
    let source = get_associated_token_address_with_program_id(
        &source_owner.pubkey(),
        &mint.pubkey(),
        &spl_token_2022::id(),
    );

    let destination_owner = setup_wallet_with_soulbound_token_account(&mut context);
    let destination = get_associated_token_address_with_program_id(
        &destination_owner.pubkey(),
        &mint.pubkey(),
        &spl_token_2022::id(),
    );

    let create_mint_instruction = order_tracker::instruction::create_mint(
        &mint.pubkey(),
        &source_owner.pubkey(),
        DECIMALS,
        NAME,
        SYMBOL,
        URI,
    );
    let create_source_instruction =
        spl_associated_token_account::instruction::create_associated_token_account(
            &context.payer.pubkey(),
            &source_owner.pubkey(),
            &mint.pubkey(),
            &spl_token_2022::id(),
        );
    let create_destination_instruction =
        spl_associated_token_account::instruction::create_associated_token_account(
            &context.payer.pubkey(),
            &destination_owner.pubkey(),
            &mint.pubkey(),
            &spl_token_2022::id(),
        );
    let mint_to_instruction = spl_token_2022::instruction::mint_to_checked(
        &spl_token_2022::id(),
        &mint.pubkey(),
        &source,
        &source_owner.pubkey(),
        &[&source_owner.pubkey()],
        100,
        DECIMALS,
    )
    .unwrap();
    let initialize_source_profile_instruction =
        order_tracker::instruction::initialize_profile(&source_owner.pubkey());
    let initialize_destination_profile_instruction =
        order_tracker::instruction::initialize_profile(&destination_owner.pubkey());

    context
        .expect_success(
            &[
                create_mint_instruction,
                create_source_instruction,
                create_destination_instruction,
                mint_to_instruction,
                initialize_source_profile_instruction,
                initialize_destination_profile_instruction,
            ],
            &[&source_owner, &destination_owner],
        )
        .await;

    ExecuteTestContext {
        context,
        source,
        mint,
        destination,
        source_owner,
        destination_owner,
    }
}

#[tokio::test]
async fn fail_incorrect_soulbound_mint() {
    let context = setup_execute().await;

    let mut instruction = context.create_execute_instruction(10).await;
    instruction.accounts[7].pubkey = Pubkey::new_unique();

    let ExecuteTestContext { mut context, .. } = context;

    context
        .expect_error(
            &[instruction],
            &[],
            (0, ProtocolError::IncorrectSoulboundMint),
        )
        .await;
}

#[tokio::test]
async fn fail_incorrect_source_soulbound_token_account() {
    let context = setup_execute().await;

    let mut instruction = context.create_execute_instruction(10).await;

    let ExecuteTestContext { mut context, .. } = context;

    let fake_address = Pubkey::new_unique();
    let fake_soulbound_token_account_address = Soulbound::token_account(&fake_address);
    setup_soulbound_token_account(&mut context, &fake_address, 1);
    instruction.accounts[8].pubkey = fake_soulbound_token_account_address;

    context
        .expect_error(
            &[instruction],
            &[],
            (0, ProtocolError::IncorrectSoulboundTokenAccount),
        )
        .await;
}

#[tokio::test]
async fn fail_source_soulbound_token_account_has_no_tokens() {
    let context = setup_execute().await;

    let instruction = context.create_execute_instruction(10).await;

    let ExecuteTestContext {
        mut context,
        source_owner,
        ..
    } = context;

    setup_soulbound_token_account(&mut context, &source_owner.pubkey(), 0);

    context
        .expect_error(
            &[instruction],
            &[],
            (0, ProtocolError::SoulboundTokenAccountIsEmpty),
        )
        .await;
}

#[tokio::test]
async fn fail_incorrect_destination_soulbound_token_account() {
    let context = setup_execute().await;

    let mut instruction = context.create_execute_instruction(10).await;

    let ExecuteTestContext { mut context, .. } = context;

    let fake_address = Pubkey::new_unique();
    let fake_soulbound_token_account_address = Soulbound::token_account(&fake_address);
    setup_soulbound_token_account(&mut context, &fake_address, 1);
    instruction.accounts[10].pubkey = fake_soulbound_token_account_address;

    context
        .expect_error(
            &[instruction],
            &[],
            (0, ProtocolError::IncorrectSoulboundTokenAccount),
        )
        .await;
}

#[tokio::test]
async fn fail_destination_soulbound_token_account_has_no_tokens() {
    let context = setup_execute().await;

    let instruction = context.create_execute_instruction(10).await;

    let ExecuteTestContext {
        mut context,
        destination_owner,
        ..
    } = context;

    setup_soulbound_token_account(&mut context, &destination_owner.pubkey(), 0);

    context
        .expect_error(
            &[instruction],
            &[],
            (0, ProtocolError::SoulboundTokenAccountIsEmpty),
        )
        .await;
}

#[tokio::test]
async fn fail_source_profile_does_not_exist() {
    let context = setup_execute().await;

    let instruction = context.create_execute_instruction(10).await;

    let ExecuteTestContext {
        mut context,
        source_owner,
        ..
    } = context;

    context.set_account(
        &Profile::address(&source_owner.pubkey()),
        &AccountSharedData::default(),
    );

    context
        .expect_error(
            &[instruction],
            &[],
            (0, ProtocolError::ProfileNotInitialized),
        )
        .await;
}

#[tokio::test]
async fn fail_destination_profile_does_not_exist() {
    let context = setup_execute().await;

    let instruction = context.create_execute_instruction(10).await;

    let ExecuteTestContext {
        mut context,
        destination_owner,
        ..
    } = context;

    context.set_account(
        &Profile::address(&destination_owner.pubkey()),
        &AccountSharedData::default(),
    );

    context
        .expect_error(
            &[instruction],
            &[],
            (0, ProtocolError::ProfileNotInitialized),
        )
        .await;
}

#[tokio::test]
async fn fail_cannot_invoke_directly() {
    let context = setup_execute().await;

    let instruction = context.create_execute_instruction(10).await;

    let ExecuteTestContext { mut context, .. } = context;

    context
        .expect_error(
            &[instruction],
            &[],
            (0, TransferHookError::ProgramCalledOutsideOfTransfer),
        )
        .await;
}

#[tokio::test]
async fn success() {
    let context = setup_execute().await;

    let instruction = context.create_transfer_checked_instruction(10).await;

    let ExecuteTestContext {
        mut context,
        source_owner: wallet,
        ..
    } = context;

    context.expect_success(&[instruction], &[&wallet]).await;
}
