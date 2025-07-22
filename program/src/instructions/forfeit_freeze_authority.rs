use solana_cpi::invoke_signed;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use solana_program_error::{ProgramError, ProgramResult};
use spl_token_2022::instruction::AuthorityType;

use crate::{
    error::EbaltsError,
    state::{load_mint_config, MintConfig},
};

pub struct ForfeitFreezeAuthority<'a> {
    pub authority: &'a AccountInfo<'a>,
    pub mint: &'a AccountInfo<'a>,
    pub mint_config: &'a AccountInfo<'a>,
    pub token_program: &'a AccountInfo<'a>,
}

impl ForfeitFreezeAuthority<'_> {
    pub const DISCRIMINATOR: u8 = 3;

    pub fn process(&self, remaining_data: &[u8]) -> ProgramResult {
        if remaining_data.len() != 32 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let new_freeze_authority =
            Pubkey::try_from(remaining_data).map_err(|_| ProgramError::InvalidInstructionData)?;

        let data = &mut self.mint_config.data.borrow_mut();
        let config = load_mint_config(data)?;

        if config.freeze_authority != *self.authority.key {
            return Err(EbaltsError::InvalidAuthority.into());
        }

        if config.mint != *self.mint.key {
            return Err(EbaltsError::InvalidTokenMint.into());
        }

        let bump_seed = [config.bump];
        let seeds = [MintConfig::SEED_PREFIX, self.mint.key.as_ref(), &bump_seed];

        let ix = spl_token_2022::instruction::set_authority(
            self.token_program.key,
            self.mint.key,
            Some(&new_freeze_authority),
            AuthorityType::FreezeAccount,
            self.mint_config.key,
            &[],
        )?;
        invoke_signed(&ix, &[self.mint.clone(), self.authority.clone()], &[&seeds])?;

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo<'a>]> for ForfeitFreezeAuthority<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'a>]) -> Result<Self, Self::Error> {
        let [authority, mint, mint_config, token_program] = &accounts else {
            return Err(ProgramError::InvalidInstructionData);
        };

        if !authority.is_signer {
            return Err(EbaltsError::InvalidAuthority.into());
        }

        if !spl_token_2022::check_id(token_program.key) {
            return Err(EbaltsError::InvalidTokenProgram.into());
        }

        Ok(Self {
            authority,
            mint,
            mint_config,
            token_program,
        })
    }
}
