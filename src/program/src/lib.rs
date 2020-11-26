#[macro_use]
extern crate serde_derive;

pub mod instruction;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("WaLLeTnuVFfoVWyTs5XjPwJSWd2Ttdd1PrZ7VE3zsUV");
