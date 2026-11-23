use solana_account_info::AccountInfo;
use solana_cpi::invoke;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramResult;
use solana_pubkey::Pubkey;
use spl_discriminator::SplDiscriminate;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

use crate::{
    get_freeze_extra_account_metas_address, get_thaw_extra_account_metas_address, instruction,
};

/// CPIs into the gating program's `can-thaw-permissionless` instruction.
///
/// Equivalent to [`invoke_can_thaw_permissionless_with_data`] with an empty payload.
pub fn invoke_can_thaw_permissionless<'a>(
    program_id: &Pubkey,
    signer: AccountInfo<'a>,
    token_account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    token_account_owner: AccountInfo<'a>,
    flag_account: AccountInfo<'a>,
    additional_accounts: &[AccountInfo<'a>],
) -> ProgramResult {
    invoke_can_thaw_permissionless_with_data(
        program_id,
        signer,
        token_account,
        mint,
        token_account_owner,
        flag_account,
        additional_accounts,
        &[],
    )
}

/// CPIs into the gating program's `can-thaw-permissionless` instruction,
/// forwarding `data` verbatim after the 8-byte discriminator.
///
/// Extra account metas are resolved against the instruction that already
/// contains `data`, so `Seed::InstructionData` seeds can reference the payload.
#[allow(clippy::too_many_arguments)]
pub fn invoke_can_thaw_permissionless_with_data<'a>(
    program_id: &Pubkey,
    signer: AccountInfo<'a>,
    token_account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    token_account_owner: AccountInfo<'a>,
    flag_account: AccountInfo<'a>,
    additional_accounts: &[AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    let instruction = instruction::can_thaw_permissionless_with_data(
        program_id,
        signer.key,
        token_account.key,
        mint.key,
        token_account_owner.key,
        flag_account.key,
        data,
    );
    let validation_pubkey = get_thaw_extra_account_metas_address(mint.key, program_id);

    invoke_with_extra_metas::<instruction::CanThawPermissionlessInstruction>(
        instruction,
        validation_pubkey,
        vec![
            signer,
            token_account,
            mint,
            token_account_owner,
            flag_account,
        ],
        additional_accounts,
    )
}

/// CPIs into the gating program's `can-freeze-permissionless` instruction.
///
/// Equivalent to [`invoke_can_freeze_permissionless_with_data`] with an empty payload.
pub fn invoke_can_freeze_permissionless<'a>(
    program_id: &Pubkey,
    signer: AccountInfo<'a>,
    token_account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    token_account_owner: AccountInfo<'a>,
    flag_account: AccountInfo<'a>,
    additional_accounts: &[AccountInfo<'a>],
) -> ProgramResult {
    invoke_can_freeze_permissionless_with_data(
        program_id,
        signer,
        token_account,
        mint,
        token_account_owner,
        flag_account,
        additional_accounts,
        &[],
    )
}

/// CPIs into the gating program's `can-freeze-permissionless` instruction,
/// forwarding `data` verbatim after the 8-byte discriminator.
///
/// Extra account metas are resolved against the instruction that already
/// contains `data`, so `Seed::InstructionData` seeds can reference the payload.
#[allow(clippy::too_many_arguments)]
pub fn invoke_can_freeze_permissionless_with_data<'a>(
    program_id: &Pubkey,
    signer: AccountInfo<'a>,
    token_account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    token_account_owner: AccountInfo<'a>,
    flag_account: AccountInfo<'a>,
    additional_accounts: &[AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    let instruction = instruction::can_freeze_permissionless_with_data(
        program_id,
        signer.key,
        token_account.key,
        mint.key,
        token_account_owner.key,
        flag_account.key,
        data,
    );
    let validation_pubkey = get_freeze_extra_account_metas_address(mint.key, program_id);

    invoke_with_extra_metas::<instruction::CanFreezePermissionlessInstruction>(
        instruction,
        validation_pubkey,
        vec![
            signer,
            token_account,
            mint,
            token_account_owner,
            flag_account,
        ],
        additional_accounts,
    )
}

fn invoke_with_extra_metas<'a, T: SplDiscriminate>(
    mut instruction: Instruction,
    validation_pubkey: Pubkey,
    mut cpi_account_infos: Vec<AccountInfo<'a>>,
    additional_accounts: &[AccountInfo<'a>],
) -> ProgramResult {
    if let Some(validation_info) = additional_accounts
        .iter()
        .find(|&x| *x.key == validation_pubkey)
    {
        instruction
            .accounts
            .push(AccountMeta::new_readonly(validation_pubkey, false));
        cpi_account_infos.push(validation_info.clone());

        // `instruction.data` already holds the forwarded payload here, so any
        // `Seed::InstructionData` in the extra metas resolves against it.
        ExtraAccountMetaList::add_to_cpi_instruction::<T>(
            &mut instruction,
            &mut cpi_account_infos,
            &validation_info.try_borrow_data()?,
            additional_accounts,
        )?;
    }

    invoke(&instruction, &cpi_account_infos)
}
