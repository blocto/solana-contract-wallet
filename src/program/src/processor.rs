//! Program state processor

use crate::{
  state::{Account, Owner, AccountState},
  instruction::WalletInstruction,
  error::WalletError,
};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  info,
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack},
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

  fn process_initialize_wallet(
    wallet_account: &mut Account,
    pubkey: Pubkey,
    weight: u16,
  ) -> ProgramResult {
    wallet_account.state = AccountState::Initialized;
    wallet_account.n_owners = 1;
    wallet_account.owners[0] = Owner {
      pubkey: pubkey,
      weight: weight,
    };

    Ok(())
  }

  /// Process a AddOwner instruction
  fn process_add_owner(
    wallet_account: &mut Account,
    pubkey: Pubkey,
    weight: u16,
  ) -> ProgramResult {
    let n_owners = wallet_account.n_owners;
    wallet_account.owners[usize::from(n_owners)] = Owner {
      pubkey: pubkey,
      weight: weight,
    };

    wallet_account.n_owners = n_owners + 1;

    Ok(())
  }

  /// Process a RemoveOwner instruction
  fn process_remove_owner(
    wallet_account: &mut Account,
    pubkey: Pubkey
  ) -> ProgramResult {
    let n_owners = wallet_account.n_owners;
    // wallet_account.owners[usize::from(n_owners)] = Owner {
    //   pubkey: pubkey,
    //   weight: weight,
    // };

    wallet_account.n_owners = n_owners - 1;

    Ok(())
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
    let mut wallet_account = Self::load_wallet_account(program_id, accounts)?;
    let is_wallet_initialized = wallet_account.is_initialized();

    match instruction {
      WalletInstruction::Hello if is_wallet_initialized => {
        info!("Instruction: Hello");
        Self::process_hello()
      },
      WalletInstruction::AddOwner { pubkey, weight } if !is_wallet_initialized => {
        info!("Instruction: AddOwner (Initialize Wallet)");
        Self::process_initialize_wallet(&mut wallet_account, pubkey, weight)
      },
      WalletInstruction::AddOwner { pubkey, weight } if is_wallet_initialized => {
        info!("Instruction: AddOwner");
        Self::process_add_owner(&mut wallet_account, pubkey, weight)
      },
      WalletInstruction::RemoveOwner { pubkey } if is_wallet_initialized => {
        info!("Instruction: RemoveOwner");
        Self::process_remove_owner(&mut wallet_account, pubkey)
      },
      _ => {
        info!("Invalid instruction");
        Err(WalletError::InvalidInstruction.into())
      }
    }?;

    Self::store_wallet_account(program_id, accounts, wallet_account)
  }
}
