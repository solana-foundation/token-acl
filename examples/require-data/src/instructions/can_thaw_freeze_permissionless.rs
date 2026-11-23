use solana_program::account_info::AccountInfo;
use solana_program_error::{ProgramError, ProgramResult};

use crate::{find_payload_address, CustomErrors, EXPECTED_PAYLOAD};

pub struct CanThawFreezePermissionless<'a> {
    pub authority: &'a AccountInfo<'a>,
    pub token_account: &'a AccountInfo<'a>,
    pub mint: &'a AccountInfo<'a>,
    pub token_account_owner: &'a AccountInfo<'a>,
    pub flag_account: &'a AccountInfo<'a>,
    pub extra_metas: &'a AccountInfo<'a>,
    /// Extra account meta derived from `[b"payload", payload[0..32]]`.
    pub payload_account: &'a AccountInfo<'a>,
}

impl CanThawFreezePermissionless<'_> {
    /// `payload` is everything after the 8-byte sRFC37 discriminator, i.e. the
    /// bytes the caller appended to Token ACL's instruction. It is untrusted.
    pub fn process(&self, payload: &[u8]) -> ProgramResult {
        // Only accept calls made by Token ACL while it holds the flag.
        if self.flag_account.owner != &token_acl_interface::TOKEN_ACL_ID
            || *self.flag_account.data.borrow() != [1u8]
        {
            return Err(CustomErrors::InvalidFlagAccount.into());
        }

        if payload != EXPECTED_PAYLOAD {
            return Err(CustomErrors::PayloadMismatch.into());
        }

        // Token ACL resolved this account from the extra metas using the
        // payload as a seed; re-derive it to prove the resolution saw it.
        if *self.payload_account.key != find_payload_address(payload).0 {
            return Err(CustomErrors::PayloadAccountMismatch.into());
        }

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo<'a>]> for CanThawFreezePermissionless<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'a>]) -> Result<Self, Self::Error> {
        let [authority, token_account, mint, token_account_owner, flag_account, extra_metas, payload_account] =
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
            payload_account,
        })
    }
}
