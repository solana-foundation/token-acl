use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};



pub enum EfficientBlockAllowInstruction {
    CanThawPermissionless,
    CanFreezePermissionless,
}


#[derive(SplDiscriminate)]
#[discriminator_hash_input("efficient-allow-block-list-standard:can-thaw-permissionless")]
pub struct CanThawPermissionlessInstruction;

#[derive(SplDiscriminate)]
#[discriminator_hash_input("efficient-allow-block-list-standard:can-freeze-permissionless")]
pub struct CanFreezePermissionlessInstruction;


impl EfficientBlockAllowInstruction {
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < ArrayDiscriminator::LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }
        let (discriminator, _) = data.split_at(ArrayDiscriminator::LENGTH);
        match discriminator {
            CanThawPermissionlessInstruction::SPL_DISCRIMINATOR_SLICE => Ok(Self::CanThawPermissionless),
            CanFreezePermissionlessInstruction::SPL_DISCRIMINATOR_SLICE => Ok(Self::CanFreezePermissionless),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        match self {
            Self::CanThawPermissionless => CanThawPermissionlessInstruction::SPL_DISCRIMINATOR_SLICE.to_vec(),
            Self::CanFreezePermissionless => CanFreezePermissionlessInstruction::SPL_DISCRIMINATOR_SLICE.to_vec(),
        }
    }
}




pub fn can_thaw_permissionless(
    program_id: &Pubkey,
    signer: &Pubkey,
    token_account: &Pubkey,
    mint: &Pubkey,
) -> Instruction {
    let data = EfficientBlockAllowInstruction::CanThawPermissionless.pack();
    let accounts = vec![
        AccountMeta::new_readonly(*signer, false),
        AccountMeta::new_readonly(*token_account, false),
        AccountMeta::new_readonly(*mint, false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data
    }
}

pub fn can_freeze_permissionless(
    program_id: &Pubkey,
    signer: &Pubkey,
    token_account: &Pubkey,
    mint: &Pubkey,
) -> Instruction {
    let data = EfficientBlockAllowInstruction::CanFreezePermissionless.pack();
    let accounts = vec![
        AccountMeta::new_readonly(*signer, false),
        AccountMeta::new_readonly(*token_account, false),
        AccountMeta::new_readonly(*mint, false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data
    }
}
