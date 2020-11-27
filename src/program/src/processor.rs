//! Program state processor

use crate::{
  state::{Account, Owner, AccountState, MIN_WEIGHT, MAX_OWNERS},
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
use std::{
  mem,
  collections::BTreeMap,
};

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
    if weight < MIN_WEIGHT {
      info!("WalletError: Initial key weight too low");
      return Err(WalletError::InvalidInstruction.into())
    }

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

    if usize::from(n_owners) >= MAX_OWNERS {
      info!("WalletError: Already too many owners");
      return Err(WalletError::InvalidInstruction.into())
    }

    for index in 0..usize::from(n_owners) {
      if wallet_account.owners[index].pubkey.to_bytes() == pubkey.to_bytes() {
        info!("WalletError: Owner already exists");
        return Err(WalletError::InvalidInstruction.into())
      }
    }

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

    for index in 0..usize::from(n_owners) {
      if wallet_account.owners[index].pubkey.to_bytes() == pubkey.to_bytes() {
        // Swap current item with the last item and remove the last item
        wallet_account.owners[index] = wallet_account.owners[usize::from(n_owners) - 1];
        wallet_account.owners[usize::from(n_owners) - 1] = Owner {
          pubkey: Pubkey::new_from_array([0; 32]),
          weight: 0,
        };
        wallet_account.n_owners = n_owners - 1;
        return Ok(())
      }
    }

    info!("WalletError: Cannot find the target owner to remove");
    Err(WalletError::InvalidInstruction.into())
  }

  /// Check if signatures have enought weight
  fn check_signatures(
    accounts: &[AccountInfo],
    wallet_account: &Account,
  ) -> ProgramResult {
    let mut weight_map = BTreeMap::new();
    let mut counted = BTreeMap::new();

    for index in 0..usize::from(wallet_account.n_owners) {
      weight_map.insert(
        wallet_account.owners[index].pubkey.to_bytes(),
        wallet_account.owners[index].weight
      );
    }

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    let mut total = 0;
    for account in accounts_iter {
      let key = account.key.to_bytes();
      match weight_map.get(&key) {
        Some(weight) if account.is_signer && counted.get(&key).is_none() => {
          total += weight;
          counted.insert(key, true);
        },
        _ => {}
      };
    }

    if total < MIN_WEIGHT {
      info!("WalletError: Signature weight too low");
      return Err(WalletError::InsufficientWeight.into())
    }

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

    if is_wallet_initialized {
      Self::check_signatures(accounts, &wallet_account)?;
    }

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
