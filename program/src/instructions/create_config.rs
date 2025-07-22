use solana_cpi::{invoke, invoke_signed};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use solana_program_error::{ProgramError, ProgramResult};
use solana_rent::Rent;
use solana_sysvar::Sysvar;
use spl_pod::{bytemuck::pod_from_bytes_mut, primitives::PodBool};
use spl_token_2022::{
    extension::{
        default_account_state::DefaultAccountState, BaseStateWithExtensions, PodStateWithExtensions,
    },
    instruction::AuthorityType,
    pod::PodMint,
};

use crate::{error::EbaltsError, state::MintConfig};

pub struct CreateConfig<'a> {
    pub payer: &'a AccountInfo<'a>,
    pub authority: &'a AccountInfo<'a>,
    pub mint: &'a AccountInfo<'a>,
    pub mint_config: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub token_program: &'a AccountInfo<'a>,
    pub config_bump: u8,
}

impl CreateConfig<'_> {
    pub const DISCRIMINATOR: u8 = 0;

    pub fn process(&self, remaining_data: &[u8]) -> ProgramResult {
        if remaining_data.len() != 32 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let gating_program =
            Pubkey::try_from(remaining_data).map_err(|_| ProgramError::InvalidInstructionData)?;

        let lamports = Rent::get()?.minimum_balance(MintConfig::LEN);
        let bump_seed = [self.config_bump];
        let seeds = [MintConfig::SEED_PREFIX, self.mint.key.as_ref(), &bump_seed];

        let ix = solana_system_interface::instruction::create_account(
            self.authority.key,
            self.mint_config.key,
            lamports,
            MintConfig::LEN as u64,
            &crate::ID,
        );

        invoke_signed(
            &ix,
            &[self.authority.clone(), self.mint_config.clone()],
            &[&seeds],
        )?;

        let data = &mut self.mint_config.data.borrow_mut();
        let config = pod_from_bytes_mut::<MintConfig>(data)?;

        config.discriminator = MintConfig::DISCRIMINATOR;
        config.mint = *self.mint.key;
        config.freeze_authority = *self.authority.key;
        config.gating_program = gating_program;
        config.bump = self.config_bump;
        config.enable_permissionless_freeze = PodBool::from_bool(false);
        config.enable_permissionless_thaw = PodBool::from_bool(false);

        let mint_data = self.mint.data.borrow_mut();
        let mint = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;

        // if no freeze authority, or DSA extension is not present,
        // this is an invalid mint for this standard
        // these can't also be changed or activated later for existing mints

        mint.get_extension::<DefaultAccountState>()
            .map_err(|_| Into::<ProgramError>::into(EbaltsError::InvalidTokenMint))?;

        let freeze_authority = mint
            .base
            .freeze_authority
            .ok_or(Into::<ProgramError>::into(EbaltsError::InvalidTokenMint))?;

        drop(mint_data);
        if freeze_authority == *self.authority.key {
            // we can cpi to change freeze authority right away
            let ix = spl_token_2022::instruction::set_authority(
                self.token_program.key,
                self.mint.key,
                Some(self.mint_config.key),
                AuthorityType::FreezeAccount,
                self.authority.key,
                &[],
            )?;
            invoke(&ix, &[self.mint.clone(), self.authority.clone()])?;
        }

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo<'a>]> for CreateConfig<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'a>]) -> Result<Self, Self::Error> {
        let [payer, authority, mint, mint_config, system_program, token_program] = &accounts else {
            return Err(ProgramError::InvalidInstructionData);
        };

        if !authority.is_signer {
            return Err(EbaltsError::InvalidAuthority.into());
        }

        let (_, config_bump) =
            Pubkey::find_program_address(&[MintConfig::SEED_PREFIX, mint.key.as_ref()], &crate::ID);

        if !solana_system_interface::program::check_id(system_program.key) {
            return Err(EbaltsError::InvalidSystemProgram.into());
        }

        if !spl_token_2022::check_id(token_program.key) {
            return Err(EbaltsError::InvalidTokenProgram.into());
        }

        Ok(Self {
            payer,
            authority,
            mint,
            mint_config,
            system_program,
            token_program,
            config_bump,
        })
    }
}
