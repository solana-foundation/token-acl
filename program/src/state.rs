use bytemuck::{Pod, Zeroable};
use solana_program_error::ProgramError;
use spl_pod::primitives::PodBool;

use crate::error::EbaltsError;
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct MintConfig {
    pub discriminator: u8,
    pub mint: Pubkey,
    pub freeze_authority: Pubkey,
    pub gating_program: Pubkey,
    pub bump: u8,
    pub enable_permissionless_thaw: PodBool,
    pub enable_permissionless_freeze: PodBool,
}

impl MintConfig {
    pub const SEED_PREFIX: &'static [u8] = b"MINT_CONFIG";
    pub const DISCRIMINATOR: u8 = 1;
    pub const LEN: usize = 1 + 32 + 32 + 32 + 1 + 1 + 1;

    pub fn is_permissionless_thaw_enabled(&self) -> bool {
        Into::<bool>::into(self.enable_permissionless_thaw)
    }

    pub fn is_permissionless_freeze_enabled(&self) -> bool {
        Into::<bool>::into(self.enable_permissionless_freeze)
    }
}

#[inline(always)]
pub fn load_mint_config(data: &[u8]) -> Result<&MintConfig, ProgramError> {
    bytemuck::try_from_bytes::<MintConfig>(data)
        .map_err(|_| EbaltsError::InvalidMintConfig.into())
        .and_then(|cfg: &MintConfig| {
            if cfg.discriminator == MintConfig::DISCRIMINATOR {
                Ok(cfg)
            } else {
                Err(EbaltsError::InvalidMintConfig.into())
            }
        })
}

#[inline(always)]
pub fn load_mint_config_mut(data: &mut [u8]) -> Result<&mut MintConfig, ProgramError> {
    bytemuck::try_from_bytes_mut::<MintConfig>(data)
        .map_err(|_| EbaltsError::InvalidMintConfig.into())
        .and_then(|cfg: &mut MintConfig| {
            if cfg.discriminator == MintConfig::DISCRIMINATOR {
                Ok(cfg)
            } else {
                Err(EbaltsError::InvalidMintConfig.into())
            }
        })
}
