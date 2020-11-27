//! Program state processor

use crate::{
  state::Account,
  state::Owner,
  instruction::WalletInstruction,
  error::WalletError,
};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  info,
  program_error::ProgramError,
  program_pack::Pack,
  pubkey::Pubkey,
};
use std::mem;

/// Program state handler.
pub struct Processor {}
impl Processor {
  /// Process a Hello instruction
  fn process_hello(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
  ) -> ProgramResult {
    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    // Get the account to say hello to
    let account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order to modify its data
    if account.owner != program_id {
        info!("Wallet account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // The data must be large enough to hold a u64 count
    if account.try_data_len()? < mem::size_of::<Account>() {
        info!("Wallet account data length too small for Account");
        return Err(ProgramError::InvalidAccountData);
    }

    // Increment and store the number of times the account has been greeted
    let mut stored_account = Account::unpack_unchecked(&account.data.borrow())?;
    // let mut num_greets = LittleEndian::read_u32(&data);
    stored_account.n_owners = 2;
    stored_account.owners[0] = Owner {
      pubkey: *account.owner,
      weight: 1000_u16,
    };

    stored_account.owners[1] = Owner {
      pubkey: *account.key,
      weight: 500_u16,
    };

    Account::pack(stored_account, &mut account.data.borrow_mut())?;

    info!("Hello!");

    Ok(())
  }

  /// Process a WalletInstruction
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
  ) -> ProgramResult {
    let instruction = WalletInstruction::unpack(input)?;

    match instruction {
      WalletInstruction::Hello => {
        info!("Instruction: Hello");
        Self::process_hello(program_id, accounts)
      },
      _ => {
        info!("Invalid instruction");
        Err(WalletError::InvalidInstruction.into())
      }
    }
  }
}
