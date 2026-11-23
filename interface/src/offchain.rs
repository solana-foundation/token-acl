use spl_discriminator::SplDiscriminate;
pub use spl_tlv_account_resolution::state::{AccountDataResult, AccountFetchError};

use crate::{
    get_freeze_extra_account_metas_address,
    instruction::{can_freeze_permissionless_with_data, CanFreezePermissionlessInstruction},
};

use {
    crate::{
        error::ThawFreezeGateError,
        get_thaw_extra_account_metas_address,
        instruction::{can_thaw_permissionless_with_data, CanThawPermissionlessInstruction},
    },
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
    spl_tlv_account_resolution::state::ExtraAccountMetaList,
    std::future::Future,
};

/// Resolves the gating program's extra account metas for a permissionless
/// freeze and appends them to `instruction`.
///
/// Equivalent to [`add_extra_account_metas_for_freeze_with_data`] with an
/// empty payload. If the Token ACL instruction carries a payload, use the
/// `_with_data` variant so `Seed::InstructionData` seeds resolve correctly.
#[allow(clippy::too_many_arguments)]
pub async fn add_extra_account_metas_for_freeze<F, Fut>(
    instruction: &mut Instruction,
    program_id: &Pubkey,
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_account_owner: &Pubkey,
    flag_account_pubkey: &Pubkey,
    fetch_account_data_fn: F,
) -> Result<(), ThawFreezeGateError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    add_extra_account_metas_for_freeze_with_data(
        instruction,
        program_id,
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        token_account_owner,
        flag_account_pubkey,
        &[],
        fetch_account_data_fn,
    )
    .await
}

/// Resolves the gating program's extra account metas for a permissionless
/// freeze whose payload is `data` and appends them to `instruction`.
///
/// `data` must be the bytes the Token ACL instruction carries after its 1-byte
/// discriminator (i.e. what Token ACL will forward to the gate). Resolution
/// runs against the gate instruction `discriminator || data`, matching the
/// on-chain CPI.
#[allow(clippy::too_many_arguments)]
pub async fn add_extra_account_metas_for_freeze_with_data<F, Fut>(
    instruction: &mut Instruction,
    program_id: &Pubkey,
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_account_owner: &Pubkey,
    flag_account_pubkey: &Pubkey,
    data: &[u8],
    fetch_account_data_fn: F,
) -> Result<(), ThawFreezeGateError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    let extra_metas_pubkey = get_freeze_extra_account_metas_address(mint_pubkey, program_id);
    let cpi_ix = can_freeze_permissionless_with_data(
        program_id,
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        token_account_owner,
        flag_account_pubkey,
        data,
    );

    add_extra_account_metas_for_permissionless_ix::<_, _, CanFreezePermissionlessInstruction>(
        instruction,
        cpi_ix,
        &extra_metas_pubkey,
        fetch_account_data_fn,
    )
    .await
}

/// Resolves the gating program's extra account metas for a permissionless
/// thaw and appends them to `instruction`.
///
/// Equivalent to [`add_extra_account_metas_for_thaw_with_data`] with an empty
/// payload. If the Token ACL instruction carries a payload, use the
/// `_with_data` variant so `Seed::InstructionData` seeds resolve correctly.
#[allow(clippy::too_many_arguments)]
pub async fn add_extra_account_metas_for_thaw<F, Fut>(
    instruction: &mut Instruction,
    program_id: &Pubkey,
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_account_owner: &Pubkey,
    flag_account_pubkey: &Pubkey,
    fetch_account_data_fn: F,
) -> Result<(), ThawFreezeGateError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    add_extra_account_metas_for_thaw_with_data(
        instruction,
        program_id,
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        token_account_owner,
        flag_account_pubkey,
        &[],
        fetch_account_data_fn,
    )
    .await
}

/// Resolves the gating program's extra account metas for a permissionless
/// thaw whose payload is `data` and appends them to `instruction`.
///
/// `data` must be the bytes the Token ACL instruction carries after its 1-byte
/// discriminator (i.e. what Token ACL will forward to the gate). Resolution
/// runs against the gate instruction `discriminator || data`, matching the
/// on-chain CPI.
#[allow(clippy::too_many_arguments)]
pub async fn add_extra_account_metas_for_thaw_with_data<F, Fut>(
    instruction: &mut Instruction,
    program_id: &Pubkey,
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_account_owner: &Pubkey,
    flag_account_pubkey: &Pubkey,
    data: &[u8],
    fetch_account_data_fn: F,
) -> Result<(), ThawFreezeGateError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    let extra_metas_pubkey = get_thaw_extra_account_metas_address(mint_pubkey, program_id);
    let cpi_ix = can_thaw_permissionless_with_data(
        program_id,
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        token_account_owner,
        flag_account_pubkey,
        data,
    );

    add_extra_account_metas_for_permissionless_ix::<_, _, CanThawPermissionlessInstruction>(
        instruction,
        cpi_ix,
        &extra_metas_pubkey,
        fetch_account_data_fn,
    )
    .await
}

/// Resolves the extra metas of `cpi_ix` (the gate instruction, including any
/// forwarded payload) and appends the resolved accounts to `instruction`.
async fn add_extra_account_metas_for_permissionless_ix<F, Fut, T>(
    instruction: &mut Instruction,
    mut cpi_ix: Instruction,
    extra_metas_pubkey: &Pubkey,
    fetch_account_data_fn: F,
) -> Result<(), ThawFreezeGateError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
    T: SplDiscriminate,
{
    let validate_state_data = fetch_account_data_fn(*extra_metas_pubkey)
        .await
        .map_err(|_| ThawFreezeGateError::MissingExtraAccountMeta)?
        .ok_or(ThawFreezeGateError::MissingExtraAccountMeta)?;

    // Check to make sure the provided keys are in the instruction
    // (gating program, signer, token account, mint, token account owner).
    if std::iter::once(&cpi_ix.program_id)
        .chain(cpi_ix.accounts[..4].iter().map(|meta| &meta.pubkey))
        .any(|key| !instruction.accounts.iter().any(|meta| meta.pubkey == *key))
    {
        Err(ThawFreezeGateError::MissingAccountMeta)?;
    }

    let base_accounts = cpi_ix.accounts.len();
    cpi_ix
        .accounts
        .push(AccountMeta::new_readonly(*extra_metas_pubkey, false));

    // `cpi_ix.data` is `discriminator || payload`, exactly what the on-chain
    // CPI carries, so `Seed::InstructionData` seeds resolve identically.
    ExtraAccountMetaList::add_to_instruction::<T, _, _>(
        &mut cpi_ix,
        fetch_account_data_fn,
        &validate_state_data,
    )
    .await?;

    // Add only the extra accounts resolved from the validation state
    instruction
        .accounts
        .extend_from_slice(&cpi_ix.accounts[base_accounts..]);

    Ok(())
}
