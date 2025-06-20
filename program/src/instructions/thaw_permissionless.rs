use ebalts_interface::onchain::invoke_can_thaw_permissionless;
use solana_cpi::invoke_signed;
use solana_program_error::{ProgramError, ProgramResult};
use solana_program::account_info::AccountInfo;

use crate::{error::EbaltsError, state::{load_mint_config, MintConfig}};


pub struct ThawPermissionless<'a> {
    pub authority: &'a AccountInfo<'a>,
    pub mint: &'a AccountInfo<'a>,
    pub token_account: &'a AccountInfo<'a>,
    pub mint_config: &'a AccountInfo<'a>,
    pub token_program: &'a AccountInfo<'a>,
    pub remaining_accounts: &'a [AccountInfo<'a>],
}

impl<'a> ThawPermissionless<'a> {
    pub const DISCRIMINATOR: u8 = 6;

    pub fn process(&self) -> ProgramResult {
        let data = &self.mint_config.data.borrow();
        let config = load_mint_config(data)?;

        if config.freeze_authority != *self.authority.key {
            return Err(EbaltsError::InvalidAuthority.into());
        }

        if config.mint != *self.mint.key {
            return Err(EbaltsError::InvalidTokenMint.into());
        }

        if !config.is_permissionless_thaw_enabled() {
            return Err(EbaltsError::PermissionlessThawNotEnabled.into());
        }

        invoke_can_thaw_permissionless(
            &crate::ID,
            self.authority.clone(),
            self.token_account.clone(),
            self.mint.clone(),
            self.remaining_accounts,
        )?;

        let bump_seed = [config.bump];
        let seeds = [MintConfig::SEED_PREFIX, self.mint.key.as_ref(), &bump_seed];

        let ix = spl_token_2022::instruction::thaw_account(self.token_program.key, self.token_account.key, self.mint.key, self.authority.key, &[])?;
        invoke_signed(&ix, &[self.token_account.clone(), self.mint.clone(), self.authority.clone()], &[&seeds])?;

        Ok(())
    }

}

impl<'a> TryFrom<&'a [AccountInfo<'a>]> for ThawPermissionless<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'a>]) -> Result<Self, Self::Error> {
        let [authority, mint, token_account, mint_config, token_program, remaining_accounts @ ..] = &accounts else {
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
            token_account,
            mint_config,
            token_program,
            remaining_accounts,
        })
    }
}