use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use num_enum::TryFromPrimitive;
use solana_program::{
  pubkey::Pubkey,
  program_error::ProgramError,
  program_pack::{Pack, Sealed},
};

/// Maximum number of multisignature owners
pub const MAX_OWNERS: usize = 11;
pub const OWNER_SIZE: usize = 32 + 2;
pub const ACCOUNT_SIZE: usize = MAX_OWNERS * OWNER_SIZE + 1 + 1; // with alignment

/// Account data.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Account {
  /// The account's state
  pub state: AccountState,

  /// The owners of this account.
  pub owners: [Owner; MAX_OWNERS],
}

impl Pack for Account {
  const LEN: usize = ACCOUNT_SIZE;
  
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, ACCOUNT_SIZE];
    let (state, owners_flat) =
        array_refs![src, 2, MAX_OWNERS * OWNER_SIZE];
    let mut result = Account {
      state: AccountState::try_from_primitive(state[0])
        .or(Err(ProgramError::InvalidAccountData))?,
      owners: [Owner::unpack_from_slice(&[0u8; 34])?; MAX_OWNERS],
    };

    for (src, dst) in owners_flat.chunks(34).zip(result.owners.iter_mut()) {
      *dst = Owner::unpack_from_slice(src)?;
    }

    Ok(result)
  }

  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, ACCOUNT_SIZE];
    let (state_dst, owner_flat) = mut_array_refs![dst, 2, MAX_OWNERS * OWNER_SIZE];
    let &Account {
        ref owners,
        state,
    } = self;
    state_dst[0] = state as u8;
    for (i, src) in owners.iter().enumerate() {
      let dst_array = array_mut_ref![owner_flat, 34 * i, 34];
      src.pack_into_slice(dst_array);
    }
  }
}
impl Sealed for Account {}

/// Account data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Owner {
  /// The public key of the owner.
  pub pubkey: Pubkey,

  /// The weight of the owner.
  pub weight: u16,
}

impl Pack for Owner {
  const LEN: usize = 34;

  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 34];
    let (pubkey, weight) =
        array_refs![src, 32, 2];
    Ok(Owner {
      pubkey: Pubkey::new_from_array(*pubkey),
      weight: u16::from_le_bytes(*weight),
    })
  }

  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 34];
    let (
        pubkey_dst,
        weight_dst,
    ) = mut_array_refs![dst, 32, 2];
    let &Owner {
        ref pubkey,
        weight,
    } = self;
    pubkey_dst.copy_from_slice(pubkey.as_ref());
    *weight_dst = weight.to_le_bytes();
  }
}

impl Sealed for Owner {}

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

