use solana_program_error::{ProgramError, ProgramResult};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use crate::{error::EbaltsError, state::load_mint_config_mut};



pub struct SetGatingProgram<'a> {
    pub authority: &'a AccountInfo<'a>,
    pub mint_config: &'a AccountInfo<'a>,
}

impl<'a> SetGatingProgram<'a> {
    pub const DISCRIMINATOR: u8 = 2;

    pub fn process(&self, remaining_data: &[u8]) -> ProgramResult {
        if remaining_data.len() != 32 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let new_gating_program = Pubkey::try_from(remaining_data).map_err(|_| ProgramError::InvalidInstructionData)?;

        let data = &mut self.mint_config.data.borrow_mut();
        let config = load_mint_config_mut(data)?;

        if config.freeze_authority != *self.authority.key {
            return Err(EbaltsError::InvalidAuthority.into());
        }
        
        config.gating_program = new_gating_program;

        Ok(())
    }

}

impl<'a> TryFrom<&'a [AccountInfo<'a>]> for SetGatingProgram<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'a>]) -> Result<Self, Self::Error> {
        let [authority, mint_config] = &accounts else {
            return Err(ProgramError::InvalidInstructionData);
        };

        if !authority.is_signer {
            return Err(EbaltsError::InvalidAuthority.into());
        }

        Ok(Self {
            authority,
            mint_config,
        })
    }
}