//! Program state processor

use crate::{
  error::WalletError,
  instruction::WalletInstruction,
  state::{Account, AccountState, MAX_OWNERS, MIN_WEIGHT},
};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  info,
  instruction::Instruction,
  program::invoke_signed,
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack},
  pubkey::Pubkey,
};
use std::{collections::BTreeMap, mem};

/// Program state handler.
pub struct Processor {}
impl Processor {
  /// Process a Hello instruction
  fn process_hello() -> ProgramResult {
    info!("Hello!");

    Ok(())
  }

  /// Process an AddOwner instruction and initialize the wallet
  fn process_initialize_wallet(
    wallet_account: &mut Account,
    owners: BTreeMap<Pubkey, u16>,
  ) -> ProgramResult {
    wallet_account.state = AccountState::Initialized;

    for (pubkey, weight) in owners {
      wallet_account.owners.insert(pubkey, weight);
    }

    Ok(())
  }

  /// Process an AddOwner instruction
  fn process_add_owner(
    wallet_account: &mut Account,
    owners: BTreeMap<Pubkey, u16>,
  ) -> ProgramResult {
    if wallet_account.owners.len() + owners.len() > MAX_OWNERS {
      info!("WalletError: too many owners");
      return Err(WalletError::InvalidInstruction.into());
    }

    for (pubkey, weight) in owners {
      if weight == 0 {
        info!("WalletError: Key weight cannot be 0");
        return Err(WalletError::InvalidInstruction.into());
      }
      if wallet_account.owners.contains_key(&pubkey) {
        info!("WalletError: Owner already exists");
        return Err(WalletError::InvalidInstruction.into());
      }
      wallet_account.owners.insert(pubkey, weight);
    }

    Ok(())
  }

  /// Process a RemoveOwner instruction
  fn process_remove_owner(wallet_account: &mut Account, pubkey: Pubkey) -> ProgramResult {
    // check target exist
    if !wallet_account.owners.contains_key(&pubkey) {
      info!("WalletError: Cannot find the target owner to remove");
      return Err(WalletError::InvalidInstruction.into());
    }

    // remove
    wallet_account.owners.remove(&pubkey);
    Ok(())
  }

  /// Process an Recovery instruction
  fn process_recovery(
    wallet_account: &mut Account,
    owners: BTreeMap<Pubkey, u16>,
  ) -> ProgramResult {
    if owners.len() > MAX_OWNERS {
      info!("WalletError: too many owners");
      return Err(WalletError::InvalidInstruction.into());
    }

    wallet_account.owners.clear();

    for (pubkey, weight) in owners {
      if weight == 0 {
        info!("WalletError: Key weight cannot be 0");
        return Err(WalletError::InvalidInstruction.into());
      }
      if wallet_account.owners.contains_key(&pubkey) {
        info!("WalletError: Owner already exists");
        return Err(WalletError::InvalidInstruction.into());
      }
      wallet_account.owners.insert(pubkey, weight);
    }

    Ok(())
  }

  /// Process an Invoke instruction and call another program
  fn process_invoke(accounts: &[AccountInfo], instruction: Instruction) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let wallet_account = next_account_info(accounts_iter)?;
    let auth_account = next_account_info(accounts_iter)?;

    let mut pass_accounts = Vec::new();

    // Pass all accounts to invoke call
    // info!(bs58::encode(wallet_account.key.to_bytes()).into_string().as_str());
    pass_accounts.push(wallet_account.clone());
    // info!(bs58::encode(auth_account.key.to_bytes()).into_string().as_str());
    pass_accounts.push(auth_account.clone());

    for account in accounts_iter {
      // info!(bs58::encode(account.key.to_bytes()).into_string().as_str());
      pass_accounts.push(account.clone());
    }

    invoke_signed(
      &instruction,
      pass_accounts.as_slice(),
      &[&[&wallet_account.key.to_bytes()]],
    )?;

    Ok(())
  }

  /// Check if signatures have enought weight
  fn check_signatures(accounts: &[AccountInfo], wallet_account: &Account) -> ProgramResult {
    let mut total_key_weight = 0;
    let mut counted = BTreeMap::new();

    for account in accounts.iter() {
      if account.is_signer && wallet_account.owners.contains_key(account.key) && !counted.contains_key(account.key) {
        counted.insert(account.key, true);
        total_key_weight += wallet_account.owners[account.key];
      }
    }

    if total_key_weight < MIN_WEIGHT {
      info!("WalletError: Signature weight too low");
      return Err(WalletError::InsufficientWeight.into());
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

    // The account containing wallet information
    let walllet_account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order to modify its data
    if walllet_account.owner != program_id {
      info!("Wallet account does not have the correct program id");
      return Err(ProgramError::IncorrectProgramId);
    }

    // The data must be large enough to hold a u64 count
    if walllet_account.try_data_len()? < mem::size_of::<Account>() {
      info!("Wallet account data length too small for Account");
      return Err(ProgramError::InvalidAccountData);
    }

    Account::unpack_unchecked(&walllet_account.data.borrow())
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
  pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let mut wallet_account = Self::load_wallet_account(program_id, accounts)?;
    let is_wallet_initialized = wallet_account.is_initialized();

    if is_wallet_initialized {
      Self::check_signatures(accounts, &wallet_account)?;
    }

    let instruction = WalletInstruction::unpack(input)?;

    match instruction {
      WalletInstruction::Hello if is_wallet_initialized => {
        info!("Instruction: Hello");
        Self::process_hello()
      }
      WalletInstruction::AddOwner { owners } if !is_wallet_initialized => {
        info!("Instruction: AddOwner (Initialize Wallet)");
        Self::process_initialize_wallet(&mut wallet_account, owners)
      }
      WalletInstruction::AddOwner { owners } if is_wallet_initialized => {
        info!("Instruction: AddOwner");
        Self::process_add_owner(&mut wallet_account, owners)
      }
      WalletInstruction::RemoveOwner { pubkey } if is_wallet_initialized => {
        info!("Instruction: RemoveOwner");
        Self::process_remove_owner(&mut wallet_account, pubkey)
      }
      WalletInstruction::Recovery { owners } if is_wallet_initialized => {
        info!("Instruction: Recovery");
        Self::process_recovery(&mut wallet_account, owners)
      }
      WalletInstruction::Invoke {
        instruction: internal_instruction,
      } if is_wallet_initialized => {
        info!("Instruction: Invoke");
        Self::process_invoke(accounts, internal_instruction)
      }
      _ => {
        info!("Invalid instruction");
        Err(WalletError::InvalidInstruction.into())
      }
    }?;

    Self::store_wallet_account(program_id, accounts, wallet_account)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::str::FromStr;

  #[test]
  fn test_process_initialize_wallet() {
    let pubkey1 = Pubkey::from_str("EvN4kgKmCmYzdbd5kL8Q8YgkUW5RoqMTpBczrfLExtx7").unwrap();
    let pubkey2 = Pubkey::from_str("A4iUVr5KjmsLymUcv4eSKPedUtoaBceiPeGipKMYc69b").unwrap();

    let mut wallet_account = Account {
      state: AccountState::Uninitialized,
      owners: BTreeMap::new(),
    };
    let mut init_keys = BTreeMap::new();
    init_keys.insert(pubkey1, 999);
    init_keys.insert(pubkey2, 1);
    assert_eq!(
      Processor::process_initialize_wallet(&mut wallet_account, init_keys),
      Ok(())
    );

    let mut expected_account = Account {
      state: AccountState::Initialized,
      owners: BTreeMap::<Pubkey, u16>::new(),
    };
    expected_account.owners.insert(pubkey1, 999);
    expected_account.owners.insert(pubkey2, 1);

    assert_eq!(wallet_account, expected_account);
  }

  #[test]
  fn test_process_add_owner() {
    let pubkey1 = Pubkey::from_str("EvN4kgKmCmYzdbd5kL8Q8YgkUW5RoqMTpBczrfLExtx7").unwrap();
    let pubkey2 = Pubkey::from_str("A4iUVr5KjmsLymUcv4eSKPedUtoaBceiPeGipKMYc69b").unwrap();

    let mut wallet_account = Account {
      state: AccountState::Initialized,
      owners: BTreeMap::new(),
    };
    wallet_account.owners.insert(pubkey1, 999);

    let mut add_keys = BTreeMap::new();
    add_keys.insert(pubkey2, 1);
    assert_eq!(
      Processor::process_add_owner(&mut wallet_account, add_keys),
      Ok(())
    );

    let mut expected_account = Account {
      state: AccountState::Initialized,
      owners: BTreeMap::<Pubkey, u16>::new(),
    };
    expected_account.owners.insert(pubkey1, 999);
    expected_account.owners.insert(pubkey2, 1);

    assert_eq!(wallet_account, expected_account);
  }

  #[test]
  fn test_process_recovery() {
    let pubkey1 = Pubkey::from_str("EvN4kgKmCmYzdbd5kL8Q8YgkUW5RoqMTpBczrfLExtx7").unwrap();
    let pubkey2 = Pubkey::from_str("A4iUVr5KjmsLymUcv4eSKPedUtoaBceiPeGipKMYc69b").unwrap();

    let mut wallet_account = Account {
      state: AccountState::Initialized,
      owners: BTreeMap::new(),
    };
    wallet_account.owners.insert(pubkey1, 1000);

    let mut recovery_keys = BTreeMap::new();
    recovery_keys.insert(pubkey2, 1000);
    assert_eq!(
      Processor::process_recovery(&mut wallet_account, recovery_keys),
      Ok(())
    );

    let mut expected_account = Account {
      state: AccountState::Initialized,
      owners: BTreeMap::<Pubkey, u16>::new(),
    };
    expected_account.owners.insert(pubkey2, 1000);

    assert_eq!(wallet_account, expected_account);
  }
}
