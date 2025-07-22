use crate::state::load_mint_config;
pub use spl_tlv_account_resolution::state::{AccountDataResult, AccountFetchError};

use {
    solana_program::{instruction::Instruction, program_error::ProgramError, pubkey::Pubkey},
    std::future::Future,
};

#[allow(clippy::too_many_arguments)]
pub async fn create_thaw_permissionless_instruction_with_extra_metas<F, Fut>(
    program_id: &Pubkey,
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_config_pubkey: &Pubkey,
    token_program_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    fetch_account_data_fn: F,
) -> Result<Instruction, AccountFetchError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    let mut thaw_permissionless_instruction = crate::instruction::thaw_permissionless(
        program_id,
        signer_pubkey,
        mint_pubkey,
        mint_config_pubkey,
        token_account_pubkey,
        token_account_owner_pubkey,
        token_program_pubkey,
    );

    add_extra_account_metas_for_thaw(
        &mut thaw_permissionless_instruction,
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        token_account_owner_pubkey,
        mint_config_pubkey,
        fetch_account_data_fn,
    )
    .await?;

    Ok(thaw_permissionless_instruction)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_freeze_permissionless_instruction_with_extra_metas<F, Fut>(
    program_id: &Pubkey,
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_config_pubkey: &Pubkey,
    token_program_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    fetch_account_data_fn: F,
) -> Result<Instruction, AccountFetchError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    let mut freeze_permissionless_instruction = crate::instruction::freeze_permissionless(
        program_id,
        signer_pubkey,
        mint_pubkey,
        mint_config_pubkey,
        token_account_pubkey,
        token_account_owner_pubkey,
        token_program_pubkey,
    );

    add_extra_account_metas_for_freeze(
        &mut freeze_permissionless_instruction,
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        token_account_owner_pubkey,
        mint_config_pubkey,
        fetch_account_data_fn,
    )
    .await?;

    Ok(freeze_permissionless_instruction)
}

pub async fn add_extra_account_metas_for_thaw<F, Fut>(
    instruction: &mut Instruction,
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    mint_config_pubkey: &Pubkey,
    fetch_account_data_fn: F,
) -> Result<(), AccountFetchError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    let mint_config_data = fetch_account_data_fn(*mint_config_pubkey)
        .await?
        .ok_or(ProgramError::InvalidAccountData)?;
    let mint_config = load_mint_config(&mint_config_data)?;

    if mint_config.gating_program != Pubkey::default() {
        ebalts_interface::offchain::add_extra_account_metas_for_thaw(
            instruction,
            &mint_config.gating_program,
            signer_pubkey,
            token_account_pubkey,
            mint_pubkey,
            token_account_owner_pubkey,
            fetch_account_data_fn,
        )
        .await?;
    }

    Ok(())
}

pub async fn add_extra_account_metas_for_freeze<F, Fut>(
    instruction: &mut Instruction,
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    mint_config_pubkey: &Pubkey,
    fetch_account_data_fn: F,
) -> Result<(), AccountFetchError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    let mint_config_data = fetch_account_data_fn(*mint_config_pubkey)
        .await?
        .ok_or(ProgramError::InvalidAccountData)?;
    let mint_config = load_mint_config(&mint_config_data)?;

    if mint_config.gating_program != Pubkey::default() {
        ebalts_interface::offchain::add_extra_account_metas_for_freeze(
            instruction,
            &mint_config.gating_program,
            signer_pubkey,
            token_account_pubkey,
            mint_pubkey,
            token_account_owner_pubkey,
            fetch_account_data_fn,
        )
        .await?;
    }

    Ok(())
}
