//! Instruction types

use crate::error::WalletError;
use serde::Serialize;
use solana_program::{
  info,
  instruction::{AccountMeta, Instruction},
  program_error::ProgramError,
  pubkey::Pubkey,
  serialize_utils::{read_pubkey, read_u16},
};
use std::{
  convert::TryInto,
  mem::size_of,
  str,
  collections::BTreeMap,
};

/// Instructions supported by the multisig wallet program.
#[repr(C)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WalletInstruction {
  /// Add a Pubkey to owner list
  AddOwner {
    /// public key => key weight
    owners: BTreeMap<Pubkey, u16>,
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
  /// Recovery can reset all your account owners
  Recovery {
    /// public key => key weight
    owners: BTreeMap<Pubkey, u16>,
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
      // AddOwner
      0 => {
        let mut current = 0;
        let mut owners = BTreeMap::new();
        while current < rest.len() {
          let pubkey = read_pubkey(&mut current, rest).unwrap();
          let weight = read_u16(&mut current, rest).unwrap();
          owners.insert(pubkey, weight);
        }
        Self::AddOwner {owners: owners}
      }
      // RemoveOwner
      1 => {
        let (pubkey, _) = Self::unpack_pubkey(rest)?;
        Self::RemoveOwner { pubkey }
      },
      // Recovery
      2 => {
        let mut current = 0;
        let mut owners = BTreeMap::new();
        while current < rest.len() {
          let pubkey = read_pubkey(&mut current, rest).unwrap();
          let weight = read_u16(&mut current, rest).unwrap();
          owners.insert(pubkey, weight);
        }
        Self::Recovery {owners: owners}
      },
      // Invoke
      3 => {
        let (program_id, rest) = Self::unpack_pubkey(rest)?;
        let (keys_length, mut rest) = rest.split_at(2);
        let keys_length = keys_length
          .try_into()
          .ok()
          .map(u16::from_le_bytes)
          .ok_or(InvalidInstruction)?;

        let mut accounts = Vec::new();
        for _ in 0..usize::from(keys_length) {
          let (pubkey, internel_rest) = Self::unpack_pubkey(rest)?;
          let (&is_signer, internel_rest) = internel_rest.split_first().ok_or(InvalidInstruction)?;
          let (&is_writable, internel_rest) = internel_rest.split_first().ok_or(InvalidInstruction)?;
          rest = internel_rest;

          let account_meta = AccountMeta {
            pubkey: pubkey,
            is_signer: is_signer == true as u8,
            is_writable: is_writable == true as u8,
          };

          info!(bs58::encode(pubkey.to_bytes()).into_string().as_str());

          accounts.push(account_meta);
        }

        Self::Invoke { instruction: Instruction {
          program_id: program_id,
          accounts: accounts,
          data: rest.iter().cloned().collect(),
        }}
      }
      // Hello (testing)
      4 => Self::Hello,
      _ => return Err(WalletError::InvalidInstruction.into()),
    })
  }

  /// Packs a WalletInstruction into a byte buffer.
  pub fn pack(&self) -> Vec<u8> {
    let mut buf = Vec::with_capacity(size_of::<Self>());

    match self {
      &Self::AddOwner {
        owners: _,
      } => {
        buf.push(0);
        // TODO
      },
      &Self::RemoveOwner {
        ref pubkey,
      } => {
        buf.push(1);
        buf.extend_from_slice(pubkey.as_ref());
      }
      &Self::Recovery {
        owners: _,
      } => {
        buf.push(2)
        // TODO
      }
      &Self::Invoke {
        ref instruction,
      } => {
        buf.push(3);
        buf.extend_from_slice(instruction.program_id.as_ref());
        // TODO: Complete invoke instruction packing
      }
      &Self::Hello => {
        buf.push(4);
      },
    }

    buf
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
