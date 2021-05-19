//! utils
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sanitize::SanitizeError,
    serialize_utils::{read_pubkey, read_u16, read_u8},
};

/// read a bool
pub fn read_bool(current: &mut usize, data: &[u8]) -> Result<bool, SanitizeError> {
    if data.len() < *current + 1 {
        return Err(SanitizeError::IndexOutOfBounds);
    }
    let e = {
        match data[*current] {
            0 => false,
            1 => true,
            _ => return Err(SanitizeError::InvalidValue),
        }
    };
    *current += 1;
    Ok(e)
}

/// write a bool
pub fn write_bool(current: &mut usize, b: bool, dst: &mut [u8]) -> Result<(), ProgramError> {
    if dst.len() < *current + 1 {
        return Err(ProgramError::InvalidAccountData);
    }
    dst[*current] = b.into();
    *current += 1;
    Ok(())
}

/// write a u16
pub fn write_u16(current: &mut usize, src: u16, dst: &mut [u8]) -> Result<(), ProgramError> {
    if dst.len() < *current + 2 {
        return Err(ProgramError::InvalidAccountData);
    }
    dst[*current..*current + 2].copy_from_slice(&src.to_le_bytes());
    *current += 2;
    Ok(())
}

/// write a pubkey
pub fn write_pubkey(
    current: &mut usize,
    pubkey: &Pubkey,
    dst: &mut [u8],
) -> Result<(), ProgramError> {
    if dst.len() < *current + 32 {
        return Err(ProgramError::InvalidAccountData);
    }
    dst[*current..*current + 32].copy_from_slice(pubkey.as_ref());
    *current += 32;
    Ok(())
}

/// read an instruction
pub fn read_instruction(current: &mut usize, input: &[u8]) -> Result<Instruction, ProgramError> {
    let account_len = usize::from(read_u16(current, &input).unwrap());
    let mut accounts = Vec::new();
    for _ in 0..account_len {
        let account_metadata = read_u8(current, &input).unwrap();
        let account_pubkey = read_pubkey(current, &input).unwrap();

        let account_meta = AccountMeta {
            pubkey: account_pubkey,
            is_signer: account_metadata & (1 << 0) != 0,
            is_writable: account_metadata & (1 << 1) != 0,
        };
        accounts.push(account_meta);
    }

    let program_id = read_pubkey(current, input).unwrap();

    let data_len = usize::from(read_u16(current, &input).unwrap());
    let data = input[*current..*current + data_len].to_vec();
    *current += data_len;

    Ok(Instruction {
        program_id: program_id,
        accounts: accounts,
        data: data,
    })
}

/// write instruction
pub fn write_instruction(
    current: &mut usize,
    instruction: &Instruction,
    dst: &mut [u8],
) -> Result<(), ProgramError> {
    dst[*current..*current + 2].copy_from_slice(&(instruction.accounts.len() as u16).to_le_bytes());
    *current += 2;

    for account_meta in instruction.accounts.iter() {
        let mut meta_byte = 0;
        if account_meta.is_signer {
            meta_byte |= 1 << 0;
        }
        if account_meta.is_writable {
            meta_byte |= 1 << 1;
        }
        dst[*current] = meta_byte;
        *current += 1;

        dst[*current..*current + 32].copy_from_slice(account_meta.pubkey.as_ref());
        *current += 32;
    }

    dst[*current..*current + 32].copy_from_slice(instruction.program_id.as_ref());
    *current += 32;

    let data_len = instruction.data.len();
    dst[*current..*current + 2].copy_from_slice(&(data_len as u16).to_le_bytes());
    *current += 2;

    dst[*current..*current + data_len].copy_from_slice(instruction.data.as_ref());
    *current += data_len;

    Ok(())
}
