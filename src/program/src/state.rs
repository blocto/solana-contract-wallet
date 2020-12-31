//! State transition types
use arrayref::array_mut_ref;
use solana_program::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
  serialize_utils::{read_pubkey, read_u16},
};

use std::{collections::BTreeMap};

/// Maximum signature weight for instructions
pub const MIN_WEIGHT: u16 = 1000;

/// Maximum number of multisignature owners
pub const MAX_OWNERS: usize = 11;

/// Account data.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Account {
  /// owners is a map (public key => weight)
  pub owners: BTreeMap<Pubkey, u16>,
}

impl Pack for Account {
  const LEN: usize = 374;

  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    if src.len() % 34 != 0 {
      return Err(ProgramError::InvalidAccountData);
    }

    let mut owners = BTreeMap::new();
    let mut current = 0;
    while current < src.len() {
      let pubkey = read_pubkey(&mut current, src).unwrap();
      let weight = read_u16(&mut current, src).unwrap();
      if weight == 0 {
        break
      }
      owners.insert(pubkey, weight);
    }
    Ok(Account { owners: owners })
  }

  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, Account::LEN];

    let mut i = 0;
    for (pubkey, weight) in &self.owners {
      let start = 34 * i;
      dst[start..start + 32].copy_from_slice(pubkey.as_ref());

      let start = start + 32;
      dst[start..start + 2].copy_from_slice(&weight.to_le_bytes());

      i += 1
    }
  }
}

impl Sealed for Account {}

impl IsInitialized for Account {
  fn is_initialized(&self) -> bool {
    self.owners.len() > 0
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::state::Account;
  // use solana_program::pubkey::Pubkey;
  use std::str::FromStr;

  #[test]
  fn test_account_pack() {
    let pubkey1 = Pubkey::from_str("EvN4kgKmCmYzdbd5kL8Q8YgkUW5RoqMTpBczrfLExtx7").unwrap();
    let pubkey2 = Pubkey::from_str("A4iUVr5KjmsLymUcv4eSKPedUtoaBceiPeGipKMYc69b").unwrap();

    let mut account = Account {
      owners: BTreeMap::<Pubkey, u16>::new(),
    };
    account.owners.insert(pubkey1, 999);
    account.owners.insert(pubkey2, 1);

    let mut dst = vec![0x00; Account::LEN];
    account.pack_into_slice(&mut dst);

    let unpack_account = Account::unpack_from_slice(&dst).unwrap();

    assert_eq!(account, unpack_account);
  }
}
