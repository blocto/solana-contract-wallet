use crate::{
    state::Account,
    state::Owner,
    instruction::WalletInstruction,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    info,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use std::mem;

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    _instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    info!("Solana multisig wallet Rust program entrypoint");

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

    info!("FUCK1");
    // Increment and store the number of times the account has been greeted
    let mut stored_account = Account::unpack_unchecked(&account.data.borrow())?;
    // let mut num_greets = LittleEndian::read_u32(&data);
    // num_greets += 1;
    stored_account.owners[0] = Owner {
      pubkey: *account.owner,
      weight: 1000_u16,
    };

    stored_account.owners[1] = Owner {
      pubkey: *account.key,
      weight: 500_u16,
    };

    Account::pack(stored_account, &mut account.data.borrow_mut())?;

    info!("Hello!");

    Ok(())
}
