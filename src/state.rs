//! State transition types
use crate::utils::{write_pubkey, write_u16};
use num_enum::TryFromPrimitive;
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Sealed},
    pubkey::Pubkey,
    serialize_utils::{read_pubkey, read_u16, read_u8},
};
use std::collections::BTreeMap;

/// Maximum signature weight for instructions
pub const MIN_WEIGHT: u16 = 1000;

/// Account data.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Account {
    /// The account's state
    pub state: AccountState,
    /// owners is a map (public key => weight)
    pub owners: BTreeMap<Pubkey, u16>,
    /// only use in program, not pack into account
    pub max_owners: usize,
}

impl Account {
    /*
        Account Len = state   + (pubkey_key + key_weight) * MAX_OWNERS
                    =    1    + (    32     +      2    ) * MAX_OWNERS
    */

    /// give data and parse it as an account
    pub fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() == 0 || (src.len() - 1) % 34 != 0 {
            msg!(&format!("check account length falied, len: {}", src.len()));
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
            state: AccountState::try_from_primitive(state)
                .or(Err(ProgramError::InvalidAccountData))?,
            owners: owners,
            max_owners: (src.len() - 1) / 34,
        })
    }

    /// store current account to a given data slice
    pub fn pack_into_slice(&self, dst: &mut [u8]) -> Result<(), ProgramError> {
        // reset all byte to 0
        for i in dst.iter_mut() {
            *i = 0;
        }

        let mut current = 0;
        dst[current] = (self.state as u8).into();
        current += 1;
        for (pubkey, weight) in &self.owners {
            // pubkey
            write_pubkey(&mut current, pubkey, dst)?;
            // key weight
            write_u16(&mut current, *weight, dst)?;
        }

        Ok(())
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
            max_owners: 101,
        };
        account.owners.insert(pubkey1, 999);
        account.owners.insert(pubkey2, 1);

        let mut dst = vec![0x00; 3435];

        assert_eq!(account.pack_into_slice(&mut dst), Ok(()));

        let unpack_account = Account::unpack_from_slice(&dst).unwrap();

        assert_eq!(account, unpack_account);
    }

    #[test]
    fn test_account_pack_into_exist_data() {
        let mut account_dst1 = vec![0x00; 3435];
        let mut account_dst2 = vec![0x00; 3435];

        // create a init account
        let mut account = Account {
            state: AccountState::Initialized,
            owners: btreemap! {
              Pubkey::from_str("A4iUVr5KjmsLymUcv4eSKPedUtoaBceiPeGipKMYc69b").unwrap() => 1000,
              Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 1000,
            },
            max_owners: 101,
        };
        assert_eq!(account.pack_into_slice(&mut account_dst1), Ok(()));

        // remove owner and pack into origin destination
        account
            .owners
            .remove(&Pubkey::from_str("A4iUVr5KjmsLymUcv4eSKPedUtoaBceiPeGipKMYc69b").unwrap());
        assert_eq!(account.pack_into_slice(&mut account_dst1), Ok(()));

        // pack into another destination
        assert_eq!(account.pack_into_slice(&mut account_dst2), Ok(()));

        // compare
        assert_eq!(account_dst1, account_dst2)
    }
}

/// InstructionBuffer
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InstructionBuffer {
    /// instruction buffer owner
    pub owner: Pubkey,

    /// data
    pub data: Vec<u8>,
}

impl InstructionBuffer {
    /// Unpack from slice
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let mut current = 0;

        // parse owner
        let owner = read_pubkey(&mut current, input).unwrap();

        // parse data
        let data: Vec<u8> = input[current..].iter().cloned().collect();

        Ok(InstructionBuffer { owner, data })
    }

    /// Pack into slice
    pub fn pack(src: Self, dst: &mut [u8]) -> Result<(), ProgramError> {
        // reset all byte to 0
        for i in dst.iter_mut() {
            *i = 0;
        }

        let mut current = 0;

        // write owner
        write_pubkey(&mut current, &src.owner, dst)?;

        // write data
        dst[current..current + src.data.len()].clone_from_slice(&src.data);

        Ok(())
    }
}
