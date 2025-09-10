use solana_program::account_info::AccountInfo;
use solana_program_error::{ProgramError, ProgramResult};
use spl_pod::primitives::PodBool;

use crate::{error::TokenAclError, state::load_mint_config_mut};

pub struct TogglePermissionlessInstructions<'a> {
    pub authority: &'a AccountInfo<'a>,
    pub mint_config: &'a AccountInfo<'a>,
}

impl TogglePermissionlessInstructions<'_> {
    pub const DISCRIMINATOR: u8 = 8;

    pub fn process(&self, remaining_data: &[u8]) -> ProgramResult {
        let [freeze_enabled, thaw_enabled] = remaining_data else {
            return Err(ProgramError::InvalidInstructionData);
        };

        let data = &mut self.mint_config.data.borrow_mut();
        let config = load_mint_config_mut(data)?;

        if config.freeze_authority != *self.authority.key {
            return Err(TokenAclError::InvalidAuthority.into());
        }

        config.enable_permissionless_freeze = PodBool::from_bool(*freeze_enabled != 0);
        config.enable_permissionless_thaw = PodBool::from_bool(*thaw_enabled != 0);

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo<'a>]> for TogglePermissionlessInstructions<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'a>]) -> Result<Self, Self::Error> {
        let [authority, mint_config] = &accounts else {
            return Err(ProgramError::InvalidInstructionData);
        };

        if !authority.is_signer {
            return Err(TokenAclError::InvalidAuthority.into());
        }

        Ok(Self {
            authority,
            mint_config,
        })
    }
}
