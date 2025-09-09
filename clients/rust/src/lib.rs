mod generated;
use std::future::Future;

pub use generated::*;

use solana_instruction::Instruction;
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;
pub use spl_tlv_account_resolution::state::{AccountDataResult, AccountFetchError};

use crate::generated::errors::ebalts;

#[allow(clippy::too_many_arguments)]
pub async fn create_thaw_permissionless_instruction_with_extra_metas<F, Fut>(
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_config_pubkey: &Pubkey,
    token_program_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    idempotent: bool,
    fetch_account_data_fn: F,
) -> Result<Instruction, AccountFetchError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    let mint_config = fetch_account_data_fn(*mint_config_pubkey)
        .await?
        .and_then(|data| crate::accounts::MintConfig::from_bytes(&data).ok())
        .ok_or(ProgramError::InvalidAccountData)?;

    
    if !mint_config.enable_permissionless_thaw {
        return Err(ebalts::EbaltsError::PermissionlessThawNotEnabled.into());
    }

    let mut ix = if idempotent {
        crate::instructions::ThawPermissionlessIdempotentBuilder::new()
            .gating_program(mint_config.gating_program)
            .authority(*signer_pubkey)
            .mint(*mint_pubkey)
            .token_account(*token_account_pubkey)
            .token_account_owner(*token_account_owner_pubkey)
            .mint_config(*mint_config_pubkey)
            .token_program(*token_program_pubkey)
            .instruction()
    } else {
        crate::instructions::ThawPermissionlessBuilder::new()
            .gating_program(mint_config.gating_program)
            .authority(*signer_pubkey)
            .mint(*mint_pubkey)
            .token_account(*token_account_pubkey)
            .token_account_owner(*token_account_owner_pubkey)
            .mint_config(*mint_config_pubkey)
            .token_program(*token_program_pubkey)
            .instruction()
    };

    if mint_config.gating_program != Pubkey::default() {
        ebalts_interface::offchain::add_extra_account_metas_for_thaw(
            &mut ix,
            &mint_config.gating_program,
            signer_pubkey,
            token_account_pubkey,
            mint_pubkey,
            token_account_owner_pubkey,
            fetch_account_data_fn,
        )
        .await?;
    }

    Ok(ix)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_freeze_permissionless_instruction_with_extra_metas<F, Fut>(
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_config_pubkey: &Pubkey,
    token_program_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    idempotent: bool,
    fetch_account_data_fn: F,
) -> Result<Instruction, AccountFetchError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    let mint_config = fetch_account_data_fn(*mint_config_pubkey)
        .await?
        .and_then(|data| crate::accounts::MintConfig::from_bytes(&data).ok())
        .ok_or(ProgramError::InvalidAccountData)?;

    if !mint_config.enable_permissionless_freeze {
        return Err(ebalts::EbaltsError::PermissionlessFreezeNotEnabled.into());
    }

    let mut ix = if idempotent {
        crate::instructions::FreezePermissionlessIdempotentBuilder::new()
            .gating_program(mint_config.gating_program)
            .authority(*signer_pubkey)
            .mint(*mint_pubkey)
            .token_account(*token_account_pubkey)
            .token_account_owner(*token_account_owner_pubkey)
            .mint_config(*mint_config_pubkey)
            .token_program(*token_program_pubkey)
            .instruction()
    } else {
        crate::instructions::FreezePermissionlessBuilder::new()
            .gating_program(mint_config.gating_program)
            .authority(*signer_pubkey)
            .mint(*mint_pubkey)
            .token_account(*token_account_pubkey)
            .token_account_owner(*token_account_owner_pubkey)
            .mint_config(*mint_config_pubkey)
            .token_program(*token_program_pubkey)
            .instruction()
    };

    if mint_config.gating_program != Pubkey::default() {
        ebalts_interface::offchain::add_extra_account_metas_for_freeze(
            &mut ix,
            &mint_config.gating_program,
            signer_pubkey,
            token_account_pubkey,
            mint_pubkey,
            token_account_owner_pubkey,
            fetch_account_data_fn,
        )
        .await?;
    }

    Ok(ix)
}
