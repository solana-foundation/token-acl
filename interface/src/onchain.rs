use solana_account_info::AccountInfo;
use solana_cpi::invoke;
use solana_instruction::AccountMeta;
use solana_program_error::ProgramResult;
use solana_pubkey::Pubkey;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

use crate::{get_freeze_extra_account_metas_address, get_thaw_extra_account_metas_address, instruction};



pub fn invoke_can_thaw_permissionless<'a>(
    program_id: &Pubkey,
    signer: AccountInfo<'a>,
    token_account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    token_account_owner: AccountInfo<'a>,
    additional_accounts: &[AccountInfo<'a>],
) -> ProgramResult {
    let mut instruction = instruction::can_thaw_permissionless(program_id, &signer.key, &token_account.key, &mint.key, &token_account_owner.key);
    
    let validation_pubkey = get_thaw_extra_account_metas_address(mint.key, program_id);

    let mut cpi_account_infos = vec![signer, token_account, mint, token_account_owner];

    if let Some(validation_info) = additional_accounts
    .iter()
    .find(|&x| *x.key == validation_pubkey)
    {
        instruction
            .accounts
            .push(AccountMeta::new_readonly(validation_pubkey, false));
        cpi_account_infos.push(validation_info.clone());

        ExtraAccountMetaList::add_to_cpi_instruction::<instruction::CanThawPermissionlessInstruction>(
            &mut instruction,
            &mut cpi_account_infos,
            &validation_info.try_borrow_data()?,
            additional_accounts,
        )?;
    }

    invoke(
        &instruction,
        &cpi_account_infos,
    )
}

pub fn invoke_can_freeze_permissionless<'a>(
    program_id: &Pubkey,
    signer: AccountInfo<'a>,
    token_account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    token_account_owner: AccountInfo<'a>,
    additional_accounts: &[AccountInfo<'a>],
) -> ProgramResult {
    let mut instruction = instruction::can_freeze_permissionless(program_id, &signer.key, &token_account.key, &mint.key, &token_account_owner.key);
    
    let validation_pubkey = get_freeze_extra_account_metas_address(mint.key, program_id);    
    let mut cpi_account_infos = vec![signer, token_account, mint, token_account_owner];

    if let Some(validation_info) = additional_accounts
    .iter()
    .find(|&x| *x.key == validation_pubkey)
    {
        instruction
            .accounts
            .push(AccountMeta::new_readonly(validation_pubkey, false));
        cpi_account_infos.push(validation_info.clone());

        ExtraAccountMetaList::add_to_cpi_instruction::<instruction::CanFreezePermissionlessInstruction>(
            &mut instruction,
            &mut cpi_account_infos,
            &validation_info.try_borrow_data()?,
            additional_accounts,
        )?;
    }

    invoke(
        &instruction,
        &cpi_account_infos,
    )
}