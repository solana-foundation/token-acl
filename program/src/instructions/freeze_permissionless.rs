use solana_cpi::invoke_signed;
use solana_program::account_info::AccountInfo;
use solana_program_error::{ProgramError, ProgramResult};
use solana_pubkey::Pubkey;
use spl_token_2022::{extension::StateWithExtensions, state::AccountState};
use token_acl_interface::onchain::invoke_can_freeze_permissionless;

use crate::{
    error::TokenAclError,
    state::{load_mint_config, MintConfig, FLAG_ACCOUNT_SEED_PREFIX},
};

pub struct FreezePermissionless<'a> {
    pub authority: &'a AccountInfo<'a>,
    pub mint: &'a AccountInfo<'a>,
    pub token_account: &'a AccountInfo<'a>,
    pub token_account_owner: &'a AccountInfo<'a>,
    pub mint_config: &'a AccountInfo<'a>,
    pub flag_account: &'a AccountInfo<'a>,
    pub token_program: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub gating_program: &'a AccountInfo<'a>,
    pub remaining_accounts: &'a [AccountInfo<'a>],
    pub flag_account_bump: u8,
}

impl FreezePermissionless<'_> {
    pub const DISCRIMINATOR: u8 = 7;

    pub fn process(&self, is_idempotent: bool) -> ProgramResult {
        let data = &self.mint_config.data.borrow();
        let config = load_mint_config(data)?;

        if config.mint != *self.mint.key {
            return Err(TokenAclError::InvalidTokenMint.into());
        }

        if !config.is_permissionless_freeze_enabled() {
            return Err(TokenAclError::PermissionlessFreezeNotEnabled.into());
        }

        if is_idempotent {
            let ta_data = self.token_account.data.borrow();
            let ta = StateWithExtensions::<spl_token_2022::state::Account>::unpack(&ta_data)?;
            if ta.base.state != AccountState::Initialized {
                return Ok(());
            }

            if ta.base.owner != *self.token_account_owner.key {
                return Err(TokenAclError::InvalidTokenAccountOwner.into());
            }
            // no need to enforce ta.mint == self.mint.key, freeze CPI will do this
        }

        if config.gating_program != *self.gating_program.key {
            return Err(TokenAclError::InvalidGatingProgram.into());
        }

        let bump_seed = [self.flag_account_bump];
        let seeds = [
            FLAG_ACCOUNT_SEED_PREFIX,
            self.token_account.key.as_ref(),
            &bump_seed,
        ];

        let ix = solana_system_interface::instruction::create_account(
            self.authority.key,
            self.flag_account.key,
            0,
            1 as u64,
            &crate::ID,
        );

        invoke_signed(
            &ix,
            &[self.authority.clone(), self.flag_account.clone()],
            &[&seeds],
        )?;

        self.flag_account.data.borrow_mut()[0] = 1;

        invoke_can_freeze_permissionless(
            self.gating_program.key,
            self.authority.clone(),
            self.token_account.clone(),
            self.mint.clone(),
            self.token_account_owner.clone(),
            self.flag_account.clone(),
            self.remaining_accounts,
        )?;

        self.flag_account.data.borrow_mut()[0] = 0;

        let bump_seed = [config.bump];
        let seeds = [MintConfig::SEED_PREFIX, self.mint.key.as_ref(), &bump_seed];

        let ix = spl_token_2022::instruction::freeze_account(
            self.token_program.key,
            self.token_account.key,
            self.mint.key,
            self.mint_config.key,
            &[],
        )?;
        invoke_signed(
            &ix,
            &[
                self.token_account.clone(),
                self.mint.clone(),
                self.mint_config.clone(),
            ],
            &[&seeds],
        )?;

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo<'a>]> for FreezePermissionless<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'a>]) -> Result<Self, Self::Error> {
        let [authority, mint, token_account, flag_account, token_account_owner, mint_config, token_program, system_program, gating_program, remaining_accounts @ ..] =
            &accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !authority.is_signer {
            return Err(TokenAclError::InvalidAuthority.into());
        }

        if !spl_token_2022::check_id(token_program.key) {
            return Err(TokenAclError::InvalidTokenProgram.into());
        }
        
        if !solana_system_interface::program::check_id(system_program.key) {
            return Err(TokenAclError::InvalidSystemProgram.into());
        }

        let (_, flag_account_bump) = Pubkey::find_program_address(
            &[FLAG_ACCOUNT_SEED_PREFIX, token_account.key.as_ref()],
            &crate::ID,
        );

        Ok(Self {
            authority,
            mint,
            token_account,
            flag_account,
            token_account_owner,
            mint_config,
            token_program,
            system_program,
            gating_program,
            remaining_accounts,
            flag_account_bump,
        })
    }
}
