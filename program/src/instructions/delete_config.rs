use solana_cpi::invoke_signed;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use solana_program_error::{ProgramError, ProgramResult};
use spl_token_2022::instruction::AuthorityType;

use crate::{
    error::TokenAclError,
    state::{load_mint_config, MintConfig},
};

pub struct DeleteConfig<'a> {
    pub authority: &'a AccountInfo<'a>,
    pub receiver: &'a AccountInfo<'a>,
    pub mint: &'a AccountInfo<'a>,
    pub mint_config: &'a AccountInfo<'a>,
    pub token_program: &'a AccountInfo<'a>,
}

impl DeleteConfig<'_> {
    pub const DISCRIMINATOR: u8 = 3;

    pub fn process(&self, remaining_data: &[u8]) -> ProgramResult {
        if remaining_data.len() != 32 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let new_freeze_authority =
            Pubkey::try_from(remaining_data).map_err(|_| ProgramError::InvalidInstructionData)?;

        let bump_seed = {
            let data = &mut self.mint_config.data.borrow_mut();
            let config = load_mint_config(data)?;

            if config.freeze_authority != *self.authority.key {
                return Err(TokenAclError::InvalidAuthority.into());
            }

            if config.mint != *self.mint.key {
                return Err(TokenAclError::InvalidTokenMint.into());
            }

            [config.bump]
        };

        let seeds = [MintConfig::SEED_PREFIX, self.mint.key.as_ref(), &bump_seed];

        let ix = spl_token_2022::instruction::set_authority(
            self.token_program.key,
            self.mint.key,
            Some(&new_freeze_authority),
            AuthorityType::FreezeAccount,
            self.mint_config.key,
            &[],
        )?;
        invoke_signed(
            &ix,
            &[self.mint.clone(), self.mint_config.clone()],
            &[&seeds],
        )?;

        **self.receiver.try_borrow_mut_lamports()? += self.mint_config.lamports();
        **self.mint_config.try_borrow_mut_lamports()? = 0;
        self.mint_config.realloc(0, false)?;

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo<'a>]> for DeleteConfig<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'a>]) -> Result<Self, Self::Error> {
        let [authority, receiver, mint, mint_config, token_program] = &accounts else {
            return Err(ProgramError::InvalidInstructionData);
        };

        if !authority.is_signer {
            return Err(TokenAclError::InvalidAuthority.into());
        }

        if !spl_token_2022::check_id(token_program.key) {
            return Err(TokenAclError::InvalidTokenProgram.into());
        }

        Ok(Self {
            authority,
            receiver,
            mint,
            mint_config,
            token_program,
        })
    }
}
