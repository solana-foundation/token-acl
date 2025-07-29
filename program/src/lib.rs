use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};
use solana_program_error::ProgramError;

use crate::instructions::{
    CreateConfig, ForfeitFreezeAuthority, Freeze, FreezePermissionless, SetAuthority,
    SetGatingProgram, Thaw, ThawPermissionless, TogglePermissionlessInstructions,
};

pub mod error;
pub mod instruction;
pub mod instructions;
pub mod offchain;
pub mod state;

declare_id!("Eba1ts11111111111111111111111111111111111111");

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);
fn process_instruction<'a>(
    _program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &'a [u8],
) -> ProgramResult {
    let [discriminator, remaining_data @ ..] = instruction_data else {
        println!("Invalid instruction data: {:?}", instruction_data);
        return Err(ProgramError::InvalidInstructionData);
    };

    match *discriminator {
        CreateConfig::DISCRIMINATOR => CreateConfig::try_from(accounts)?.process(remaining_data),
        Freeze::DISCRIMINATOR => Freeze::try_from(accounts)?.process(),
        Thaw::DISCRIMINATOR => Thaw::try_from(accounts)?.process(),
        ThawPermissionless::DISCRIMINATOR => ThawPermissionless::try_from(accounts)?.process(),
        FreezePermissionless::DISCRIMINATOR => FreezePermissionless::try_from(accounts)?.process(),
        SetAuthority::DISCRIMINATOR => SetAuthority::try_from(accounts)?.process(remaining_data),
        SetGatingProgram::DISCRIMINATOR => {
            SetGatingProgram::try_from(accounts)?.process(remaining_data)
        }
        ForfeitFreezeAuthority::DISCRIMINATOR => {
            ForfeitFreezeAuthority::try_from(accounts)?.process(remaining_data)
        }
        TogglePermissionlessInstructions::DISCRIMINATOR => {
            TogglePermissionlessInstructions::try_from(accounts)?.process(remaining_data)
        }
        _ => {
            println!("Invalid instruction discriminator: {:?}", discriminator);
            Err(ProgramError::InvalidInstructionData)
        }
    }
}
