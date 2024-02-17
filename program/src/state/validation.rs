use {
    super::{OrderTracker, Profile, Soulbound},
    solana_program::{program_error::ProgramError, pubkey::Pubkey},
    spl_tlv_account_resolution::{
        account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
    },
    spl_transfer_hook_interface::{
        get_extra_account_metas_address, instruction::ExecuteInstruction,
    },
};

pub struct ValidationData;

impl ValidationData {
    pub const NUM_EXTRA_ACCOUNTS: usize = 8;

    pub fn get_len() -> usize {
        ExtraAccountMetaList::size_of(Self::NUM_EXTRA_ACCOUNTS).unwrap()
    }

    pub fn address(mint: &Pubkey) -> Pubkey {
        get_extra_account_metas_address(mint, &crate::id())
    }

    fn extra_metas() -> [ExtraAccountMeta; Self::NUM_EXTRA_ACCOUNTS] {
        [
            // 5: Token-2022 Program
            ExtraAccountMeta::new_with_pubkey(&spl_token_2022::id(), false, false).unwrap(),
            // 6: Associated Token Program
            ExtraAccountMeta::new_with_pubkey(&spl_associated_token_account::id(), false, false)
                .unwrap(),
            // 7: Soulbound Mint
            ExtraAccountMeta::new_with_pubkey(&Soulbound::address(), false, false).unwrap(),
            // 8: Source Owner Soulbound Token Account
            ExtraAccountMeta::new_external_pda_with_seeds(
                6, // Associated Token Program
                &[
                    Seed::AccountKey {
                        index: 3, // Source Owner
                    },
                    Seed::AccountKey {
                        index: 5, // Token-2022 Program
                    },
                    Seed::AccountKey {
                        index: 7, // Soulbound Mint
                    },
                ],
                false,
                false,
            )
            .unwrap(),
            // 9: Source Owner Profile
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal {
                        bytes: Profile::seed().to_vec(),
                    },
                    Seed::AccountKey {
                        index: 3, // Source Owner (wallet)
                    },
                ],
                false,
                true, // writable
            )
            .unwrap(),
            // 10: Destination Owner Soulbound Token Account
            ExtraAccountMeta::new_external_pda_with_seeds(
                6, // Associated Token Program
                &[
                    // Reads the token account owner from the account's data.
                    // See: https://docs.rs/spl-token-2022/latest/spl_token_2022/state/struct.Account.html
                    Seed::AccountData {
                        account_index: 2, // Destination (token account)
                        data_index: 32,   // `owner` field
                        length: 32,       // length of public key
                    },
                    Seed::AccountKey {
                        index: 5, // Token-2022 Program
                    },
                    Seed::AccountKey {
                        index: 7, // Soulbound Mint
                    },
                ],
                false,
                false,
            )
            .unwrap(),
            // 11: Destination Owner Profile
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal {
                        bytes: Profile::seed().to_vec(),
                    },
                    Seed::AccountData {
                        account_index: 2, // Destination (token account)
                        data_index: 32,   // `owner` field
                        length: 32,       // length of public key
                    },
                ],
                false,
                true, // writable
            )
            .unwrap(),
            // 12: Order Tracker
            ExtraAccountMeta::new_with_seeds(
                &[Seed::Literal {
                    bytes: OrderTracker::seed().to_vec(),
                }],
                false,
                true, // writable
            )
            .unwrap(),
        ]
    }

    pub fn write_validation_data(data: &mut [u8]) -> Result<(), ProgramError> {
        ExtraAccountMetaList::init::<ExecuteInstruction>(data, &Self::extra_metas())
    }
}
