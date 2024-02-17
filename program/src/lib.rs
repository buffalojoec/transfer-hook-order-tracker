#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("AGmLBzK3SQB66qN1VZBim1f1DAmeR4o2kQUctHRfs8g8");
