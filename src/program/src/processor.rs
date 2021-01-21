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
    // check key weight
    Self::is_key_weight_enough(&owners)?;

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

    // check key weight
    Self::is_key_weight_enough(&wallet_account.owners)?;

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

    // check key weight
    Self::is_key_weight_enough(&wallet_account.owners)?;

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

  /// Process an Revoke insturction
  fn process_revoke(wallet_account: &mut Account) -> ProgramResult {
    wallet_account.owners.clear();
    Ok(())
  }

  /// Process an Invoke instruction and call another program
  fn process_invoke(accounts: &[AccountInfo], instruction: Instruction) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let wallet_account = next_account_info(accounts_iter)?;
    let auth_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;

    let mut pass_accounts = Vec::new();

    // Pass all accounts to invoke call
    // info!(bs58::encode(wallet_account.key.to_bytes()).into_string().as_str());
    pass_accounts.push(wallet_account.clone());
    // info!(bs58::encode(auth_account.key.to_bytes()).into_string().as_str());
    pass_accounts.push(auth_account.clone());
    pass_accounts.push(payer_account.clone());

    for account in accounts_iter {
      // info!(bs58::encode(account.key.to_bytes()).into_string().as_str());
      pass_accounts.push(account.clone());
    }

    // limit payer auth
    let mut instruction = instruction.clone();
    for account in &mut instruction.accounts {
      if &account.pubkey == payer_account.key {
        account.is_signer = false;
        account.is_writable = false;
      }
    }

    invoke_signed(
      &instruction,
      pass_accounts.as_slice(),
      &[&[&wallet_account.key.to_bytes()]],
    )?;

    Ok(())
  }

  fn is_key_weight_enough(owners: &BTreeMap<Pubkey, u16>) -> ProgramResult {
    let mut sum_of_key_weight = 0;
    for (_, weight) in owners {
      sum_of_key_weight += weight;
    }
    if sum_of_key_weight < MIN_WEIGHT {
      return Err(WalletError::InsufficientWeight.into());
    }
    Ok(())
  }

  /// Check if signatures have enought weight
  fn check_signatures(accounts: &[AccountInfo], wallet_account: &Account) -> ProgramResult {
    let mut total_key_weight = 0;
    let mut counted = BTreeMap::new();

    for account in accounts.iter() {
      if account.is_signer
        && wallet_account.owners.contains_key(account.key)
        && !counted.contains_key(account.key)
      {
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
      WalletInstruction::Revoke if is_wallet_initialized => {
        Self::process_revoke(&mut wallet_account)
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
  use maplit::btreemap;
  use std::str::FromStr;

  #[test]
  fn should_fail_when_init_with_key_weight_is_not_enough() {
    let mut init_account = Account {
      state: AccountState::Uninitialized,
      owners: BTreeMap::new(),
    };
    let init_keys = btreemap! {
      Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 1,
      Pubkey::from_str("65JQyZBU2RzNpP9vTdW5zSzujZR5JHZyChJsDWvkbM8u").unwrap() => 1,
    };
    let expected_account = Account {
      state: AccountState::Uninitialized,
      owners: BTreeMap::new(),
    };

    assert_eq!(
      Processor::process_initialize_wallet(&mut init_account, init_keys.clone()),
      Err(WalletError::InsufficientWeight.into()),
    );
    assert_eq!(init_account, expected_account);
  }

  #[test]
  fn process_initialize_wallet_should_success() {
    let mut init_account = Account {
      state: AccountState::Uninitialized,
      owners: BTreeMap::new(),
    };
    let init_keys = btreemap! {
      Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 999,
      Pubkey::from_str("65JQyZBU2RzNpP9vTdW5zSzujZR5JHZyChJsDWvkbM8u").unwrap() => 1,
    };

    assert_eq!(
      Processor::process_initialize_wallet(&mut init_account, init_keys.clone()),
      Ok(()),
    );
    assert_eq!(
      init_account,
      Account {
        state: AccountState::Initialized,
        owners: init_keys.clone(),
      },
    );
  }

  #[test]
  fn process_add_owner_should_success() {
    let mut init_account = Account {
      state: AccountState::Initialized,
      owners: btreemap! {
        Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 1000,
      },
    };

    let add_keys =
      btreemap! {Pubkey::from_str("65JQyZBU2RzNpP9vTdW5zSzujZR5JHZyChJsDWvkbM8u").unwrap() => 1};
    assert_eq!(
      Processor::process_add_owner(&mut init_account, add_keys),
      Ok(())
    );

    let expected_account = Account {
      state: AccountState::Initialized,
      owners: btreemap! {
        Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 1000,
        Pubkey::from_str("65JQyZBU2RzNpP9vTdW5zSzujZR5JHZyChJsDWvkbM8u").unwrap() => 1
      },
    };
    assert_eq!(init_account, expected_account);
  }

  #[test]
  fn should_fail_when_recovery_with_key_weight_is_not_enough() {
    let mut wallet_account = Account {
      state: AccountState::Initialized,
      owners: btreemap! {Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 1000},
    };
    let recovery_keys = btreemap! {
      Pubkey::from_str("65JQyZBU2RzNpP9vTdW5zSzujZR5JHZyChJsDWvkbM8u").unwrap() => 1,
    };
    assert_eq!(
      Processor::process_initialize_wallet(&mut wallet_account, recovery_keys),
      Err(WalletError::InsufficientWeight.into()),
    );
  }

  #[test]
  fn process_recovery_should_success() {
    let mut wallet_account = Account {
      state: AccountState::Initialized,
      owners: btreemap! {Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 1000},
    };
    let recovery_keys = btreemap! {Pubkey::from_str("65JQyZBU2RzNpP9vTdW5zSzujZR5JHZyChJsDWvkbM8u").unwrap() => 1000};
    assert_eq!(
      Processor::process_recovery(&mut wallet_account, recovery_keys.clone()),
      Ok(())
    );

    let expected_account = Account {
      state: AccountState::Initialized,
      owners: recovery_keys.clone(),
    };
    assert_eq!(wallet_account, expected_account);
  }

  #[test]
  fn process_revoke_should_success() {
    let mut wallet_account = Account {
      state: AccountState::Initialized,
      owners: btreemap! {Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 1000},
    };
    assert_eq!(
      Processor::process_revoke(&mut wallet_account),
      Ok(())
    );

    let expected_account = Account {
      state: AccountState::Initialized,
      owners: btreemap!{},
    };
    assert_eq!(wallet_account, expected_account);
  }
}
