//! Instruction types

use crate::error::WalletError;
use serde::Serialize;
use solana_program::{
  instruction::{AccountMeta, Instruction},
  program_error::ProgramError,
  pubkey::Pubkey,
};
use std::convert::TryInto;

/// Instructions supported by the multisig wallet program.
#[repr(C)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WalletInstruction {
  /// Add a Pubkey to owner list
  AddOwner {
    /// The public key to add to the owner list
    pubkey: Pubkey,
    /// Weight of the public key (0-1000)
    weight: u16,
  },
  /// Remove a Pubkey from owner list
  RemoveOwner {
    /// The public key to remove from the owner list
    pubkey: Pubkey
  },
  /// Invoke an instruction to another program
  Invoke {
    /// The instruction for the wallet to invoke
    instruction: Instruction
  },
  /// Say hello
  Hello,
}

impl WalletInstruction {
  /// Unpacks a byte buffer into a WalletInstruction
  pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
    use WalletError::InvalidInstruction;

    let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
    Ok(match tag {
      0 => {
        let (pubkey, rest) = Self::unpack_pubkey(rest)?;
        let weight = rest
          .get(..2)
          .and_then(|slice| slice.try_into().ok())
          .map(u16::from_le_bytes)
          .ok_or(InvalidInstruction)?;
        Self::AddOwner {
          pubkey,
          weight,
        }
      }
      // 1 => Self::InitializeAccount,
      2 => {
        let (program_id, rest) = Self::unpack_pubkey(rest)?;
        Self::Invoke { instruction: Instruction::new(program_id, &vec!(rest), vec!())}
      }
      3 => Self::Hello,
      _ => return Err(WalletError::InvalidInstruction.into()),
    })
  }

  fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
    if input.len() >= 32 {
      let (key, rest) = input.split_at(32);
      let pk = Pubkey::new(key);
      Ok((pk, rest))
    } else {
      Err(WalletError::InvalidInstruction.into())
    }
  }
}
