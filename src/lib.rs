#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("EwRgcTkaE2inYC62YHYcr5XwS2P8kzBGBa7pQHCHwTSo");
