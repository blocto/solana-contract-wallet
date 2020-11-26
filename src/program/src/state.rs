use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use num_enum::TryFromPrimitive;
use solana_program::{
  pubkey::Pubkey,
  program_error::ProgramError,
  program_pack::{Pack, Sealed},
};

/// Account data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Account {
    /// The owner of this account.
    pub owner: Pubkey,
    /// The account's state
    pub state: AccountState,
}

impl Pack for Account {
  const LEN: usize = 33;
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
      let src = array_ref![src, 0, 33];
      let (owner, state) =
          array_refs![src, 32, 1];
      Ok(Account {
          owner: Pubkey::new_from_array(*owner),
          state: AccountState::try_from_primitive(state[0])
              .or(Err(ProgramError::InvalidAccountData))?,
      })
  }
  fn pack_into_slice(&self, dst: &mut [u8]) {
      let dst = array_mut_ref![dst, 0, 33];
      let (
          owner_dst,
          state_dst,
      ) = mut_array_refs![dst, 32, 1];
      let &Account {
          ref owner,
          state,
      } = self;
      owner_dst.copy_from_slice(owner.as_ref());
      state_dst[0] = state as u8;
  }
}
impl Sealed for Account {}

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

