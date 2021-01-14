//! State transition types
use arrayref::{array_mut_ref, mut_array_refs};
use num_enum::TryFromPrimitive;
use solana_program::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
  serialize_utils::{read_pubkey, read_u16, read_u8},
};

use std::collections::BTreeMap;

/// Maximum signature weight for instructions
pub const MIN_WEIGHT: u16 = 1000;

/// Maximum number of multisignature owners
pub const MAX_OWNERS: usize = 101;

/// Account data.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Account {
  /// The account's state
  pub state: AccountState,
  /// owners is a map (public key => weight)
  pub owners: BTreeMap<Pubkey, u16>,
}

impl Pack for Account {
  /*
    is_init = 1 byte
    (public key + key weight) * MAX_OWNERS = (32 + 2) * 101 = 3434
    ---
    total: 375
  */
  const LEN: usize = 3435;

  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    if src.len() == 0 || (src.len() - 1) % 34 != 0 {
      return Err(ProgramError::InvalidAccountData);
    }

    let mut current = 0;
    let state = read_u8(&mut current, src).unwrap();

    let mut owners = BTreeMap::new();
    while current < src.len() {
      let pubkey = read_pubkey(&mut current, src).unwrap();
      let weight = read_u16(&mut current, src).unwrap();
      if weight == 0 {
        break;
      }
      owners.insert(pubkey, weight);
    }
    Ok(Account {
      state: AccountState::try_from_primitive(state).or(Err(ProgramError::InvalidAccountData))?,
      owners: owners,
    })
  }

  fn pack_into_slice(&self, dst: &mut [u8]) {
    // reset all byte to 0
    for i in dst.iter_mut() {
      *i = 0;
    }

    let dst = array_mut_ref![dst, 0, Account::LEN];

    let (is_init, key_and_weight) = mut_array_refs![dst, 1; ..;];

    is_init.copy_from_slice(&(self.state as u8).to_le_bytes());

    let mut start = 0;
    for (pubkey, weight) in &self.owners {
      key_and_weight[start..start + 32].copy_from_slice(pubkey.as_ref());
      start += 32;

      key_and_weight[start..start + 2].copy_from_slice(&weight.to_le_bytes());
      start += 2;
    }
  }
}

impl Sealed for Account {}

impl IsInitialized for Account {
  fn is_initialized(&self) -> bool {
    self.state != AccountState::Uninitialized
  }
}

/// Account state.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, TryFromPrimitive)]
pub enum AccountState {
  /// Account is not yet initialized
  Uninitialized,
  /// Account is initialized; the account owner and/or delegate may perform permitted operations
  /// on this account
  Initialized,
}

impl Default for AccountState {
  fn default() -> Self {
    AccountState::Uninitialized
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::state::Account;
  use maplit::btreemap;
  use std::str::FromStr;

  #[test]
  fn test_account_pack() {
    let pubkey1 = Pubkey::from_str("EvN4kgKmCmYzdbd5kL8Q8YgkUW5RoqMTpBczrfLExtx7").unwrap();
    let pubkey2 = Pubkey::from_str("A4iUVr5KjmsLymUcv4eSKPedUtoaBceiPeGipKMYc69b").unwrap();

    let mut account = Account {
      state: AccountState::Initialized,
      owners: BTreeMap::<Pubkey, u16>::new(),
    };
    account.owners.insert(pubkey1, 999);
    account.owners.insert(pubkey2, 1);

    let mut dst = vec![0x00; Account::LEN];
    account.pack_into_slice(&mut dst);

    let unpack_account = Account::unpack_from_slice(&dst).unwrap();

    assert_eq!(account, unpack_account);
  }

  #[test]
  fn test_account_pack_into_exist_data() {
    let mut account_dst1 = vec![0x00; Account::LEN];
    let mut account_dst2 = vec![0x00; Account::LEN];

    // create a init account
    let mut account = Account {
      state: AccountState::Initialized,
      owners: btreemap! {
        Pubkey::from_str("A4iUVr5KjmsLymUcv4eSKPedUtoaBceiPeGipKMYc69b").unwrap() => 1000,
        Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 1000,
      },
    };
    account.pack_into_slice(&mut account_dst1);

    // remove owner and pack into origin destination
    account.owners.remove(&Pubkey::from_str("A4iUVr5KjmsLymUcv4eSKPedUtoaBceiPeGipKMYc69b").unwrap());
    account.pack_into_slice(&mut account_dst1);

    // pack into another destination
    account.pack_into_slice(&mut account_dst2);

    // compare
    assert_eq!(account_dst1, account_dst2)
  }
}
