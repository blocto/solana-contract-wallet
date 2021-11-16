//! Program state processor

use crate::{
    error::WalletError,
    instruction::WalletInstruction,
    state::{Account, AccountState, InstructionBuffer, MIN_WEIGHT},
    utils::read_instruction,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::Instruction,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
};
use std::collections::BTreeMap;

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Process a Hello instruction
    fn process_hello() -> ProgramResult {
        msg!("Hello!");

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
        if wallet_account.owners.len() + owners.len() > wallet_account.max_owners {
            msg!("WalletError: too many owners");
            return Err(WalletError::InvalidInstruction.into());
        }

        for (pubkey, weight) in owners {
            if weight == 0 {
                msg!("WalletError: Key weight cannot be 0");
                return Err(WalletError::InvalidInstruction.into());
            }
            if wallet_account.owners.contains_key(&pubkey) {
                msg!("WalletError: Owner already exists");
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
            msg!("WalletError: Cannot find the target owner to remove");
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
        if owners.len() > wallet_account.max_owners {
            msg!("WalletError: too many owners");
            return Err(WalletError::InvalidInstruction.into());
        }

        // check key weight
        Self::is_key_weight_enough(&wallet_account.owners)?;

        wallet_account.owners.clear();

        for (pubkey, weight) in owners {
            if weight == 0 {
                msg!("WalletError: Key weight cannot be 0");
                return Err(WalletError::InvalidInstruction.into());
            }
            if wallet_account.owners.contains_key(&pubkey) {
                msg!("WalletError: Owner already exists");
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
        // msg!(bs58::encode(wallet_account.key.to_bytes()).into_string().as_str());
        pass_accounts.push(wallet_account.clone());
        // msg!(bs58::encode(auth_account.key.to_bytes()).into_string().as_str());
        pass_accounts.push(auth_account.clone());
        pass_accounts.push(payer_account.clone());

        for account in accounts_iter {
            // msg!(bs58::encode(account.key.to_bytes()).into_string().as_str());
            pass_accounts.push(account.clone());
        }

        // limit payer auth
        let mut instruction = instruction.clone();
        for account in &mut instruction.accounts {
            if &account.pubkey == payer_account.key {
                account.is_signer = false;
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
            msg!("WalletError: Signature weight too low");
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
            msg!("Wallet account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }

        Account::unpack_from_slice(&walllet_account.data.borrow())
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
            msg!("Wallet account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }

        // The account must be declaired writable
        if !account.is_writable {
            msg!("Wallet account was not declaired writable");
            return Err(ProgramError::InvalidAccountData);
        }

        Account::pack_into_slice(&wallet_account, &mut account.data.borrow_mut())?;

        Ok(())
    }

    fn process_init_instruction_buffer(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let instruction_account_info = next_account_info(accounts_iter)?;
        let owner_account_info = next_account_info(accounts_iter)?;
        let mut sequence_instructions =
            InstructionBuffer::unpack(&instruction_account_info.data.borrow())?;
        if sequence_instructions.owner != Pubkey::default() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        sequence_instructions.owner = *owner_account_info.key;

        InstructionBuffer::pack(
            sequence_instructions,
            &mut instruction_account_info.data.borrow_mut(),
        )?;

        Ok(())
    }

    fn process_append_partial_instruction(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        offset: u16,
        data: Vec<u8>,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let _wallet_account_info = next_account_info(accounts_iter)?;
        let instruction_buffer_account_info = next_account_info(accounts_iter)?;
        let owner_account_info = next_account_info(accounts_iter)?;

        if !owner_account_info.is_signer {
            msg!(&format!("{} should be a signer", owner_account_info.key));
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut instruction_buffer =
            InstructionBuffer::unpack(&instruction_buffer_account_info.data.borrow())?;
        if instruction_buffer.owner != *owner_account_info.key {
            msg!(&format!(
                "buffer account owner mismatch, want: {}, got: {}",
                instruction_buffer.owner, *owner_account_info.key
            ));
            return Err(ProgramError::InvalidAccountData);
        }

        instruction_buffer.data[offset as usize..offset as usize + data.len()]
            .copy_from_slice(&data[..]);

        InstructionBuffer::pack(
            instruction_buffer,
            &mut instruction_buffer_account_info.data.borrow_mut(),
        )?;

        Ok(())
    }

    fn process_run_insturction_buffer(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        expected_instruction_count: u16,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let wallet_account = next_account_info(accounts_iter)?;
        let instruction_buffer_account_info = next_account_info(accounts_iter)?;
        let owner_account_info = next_account_info(accounts_iter)?;

        if !owner_account_info.is_signer {
            msg!(&format!("{} should be a signer", owner_account_info.key));
            return Err(ProgramError::MissingRequiredSignature);
        }
        let instruction_buffer =
            InstructionBuffer::unpack(&instruction_buffer_account_info.data.borrow())?;
        if instruction_buffer.owner != *owner_account_info.key {
            msg!(&format!(
                "buffer account owner mismatch, want: {}, got: {}",
                instruction_buffer.owner, *owner_account_info.key
            ));
            return Err(ProgramError::InvalidAccountData);
        }

        // prepare account info
        let mut pass_accounts = Vec::new();
        for account in accounts_iter {
            let mut pass_account = account.clone();
            if pass_account.key == owner_account_info.key {
                pass_account.is_signer = false;
            }
            pass_accounts.push(pass_account);
        }

        // execute instructions
        let mut current = 0;
        let mut instruction_count = 0;
        while current < instruction_buffer.data.len() {
            let instruction = read_instruction(&mut current, &instruction_buffer.data[..])?;
            if instruction.program_id == Pubkey::default()
                && instruction.accounts.len() == 0
                && instruction.data.len() == 0
            {
                break;
            }
            invoke_signed(
                &instruction,
                &pass_accounts,
                &[&[&wallet_account.key.to_bytes()]],
            )?;
            instruction_count += 1;
        }

        // check instruction count
        if instruction_count != expected_instruction_count {
            msg!(&format!(
                "instruction count mismatch, want: {}, got: {}",
                expected_instruction_count, instruction_count
            ));
            return Err(ProgramError::InvalidAccountData);
        }

        // close buffer account
        let dest_starting_lamports = owner_account_info.lamports();
        **owner_account_info.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(instruction_buffer_account_info.lamports())
            .ok_or(ProgramError::InvalidAccountData)?;
        **instruction_buffer_account_info.lamports.borrow_mut() = 0;

        Ok(())
    }

    fn process_close_instruction_buffer(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let instruction_account_info = next_account_info(accounts_iter)?;
        let owner_account_info = next_account_info(accounts_iter)?;

        if !owner_account_info.is_signer {
            msg!(&format!("{} should be a signer", owner_account_info.key));
            return Err(ProgramError::MissingRequiredSignature);
        }

        let sequence_instructions =
            InstructionBuffer::unpack(&instruction_account_info.data.borrow())?;

        if sequence_instructions.owner != *owner_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let dest_starting_lamports = owner_account_info.lamports();
        **owner_account_info.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(instruction_account_info.lamports())
            .ok_or(ProgramError::InvalidAccountData)?;
        **instruction_account_info.lamports.borrow_mut() = 0;

        Ok(())
    }

    /// Process a WalletInstruction
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = WalletInstruction::unpack(input, &accounts)?;

        match instruction {
            WalletInstruction::AddOwner { owners } => {
                let mut wallet_account = Self::load_wallet_account(program_id, accounts)?;
                let is_wallet_initialized = wallet_account.is_initialized();

                // TODO add init instruction to handle it
                if !is_wallet_initialized {
                    msg!("Instruction: AddOwner (Initialize Wallet)");
                    Self::process_initialize_wallet(&mut wallet_account, owners)?;
                } else {
                    msg!("Instruction: AddOwner");
                    Self::check_signatures(accounts, &wallet_account)?;
                    Self::process_add_owner(&mut wallet_account, owners)?;
                }

                Self::store_wallet_account(program_id, accounts, wallet_account)
            }
            WalletInstruction::RemoveOwner { pubkey } => {
                msg!("Instruction: RemoveOwner");
                let mut wallet_account = Self::load_wallet_account(program_id, accounts)?;
                if !wallet_account.is_initialized() {
                    return Err(ProgramError::UninitializedAccount);
                }
                Self::check_signatures(accounts, &wallet_account)?;
                Self::process_remove_owner(&mut wallet_account, pubkey)?;

                Self::store_wallet_account(program_id, accounts, wallet_account)
            }
            WalletInstruction::Recovery { owners } => {
                msg!("Instruction: Recovery");
                let mut wallet_account = Self::load_wallet_account(program_id, accounts)?;
                if !wallet_account.is_initialized() {
                    return Err(ProgramError::UninitializedAccount);
                }
                Self::check_signatures(accounts, &wallet_account)?;
                Self::process_recovery(&mut wallet_account, owners)?;

                Self::store_wallet_account(program_id, accounts, wallet_account)
            }
            WalletInstruction::Revoke => {
                msg!("Instruction: Revoke");
                let mut wallet_account = Self::load_wallet_account(program_id, accounts)?;
                if !wallet_account.is_initialized() {
                    return Err(ProgramError::UninitializedAccount);
                }
                Self::check_signatures(accounts, &wallet_account)?;
                Self::process_revoke(&mut wallet_account)?;

                Self::store_wallet_account(program_id, accounts, wallet_account)
            }
            WalletInstruction::Invoke {
                instruction: internal_instruction,
            } => {
                msg!("Instruction: Invoke");
                let wallet_account = Self::load_wallet_account(program_id, accounts)?;
                if !wallet_account.is_initialized() {
                    return Err(ProgramError::UninitializedAccount);
                }
                Self::check_signatures(accounts, &wallet_account)?;
                Self::process_invoke(accounts, internal_instruction)
            }
            WalletInstruction::Hello => {
                msg!("Instruction: Hello");
                let wallet_account = Self::load_wallet_account(program_id, accounts)?;
                if !wallet_account.is_initialized() {
                    return Err(ProgramError::UninitializedAccount);
                }
                Self::check_signatures(accounts, &wallet_account)?;
                Self::process_hello()
            }
            WalletInstruction::InitInstructionBuffer => {
                msg!("Instruction: InitInstructionBuffer");
                Self::process_init_instruction_buffer(program_id, accounts)
            }
            WalletInstruction::AppendPartialInsturciton { offset, data } => {
                msg!("Instruction: AppendPartialInsturciton");
                let wallet_account = Self::load_wallet_account(program_id, accounts)?;
                if !wallet_account.is_initialized() {
                    return Err(ProgramError::UninitializedAccount);
                }
                Self::check_signatures(accounts, &wallet_account)?;
                Self::process_append_partial_instruction(program_id, accounts, offset, data)
            }
            WalletInstruction::RunInstructionBuffer {
                expected_instruction_count,
            } => {
                msg!("Instruction: RunInstructionBuffer");
                let wallet_account = Self::load_wallet_account(program_id, accounts)?;
                if !wallet_account.is_initialized() {
                    return Err(ProgramError::UninitializedAccount);
                }
                Self::process_run_insturction_buffer(
                    program_id,
                    accounts,
                    expected_instruction_count,
                )
            }
            WalletInstruction::CloseInstructionBuffer => {
                msg!("Instruction: CloseInstructionBuffer");
                Self::process_close_instruction_buffer(program_id, accounts)
            }
        }?;
        Ok(())
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
            max_owners: 101,
        };
        let init_keys = btreemap! {
          Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 1,
          Pubkey::from_str("65JQyZBU2RzNpP9vTdW5zSzujZR5JHZyChJsDWvkbM8u").unwrap() => 1,
        };
        let expected_account = Account {
            state: AccountState::Uninitialized,
            owners: BTreeMap::new(),
            max_owners: 101,
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
            max_owners: 101,
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
                max_owners: 101,
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
            max_owners: 101,
        };

        let add_keys = btreemap! {Pubkey::from_str("65JQyZBU2RzNpP9vTdW5zSzujZR5JHZyChJsDWvkbM8u").unwrap() => 1};
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
            max_owners: 101,
        };
        assert_eq!(init_account, expected_account);
    }

    #[test]
    fn should_fail_when_recovery_with_key_weight_is_not_enough() {
        let mut wallet_account = Account {
            state: AccountState::Initialized,
            owners: btreemap! {Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 1000},
            max_owners: 101,
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
            max_owners: 101,
        };
        let recovery_keys = btreemap! {Pubkey::from_str("65JQyZBU2RzNpP9vTdW5zSzujZR5JHZyChJsDWvkbM8u").unwrap() => 1000};
        assert_eq!(
            Processor::process_recovery(&mut wallet_account, recovery_keys.clone()),
            Ok(())
        );

        let expected_account = Account {
            state: AccountState::Initialized,
            owners: recovery_keys.clone(),
            max_owners: 101,
        };
        assert_eq!(wallet_account, expected_account);
    }

    #[test]
    fn process_revoke_should_success() {
        let mut wallet_account = Account {
            state: AccountState::Initialized,
            owners: btreemap! {Pubkey::from_str("EmPaWGCw48Sxu9Mu9pVrxe4XL2JeXUNTfoTXLuLz31gv").unwrap() => 1000},
            max_owners: 101,
        };
        assert_eq!(Processor::process_revoke(&mut wallet_account), Ok(()));

        let expected_account = Account {
            state: AccountState::Initialized,
            owners: btreemap! {},
            max_owners: 101,
        };
        assert_eq!(wallet_account, expected_account);
    }
}
