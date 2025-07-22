use spl_discriminator::SplDiscriminate;
pub use spl_tlv_account_resolution::state::{AccountDataResult, AccountFetchError};

use crate::{
    get_freeze_extra_account_metas_address,
    instruction::{can_freeze_permissionless, CanFreezePermissionlessInstruction},
};

use {
    crate::{
        error::ThawFreezeGateError,
        get_thaw_extra_account_metas_address,
        instruction::{can_thaw_permissionless, CanThawPermissionlessInstruction},
    },
    solana_instruction::{AccountMeta, Instruction},
    solana_program_error::ProgramError,
    solana_pubkey::Pubkey,
    spl_tlv_account_resolution::state::ExtraAccountMetaList,
    std::future::Future,
};

// #[allow(clippy::too_many_arguments)]
// pub async fn add_extra_account_metas_for_thaw<F, Fut>(
//     instruction: &mut Instruction,
//     program_id: &Pubkey,
//     signer_pubkey: &Pubkey,
//     token_account_pubkey: &Pubkey,
//     mint_pubkey: &Pubkey,
//     fetch_account_data_fn: F,
// ) -> Result<(), AccountFetchError>
// where
//     F: Fn(Pubkey) -> Fut,
//     Fut: Future<Output = AccountDataResult>,
// {
//     let validate_state_pubkey = get_thaw_extra_account_metas_address(mint_pubkey, program_id);
//     let validate_state_data = fetch_account_data_fn(validate_state_pubkey)
//         .await?
//         .ok_or(ProgramError::InvalidAccountData)?;

//     // Check to make sure the provided keys are in the instruction
//     if [
//         signer_pubkey,
//         token_account_pubkey,
//         mint_pubkey,
//     ]
//     .iter()
//     .any(|&key| !instruction.accounts.iter().any(|meta| meta.pubkey == *key))
//     {
//         Err(ThawFreezeGateError::IncorrectAccount)?;
//     }

//     let mut can_thaw_instruction = can_thaw_permissionless(
//         program_id,
//         signer_pubkey,
//         token_account_pubkey,
//         mint_pubkey,
//     );
//     can_thaw_instruction
//         .accounts
//         .push(AccountMeta::new_readonly(validate_state_pubkey, false));

//     ExtraAccountMetaList::add_to_instruction::<CanThawPermissionlessInstruction, _, _>(
//         &mut can_thaw_instruction,
//         fetch_account_data_fn,
//         &validate_state_data,
//     )
//     .await?;

//     // Add only the extra accounts resolved from the validation state
//     instruction
//         .accounts
//         .extend_from_slice(&can_thaw_instruction.accounts[3..]);

//     // Add the program id and validation state account
//     instruction
//         .accounts
//         .push(AccountMeta::new_readonly(*program_id, false));
//     instruction
//         .accounts
//         .push(AccountMeta::new_readonly(validate_state_pubkey, false));

//     Ok(())
// }

#[allow(clippy::too_many_arguments)]
pub async fn add_extra_account_metas_for_freeze<F, Fut>(
    instruction: &mut Instruction,
    program_id: &Pubkey,
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_account_owner: &Pubkey,
    fetch_account_data_fn: F,
) -> Result<(), AccountFetchError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    let extra_metas_pubkey = get_freeze_extra_account_metas_address(mint_pubkey, program_id);

    add_extra_account_metas_for_permissionless_ix::<_, _, CanFreezePermissionlessInstruction, _>(
        instruction,
        program_id,
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        token_account_owner,
        &extra_metas_pubkey,
        fetch_account_data_fn,
        |program_id, signer_pubkey, token_account_pubkey, mint_pubkey, token_account_owner| {
            can_freeze_permissionless(
                program_id,
                signer_pubkey,
                token_account_pubkey,
                mint_pubkey,
                token_account_owner,
            )
        },
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn add_extra_account_metas_for_thaw<F, Fut>(
    instruction: &mut Instruction,
    program_id: &Pubkey,
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_account_owner: &Pubkey,
    fetch_account_data_fn: F,
) -> Result<(), AccountFetchError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    let extra_metas_pubkey = get_thaw_extra_account_metas_address(mint_pubkey, program_id);

    add_extra_account_metas_for_permissionless_ix::<_, _, CanThawPermissionlessInstruction, _>(
        instruction,
        program_id,
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        token_account_owner,
        &extra_metas_pubkey,
        fetch_account_data_fn,
        |program_id, signer_pubkey, token_account_pubkey, mint_pubkey, token_account_owner| {
            can_thaw_permissionless(
                program_id,
                signer_pubkey,
                token_account_pubkey,
                mint_pubkey,
                token_account_owner,
            )
        },
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn add_extra_account_metas_for_permissionless_ix<F, Fut, T, F2>(
    instruction: &mut Instruction,
    program_id: &Pubkey,
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_account_owner: &Pubkey,
    extra_metas_pubkey: &Pubkey,
    fetch_account_data_fn: F,
    cpi_ix_builder_fn: F2,
) -> Result<(), AccountFetchError>
where
    F: Fn(Pubkey) -> Fut,
    F2: Fn(&Pubkey, &Pubkey, &Pubkey, &Pubkey, &Pubkey) -> Instruction,
    Fut: Future<Output = AccountDataResult>,
    T: SplDiscriminate,
{
    //let validate_state_pubkey = get_thaw_extra_account_metas_address(mint_pubkey, program_id);
    let validate_state_data = fetch_account_data_fn(*extra_metas_pubkey)
        .await?
        .ok_or(ProgramError::InvalidAccountData)?;

    // Check to make sure the provided keys are in the instruction
    if [
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        token_account_owner,
    ]
    .iter()
    .any(|&key| !instruction.accounts.iter().any(|meta| meta.pubkey == *key))
    {
        Err(ThawFreezeGateError::IncorrectAccount)?;
    }

    let mut cpi_ix = cpi_ix_builder_fn(
        program_id,
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        token_account_owner,
    );
    cpi_ix
        .accounts
        .push(AccountMeta::new_readonly(*extra_metas_pubkey, false));

    ExtraAccountMetaList::add_to_instruction::<T, _, _>(
        &mut cpi_ix,
        fetch_account_data_fn,
        &validate_state_data,
    )
    .await?;

    // Add only the extra accounts resolved from the validation state
    instruction
        .accounts
        .extend_from_slice(&cpi_ix.accounts[3..]);

    // Add the program id and validation state account
    instruction
        .accounts
        .push(AccountMeta::new_readonly(*program_id, false));
    instruction
        .accounts
        .push(AccountMeta::new_readonly(*extra_metas_pubkey, false));

    Ok(())
}
