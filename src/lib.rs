#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("9QXSTPR4Qfv2KxPKEsCsJQHDTyPeTvJPNGuBhmqHYQVB");
