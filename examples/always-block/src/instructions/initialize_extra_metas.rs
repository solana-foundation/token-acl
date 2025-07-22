use ebalts_interface::instruction::{
    CanFreezePermissionlessInstruction, CanThawPermissionlessInstruction,
};
use solana_cpi::invoke_signed;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use solana_program_error::{ProgramError, ProgramResult};
use solana_rent::Rent;
use solana_sysvar::Sysvar;
use spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList};

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

    pub fn process(&self) -> ProgramResult {
        let size = ExtraAccountMetaList::size_of(0).unwrap();
        let lamports = Rent::get()?.minimum_balance(size);

        let bump_seed = [self.thaw_bump];
        let seeds = [
            ebalts_interface::THAW_EXTRA_ACCOUNT_METAS_SEED,
            self.mint.key.as_ref(),
            &bump_seed,
        ];

        let ix = solana_system_interface::instruction::create_account(
            self.payer.key,
            self.thaw_extra_metas.key,
            lamports,
            size as u64,
            &crate::ID,
        );
        invoke_signed(
            &ix,
            &[self.payer.clone(), self.thaw_extra_metas.clone()],
            &[&seeds],
        )?;

        let bump_seed = [self.freeze_bump];
        let seeds = [
            ebalts_interface::FREEZE_EXTRA_ACCOUNT_METAS_SEED,
            self.mint.key.as_ref(),
            &bump_seed,
        ];

        let ix = solana_system_interface::instruction::create_account(
            self.payer.key,
            self.freeze_extra_metas.key,
            lamports,
            size as u64,
            &crate::ID,
        );
        invoke_signed(
            &ix,
            &[self.payer.clone(), self.freeze_extra_metas.clone()],
            &[&seeds],
        )?;

        let metas: Vec<ExtraAccountMeta> = vec![];
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
                ebalts_interface::THAW_EXTRA_ACCOUNT_METAS_SEED,
                mint.key.as_ref(),
            ],
            &crate::ID,
        );
        let (_, freeze_bump) = Pubkey::find_program_address(
            &[
                ebalts_interface::FREEZE_EXTRA_ACCOUNT_METAS_SEED,
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
