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
  fn process_hello() -> ProgramResult {
    info!("Hello!");

    Ok(())
  }

  /// Process a AddOwner instruction
  fn process_add_owner(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    pubkey: Pubkey,
    weight: u16,
  ) -> ProgramResult {
    let mut wallet_account = Self::load_wallet_account(program_id, accounts)?;
    
    // let mut num_greets = LittleEndian::read_u32(&data);
    let n_owners = wallet_account.n_owners;
    wallet_account.owners[usize::from(n_owners)] = Owner {
      pubkey: pubkey,
      weight: weight,
    };

    wallet_account.n_owners = n_owners + 1;

    Self::store_wallet_account(program_id, accounts, wallet_account)
  }

  /// Load wallet account data
  fn load_wallet_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
  ) -> Result<Account, ProgramError> {
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

    Account::unpack_unchecked(&account.data.borrow())
  }

  /// Store wallet account data
  fn store_wallet_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    wallet_account: Account,
  ) -> Result<(), ProgramError> {

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    // Get the account to say hello to
    let account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order to modify its data
    if account.owner != program_id {
      info!("Wallet account does not have the correct program id");
      return Err(ProgramError::IncorrectProgramId);
    }

    // The account must be declaired writable
    if !account.is_writable {
      info!("Wallet account was not declaired writable");
      return Err(ProgramError::InvalidAccountData);
    }

    Account::pack(wallet_account, &mut account.data.borrow_mut())?;

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
        Self::process_hello()
      },
      WalletInstruction::AddOwner { pubkey, weight } => {
        info!("Instruction: AddOwner");
        Self::process_add_owner(program_id, accounts, pubkey, weight)
      },
      _ => {
        info!("Invalid instruction");
        Err(WalletError::InvalidInstruction.into())
      }
    }
  }
}
