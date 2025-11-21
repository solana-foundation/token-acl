use solana_program::account_info::AccountInfo;
use solana_program_error::{ProgramError, ProgramResult};

pub struct CanThawFreezePermissionless<'a> {
    pub authority: &'a AccountInfo<'a>,
    pub token_account: &'a AccountInfo<'a>,
    pub mint: &'a AccountInfo<'a>,
    pub token_account_owner: &'a AccountInfo<'a>,
    pub flag_account: &'a AccountInfo<'a>,
    pub extra_metas: &'a AccountInfo<'a>,
    pub associated_token_program: &'a AccountInfo<'a>,
    pub token_program: &'a AccountInfo<'a>,
    pub token_account_owner_again: &'a AccountInfo<'a>,
    pub ata: &'a AccountInfo<'a>,
    pub extra_metas_again: &'a AccountInfo<'a>,
}

impl CanThawFreezePermissionless<'_> {
    pub fn process(&self) -> ProgramResult {
        if self.ata.key != self.token_account.key {
            return Err(ProgramError::InvalidArgument);
        }

        if self.extra_metas.key != self.extra_metas_again.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if self.token_program.key != &spl_token_2022::ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        if self.flag_account.owner != &token_acl_interface::TOKEN_ACL_ID
            && *self.flag_account.data.borrow() != [1u8]
        {
            return Err(ProgramError::InvalidAccountData);
        }

        if self.associated_token_program.key != &spl_associated_token_account_client::program::ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        if self.token_account_owner.key != self.token_account_owner_again.key {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo<'a>]> for CanThawFreezePermissionless<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'a>]) -> Result<Self, Self::Error> {
        let [authority, token_account, mint, token_account_owner, flag_account, extra_metas, associated_token_program, token_program, token_account_owner_again, ata, extra_metas_again] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            authority,
            token_account,
            mint,
            token_account_owner,
            flag_account,
            extra_metas,
            associated_token_program,
            token_program,
            token_account_owner_again,
            ata,
            extra_metas_again,
        })
    }
}
