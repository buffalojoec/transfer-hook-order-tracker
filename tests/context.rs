use {
    async_trait::async_trait,
    order_tracker::state::{MintAuthority, Soulbound},
    solana_program::program_error::ProgramError,
    solana_program_test::{
        processor, BanksClient, BanksClientError, ProgramTest, ProgramTestContext,
    },
    solana_sdk::{
        account::Account,
        hash::Hash,
        instruction::{Instruction, InstructionError},
        program_option::COption,
        program_pack::Pack,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
    spl_token_2022::{
        extension::{
            immutable_owner::ImmutableOwner, non_transferable::NonTransferableAccount,
            ExtensionType, StateWithExtensionsMut,
        },
        state::{Account as TokenAccount, AccountState, Mint},
    },
};

pub async fn setup() -> ProgramTestContext {
    let mut program_test = ProgramTest::new(
        "order_tracker",
        order_tracker::id(),
        processor!(order_tracker::processor::process),
    );
    program_test.prefer_bpf(false);
    program_test.add_program(
        "spl_token_2022",
        spl_token_2022::id(),
        processor!(spl_token_2022::processor::Processor::process),
    );

    // Add the soulbound mint authority.
    program_test.add_account(
        MintAuthority::address(),
        Account {
            lamports: 1_000_000_000,
            owner: order_tracker::id(),
            ..Account::default()
        },
    );
    // Add the soulbound mint.
    program_test.add_account(
        Soulbound::address(),
        Account {
            lamports: 1_000_000_000,
            data: vec![0; Mint::LEN],
            owner: spl_token_2022::id(),
            ..Account::default()
        },
    );
    // Add the order tracker.
    program_test.add_account(
        order_tracker::state::OrderTracker::address(),
        Account {
            lamports: 1_000_000_000,
            data: vec![0; 4], // Empty `HashMap`
            owner: order_tracker::id(),
            ..Account::default()
        },
    );

    let mut context = program_test.start_with_context().await;

    // Initialize the protocol.
    let transaction = Transaction::new_signed_with_payer(
        &[order_tracker::instruction::initialize_protocol(
            &context.payer.pubkey(),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    context
}

pub fn setup_wallet(context: &mut ProgramTestContext) -> Keypair {
    let wallet = Keypair::new();
    context.set_account(
        &wallet.pubkey(),
        &Account {
            lamports: 1_000_000_000,
            owner: solana_sdk::system_program::id(),
            ..Account::default()
        }
        .into(),
    );
    wallet
}

pub fn setup_soulbound_token_account(
    context: &mut ProgramTestContext,
    owner: &Pubkey,
    amount: u64,
) {
    let account_size = ExtensionType::try_calculate_account_len::<TokenAccount>(&[
        ExtensionType::ImmutableOwner,
        ExtensionType::NonTransferableAccount,
    ])
    .unwrap();
    let mut account_data = vec![0; account_size];
    let mut state =
        StateWithExtensionsMut::<TokenAccount>::unpack_uninitialized(&mut account_data).unwrap();
    state.init_extension::<ImmutableOwner>(true).unwrap();
    state
        .init_extension::<NonTransferableAccount>(true)
        .unwrap();
    state.base = TokenAccount {
        mint: Soulbound::address(),
        owner: *owner,
        amount,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };
    state.pack_base();
    state.init_account_type().unwrap();

    context.set_account(
        &Soulbound::token_account(owner),
        &Account {
            lamports: 1_000_000_000,
            data: account_data,
            owner: spl_token_2022::id(),
            ..Account::default()
        }
        .into(),
    );
}

pub fn setup_wallet_with_soulbound_token_account(context: &mut ProgramTestContext) -> Keypair {
    let wallet = setup_wallet(context);
    setup_soulbound_token_account(context, &wallet.pubkey(), 0);
    wallet
}

#[async_trait]
pub trait ProtocolTestContext {
    fn banks_client_mut(&mut self) -> &mut BanksClient;
    fn payer(&self) -> Keypair;
    fn last_blockhash(&self) -> Hash;

    fn default_transaction(
        &self,
        instructions: &[Instruction],
        additional_signers: &[&Keypair],
    ) -> Transaction {
        let payer = self.payer();
        let mut signers: Vec<&Keypair> = vec![&payer];
        signers.extend_from_slice(additional_signers);
        Transaction::new_signed_with_payer(
            instructions,
            Some(&payer.pubkey()),
            &signers,
            self.last_blockhash(),
        )
    }

    async fn process_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), BanksClientError> {
        self.banks_client_mut()
            .process_transaction(transaction)
            .await
    }

    async fn expect_success(
        &mut self,
        instructions: &[Instruction],
        additional_signers: &[&Keypair],
    ) {
        let transaction = self.default_transaction(instructions, additional_signers);
        self.process_transaction(transaction).await.unwrap();
    }

    async fn expect_error<E: Into<ProgramError> + Send + Sync>(
        &mut self,
        instructions: &[Instruction],
        additional_signers: &[&Keypair],
        expected: (u8, E),
    ) {
        let (index, e) = expected;
        let expected_code = u64::from(e.into());
        let transaction = self.default_transaction(instructions, additional_signers);
        let err = self
            .process_transaction(transaction)
            .await
            .expect_err("Expected an error")
            .unwrap();
        assert_eq!(
            err,
            TransactionError::InstructionError(index, InstructionError::from(expected_code))
        );
    }
}

impl ProtocolTestContext for ProgramTestContext {
    fn banks_client_mut(&mut self) -> &mut BanksClient {
        &mut self.banks_client
    }

    fn payer(&self) -> Keypair {
        self.payer.insecure_clone()
    }

    fn last_blockhash(&self) -> Hash {
        self.last_blockhash
    }
}
