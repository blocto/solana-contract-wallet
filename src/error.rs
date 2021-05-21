//! Error types

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    info,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum WalletError {
    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
    /// Insufficient funds for the operation requested.
    #[error("Insufficient funds")]
    InsufficientFunds,
    /// Invalid Owner.
    #[error("Invalid owner")]
    InvalidOwner,
    /// Insufficient signature weight.
    #[error("Insufficient weight")]
    InsufficientWeight,
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    /// State is invalid for requested operation.
    #[error("State is invalid for requested operation")]
    InvalidState,
}

impl From<WalletError> for ProgramError {
    fn from(e: WalletError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for WalletError {
    fn type_of() -> &'static str {
        "WalletError"
    }
}

impl PrintProgramError for WalletError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            WalletError::NotRentExempt => info!("WalletError: NotRentExempt"),
            WalletError::InsufficientFunds => info!("WalletError: InsufficientFunds"),
            WalletError::InvalidOwner => info!("WalletError: InvalidOwner"),
            WalletError::InsufficientWeight => info!("WalletError: InsufficientWeight"),
            WalletError::InvalidInstruction => info!("WalletError: InvalidInstruction"),
            WalletError::InvalidState => info!("WalletError: InvalidState"),
        }
    }
}
