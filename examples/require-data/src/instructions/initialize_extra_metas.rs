use solana_cpi::invoke_signed;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use solana_program_error::{ProgramError, ProgramResult};
use solana_rent::Rent;
use solana_sysvar::Sysvar;
use spl_discriminator::ArrayDiscriminator;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use token_acl_interface::instruction::{
    CanFreezePermissionlessInstruction, CanThawPermissionlessInstruction,
};

use crate::{EXPECTED_PAYLOAD, PAYLOAD_SEED_PREFIX};

pub struct InitializeExtraMetas<'a> {
    pub payer: &'a AccountInfo<'a>,
    pub mint: &'a AccountInfo<'a>,
    pub thaw_extra_metas: &'a AccountInfo<'a>,
    pub freeze_extra_metas: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub thaw_bump: u8,
    pub freeze_bump: u8,
}

impl InitializeExtraMetas<'_> {
    pub const DISCRIMINATOR: [u8; 8] = [1; 8];
    pub const DISCRIMINATOR_SLICE: &'static [u8] = Self::DISCRIMINATOR.as_slice();

    /// `[6]` payload PDA: `[b"payload", instruction_data[8..40]]`, i.e. the
    /// first 32 payload bytes after the gate's 8-byte discriminator.
    fn metas() -> Result<Vec<ExtraAccountMeta>, ProgramError> {
        Ok(vec![ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: PAYLOAD_SEED_PREFIX.to_vec(),
                },
                Seed::InstructionData {
                    index: ArrayDiscriminator::LENGTH as u8,
                    length: EXPECTED_PAYLOAD.len() as u8,
                },
            ],
            false,
            false,
        )?])
    }

    pub fn process(&self) -> ProgramResult {
        let metas = Self::metas()?;
        let size = ExtraAccountMetaList::size_of(metas.len())?;
        let lamports = Rent::get()?.minimum_balance(size);

        for (account, seed, bump) in [
            (
                self.thaw_extra_metas,
                token_acl_interface::THAW_EXTRA_ACCOUNT_METAS_SEED,
                self.thaw_bump,
            ),
            (
                self.freeze_extra_metas,
                token_acl_interface::FREEZE_EXTRA_ACCOUNT_METAS_SEED,
                self.freeze_bump,
            ),
        ] {
            let bump_seed = [bump];
            let seeds = [seed, self.mint.key.as_ref(), &bump_seed];
            let ix = solana_system_interface::instruction::create_account(
                self.payer.key,
                account.key,
                lamports,
                size as u64,
                &crate::ID,
            );
            invoke_signed(&ix, &[self.payer.clone(), account.clone()], &[&seeds])?;
        }

        ExtraAccountMetaList::init::<CanThawPermissionlessInstruction>(
            &mut self.thaw_extra_metas.data.borrow_mut(),
            &metas,
        )?;
        ExtraAccountMetaList::init::<CanFreezePermissionlessInstruction>(
            &mut self.freeze_extra_metas.data.borrow_mut(),
            &metas,
        )?;
        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo<'a>]> for InitializeExtraMetas<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'a>]) -> Result<Self, Self::Error> {
        let [payer, mint, thaw_extra_metas, freeze_extra_metas, system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let (_, thaw_bump) = Pubkey::find_program_address(
            &[
                token_acl_interface::THAW_EXTRA_ACCOUNT_METAS_SEED,
                mint.key.as_ref(),
            ],
            &crate::ID,
        );
        let (_, freeze_bump) = Pubkey::find_program_address(
            &[
                token_acl_interface::FREEZE_EXTRA_ACCOUNT_METAS_SEED,
                mint.key.as_ref(),
            ],
            &crate::ID,
        );

        Ok(Self {
            payer,
            mint,
            thaw_extra_metas,
            freeze_extra_metas,
            system_program,
            thaw_bump,
            freeze_bump,
        })
    }
}
