//! Instruction types

use crate::error::WalletError;
use serde::Serialize;
use solana_program::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    serialize_utils::{read_pubkey, read_u16, read_u8},
};
use std::{collections::BTreeMap, str};

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
        pubkey: Pubkey,
    },
    /// Recovery can reset all your account owners
    Recovery {
        /// public key => key weight
        owners: BTreeMap<Pubkey, u16>,
    },
    /// Invoke an instruction to another program
    Invoke {
        /// The instruction for the wallet to invoke
        instruction: Instruction,
    },
    /// Revoke will freeze wallet
    Revoke,
    /// Say hello
    Hello,
    /// Init an instruction buffer account
    InitInstructionBuffer,
    /// Append instruction to instruction buffer
    AppendPartialInsturciton {
        /// offset
        offset: u16,
        /// data
        data: Vec<u8>,
    },
    /// Run instructions in the instruction buffer
    RunInstructionBuffer {
        /// expected number of instructions
        expected_instruction_count: u16,
    },
    /// Close an insturction buffer
    CloseInstructionBuffer,
}

impl WalletInstruction {
    /// Unpacks a byte buffer into a WalletInstruction
    pub fn unpack(input: &[u8], accounts: &[AccountInfo]) -> Result<Self, ProgramError> {
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
                Self::AddOwner { owners: owners }
            }
            // RemoveOwner
            1 => {
                let mut current = 0;
                let pubkey = read_pubkey(&mut current, rest).unwrap();
                Self::RemoveOwner { pubkey }
            }
            // Recovery
            2 => {
                let mut current = 0;
                let mut owners = BTreeMap::new();
                while current < rest.len() {
                    let pubkey = read_pubkey(&mut current, rest).unwrap();
                    let weight = read_u16(&mut current, rest).unwrap();
                    owners.insert(pubkey, weight);
                }
                Self::Recovery { owners: owners }
            }
            // Invoke
            3 => {
                let mut current = 0;
                let program_id_idx = usize::from(read_u8(&mut current, rest).unwrap());
                let account_len = usize::from(read_u16(&mut current, rest).unwrap());

                let mut invoke_accounts = Vec::new();
                for _ in 0..account_len {
                    let account_idx = usize::from(read_u8(&mut current, rest).unwrap());
                    let account_metadata = read_u8(&mut current, rest).unwrap();

                    let account_meta = AccountMeta {
                        pubkey: *accounts[account_idx].key,
                        is_signer: account_metadata >> 1 & 1 == 1,
                        is_writable: account_metadata & 1 == 1,
                    };
                    invoke_accounts.push(account_meta);
                }

                Self::Invoke {
                    instruction: Instruction {
                        program_id: *accounts[program_id_idx].key,
                        accounts: invoke_accounts,
                        data: rest[current..].to_vec(),
                    },
                }
            }
            4 => Self::Revoke,
            // Hello (testing)
            5 => Self::Hello,
            6 => Self::InitInstructionBuffer,
            7 => {
                let mut current = 0;
                let offset = read_u16(&mut current, rest).unwrap();
                let data = rest[current..].iter().cloned().collect();
                Self::AppendPartialInsturciton { offset, data }
            }
            8 => {
                let mut current = 0;
                let expected_instruction_count = read_u16(&mut current, rest).unwrap();
                Self::RunInstructionBuffer { expected_instruction_count }
            }
            9 => Self::CloseInstructionBuffer,
            _ => return Err(WalletError::InvalidInstruction.into()),
        })
    }
}
