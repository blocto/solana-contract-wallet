use serde::Serialize;
use solana_program::{
    pubkey::Pubkey,
    instruction::Instruction
};

#[repr(C)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Command {
    /// Add a Pubkey to owner list
    AddOwner {
        pubkey: Pubkey,
        weight: u16,
    },
    /// Remove a Pubkey from owner list
    RemoveOwner {
        pubkey: Pubkey
    },
    /// Invoke an instruction to another program
    Invoke {
        instruction: Instruction
    },
    /// Say hello
    Hello,
}
