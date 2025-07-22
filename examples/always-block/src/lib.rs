use ebalts_interface::instruction::{
    CanFreezePermissionlessInstruction, CanThawPermissionlessInstruction,
};
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};
use solana_program_error::ProgramError;
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};

pub mod instructions;
pub use instructions::*;

declare_id!("Eba1ts11111111111111111111111111111111111113");

entrypoint!(process_instruction);
fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &'a [u8],
) -> ProgramResult {
    let (discriminator, _remaining) = instruction_data.split_at(ArrayDiscriminator::LENGTH);

    match discriminator {
        InitializeExtraMetas::DISCRIMINATOR_SLICE => {
            InitializeExtraMetas::try_from(accounts)?.process()
        }
        CanThawPermissionlessInstruction::SPL_DISCRIMINATOR_SLICE => {
            Err(CustomErrors::UnsupportedInstruction.into())
        }
        CanFreezePermissionlessInstruction::SPL_DISCRIMINATOR_SLICE => {
            Err(CustomErrors::UnsupportedInstruction.into())
        }
        _ => Err(CustomErrors::InvalidInstruction.into()),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CustomErrors {
    InvalidInstruction,
    UnsupportedInstruction = 999999999,
}

impl From<CustomErrors> for ProgramError {
    fn from(e: CustomErrors) -> Self {
        ProgramError::Custom(e as u32)
    }
}
