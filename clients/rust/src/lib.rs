mod generated;
mod metadata;
mod permissionless_data;
use std::future::Future;

pub use generated::*;
pub use metadata::*;
pub use permissionless_data::*;

#[cfg(feature = "fetch")]
use solana_client::nonblocking;
use solana_instruction::Instruction;
use solana_program_error::ProgramError;
use solana_program_option::COption;
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use spl_associated_token_account_interface::instruction::{
    create_associated_token_account, create_associated_token_account_idempotent,
};
pub use spl_tlv_account_resolution::state::{AccountDataResult, AccountFetchError};
use spl_token_2022_interface::state::{Account, AccountState};
use spl_token_2022_interface::ID as SPL_TOKEN_2022_ID;

use crate::generated::errors::token_acl::TokenAclError;

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
    create_thaw_permissionless_instruction_with_extra_metas_and_data(
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        mint_config_pubkey,
        token_program_pubkey,
        token_account_owner_pubkey,
        idempotent,
        &[],
        fetch_account_data_fn,
    )
    .await
}

/// Like [`create_thaw_permissionless_instruction_with_extra_metas`], but
/// appends `data` to the Token ACL instruction (forwarded verbatim to the
/// gating program) and resolves the gate's extra account metas against the
/// gate instruction containing that payload.
#[allow(clippy::too_many_arguments)]
pub async fn create_thaw_permissionless_instruction_with_extra_metas_and_data<F, Fut>(
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_config_pubkey: &Pubkey,
    token_program_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    idempotent: bool,
    data: &[u8],
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

    let flag_account = crate::accounts::FlagAccount::find_pda(token_account_pubkey).0;

    if !mint_config.enable_permissionless_thaw {
        return Err(TokenAclError::PermissionlessThawNotEnabled.into());
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
            .flag_account(flag_account)
            .system_program(solana_system_interface::program::ID)
            .instruction_with_data(data)
    } else {
        crate::instructions::ThawPermissionlessBuilder::new()
            .gating_program(mint_config.gating_program)
            .authority(*signer_pubkey)
            .mint(*mint_pubkey)
            .token_account(*token_account_pubkey)
            .token_account_owner(*token_account_owner_pubkey)
            .mint_config(*mint_config_pubkey)
            .token_program(*token_program_pubkey)
            .flag_account(flag_account)
            .system_program(solana_system_interface::program::ID)
            .instruction_with_data(data)
    };

    if mint_config.gating_program != Pubkey::default() {
        token_acl_interface::offchain::add_extra_account_metas_for_thaw_with_data(
            &mut ix,
            &mint_config.gating_program,
            signer_pubkey,
            token_account_pubkey,
            mint_pubkey,
            token_account_owner_pubkey,
            &flag_account,
            data,
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
    create_freeze_permissionless_instruction_with_extra_metas_and_data(
        signer_pubkey,
        token_account_pubkey,
        mint_pubkey,
        mint_config_pubkey,
        token_program_pubkey,
        token_account_owner_pubkey,
        idempotent,
        &[],
        fetch_account_data_fn,
    )
    .await
}

/// Like [`create_freeze_permissionless_instruction_with_extra_metas`], but
/// appends `data` to the Token ACL instruction (forwarded verbatim to the
/// gating program) and resolves the gate's extra account metas against the
/// gate instruction containing that payload.
#[allow(clippy::too_many_arguments)]
pub async fn create_freeze_permissionless_instruction_with_extra_metas_and_data<F, Fut>(
    signer_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_config_pubkey: &Pubkey,
    token_program_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    idempotent: bool,
    data: &[u8],
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
        return Err(TokenAclError::PermissionlessFreezeNotEnabled.into());
    }

    let flag_account = crate::accounts::FlagAccount::find_pda(&token_account_pubkey).0;

    let mut ix = if idempotent {
        crate::instructions::FreezePermissionlessIdempotentBuilder::new()
            .gating_program(mint_config.gating_program)
            .authority(*signer_pubkey)
            .mint(*mint_pubkey)
            .token_account(*token_account_pubkey)
            .token_account_owner(*token_account_owner_pubkey)
            .mint_config(*mint_config_pubkey)
            .token_program(*token_program_pubkey)
            .system_program(solana_system_interface::program::ID)
            .flag_account(flag_account)
            .instruction_with_data(data)
    } else {
        crate::instructions::FreezePermissionlessBuilder::new()
            .gating_program(mint_config.gating_program)
            .authority(*signer_pubkey)
            .mint(*mint_pubkey)
            .token_account(*token_account_pubkey)
            .token_account_owner(*token_account_owner_pubkey)
            .mint_config(*mint_config_pubkey)
            .token_program(*token_program_pubkey)
            .system_program(solana_system_interface::program::ID)
            .flag_account(flag_account)
            .instruction_with_data(data)
    };

    if mint_config.gating_program != Pubkey::default() {
        token_acl_interface::offchain::add_extra_account_metas_for_freeze_with_data(
            &mut ix,
            &mint_config.gating_program,
            signer_pubkey,
            token_account_pubkey,
            mint_pubkey,
            token_account_owner_pubkey,
            &flag_account,
            data,
            fetch_account_data_fn,
        )
        .await?;
    }

    Ok(ix)
}

#[cfg(feature = "fetch")]
pub async fn create_ata_and_thaw_permissionless(
    rpc: &nonblocking::rpc_client::RpcClient,
    payer_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    idempotent: bool,
) -> Result<Vec<Instruction>, AccountFetchError> {
    create_ata_and_thaw_permissionless_with_data(
        rpc,
        payer_pubkey,
        mint_pubkey,
        token_account_owner_pubkey,
        idempotent,
        &[],
    )
    .await
}

/// Like [`create_ata_and_thaw_permissionless`], forwarding `data` to the gating program.
#[cfg(feature = "fetch")]
pub async fn create_ata_and_thaw_permissionless_with_data(
    rpc: &nonblocking::rpc_client::RpcClient,
    payer_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    idempotent: bool,
    data: &[u8],
) -> Result<Vec<Instruction>, AccountFetchError> {
    let fetch_account_data_fn = |pubkey: Pubkey| async move {
        rpc.get_account_data(&pubkey)
            .await
            .map(|data| Some(data.to_vec()))
            .map_err(Into::<AccountFetchError>::into)
    };

    create_ata_and_thaw_permissionless_instructions_with_data(
        payer_pubkey,
        mint_pubkey,
        &SPL_TOKEN_2022_ID,
        token_account_owner_pubkey,
        idempotent,
        data,
        &fetch_account_data_fn,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn create_ata_and_thaw_permissionless_instructions<F, Fut>(
    payer_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_program_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    idempotent: bool,
    fetch_account_data_fn: &F,
) -> Result<Vec<Instruction>, AccountFetchError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    create_ata_and_thaw_permissionless_instructions_with_data(
        payer_pubkey,
        mint_pubkey,
        token_program_pubkey,
        token_account_owner_pubkey,
        idempotent,
        &[],
        fetch_account_data_fn,
    )
    .await
}

/// Like [`create_ata_and_thaw_permissionless_instructions`], forwarding `data`
/// to the gating program and resolving extra metas against it.
#[allow(clippy::too_many_arguments)]
pub async fn create_ata_and_thaw_permissionless_instructions_with_data<F, Fut>(
    payer_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_program_pubkey: &Pubkey,
    token_account_owner_pubkey: &Pubkey,
    idempotent: bool,
    data: &[u8],
    fetch_account_data_fn: &F,
) -> Result<Vec<Instruction>, AccountFetchError>
where
    F: Fn(Pubkey) -> Fut,
    Fut: Future<Output = AccountDataResult>,
{
    let token_account = get_associated_token_address_with_program_id(
        &token_account_owner_pubkey,
        &mint_pubkey,
        &SPL_TOKEN_2022_ID,
    );

    let ix = if idempotent {
        create_associated_token_account_idempotent(
            &payer_pubkey,
            &token_account_owner_pubkey,
            &mint_pubkey,
            &SPL_TOKEN_2022_ID,
        )
    } else {
        create_associated_token_account(
            &payer_pubkey,
            &token_account_owner_pubkey,
            &mint_pubkey,
            &SPL_TOKEN_2022_ID,
        )
    };
    let mut instructions = vec![ix];

    // assume account doesn't exist, so we mock it
    let acc = Account {
        mint: *mint_pubkey,
        owner: *token_account_owner_pubkey,
        amount: 0,
        delegate: COption::None,
        state: AccountState::Frozen,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let mut mock_token_account_data = vec![0u8; Account::LEN];
    Account::pack(acc, &mut mock_token_account_data)?;

    let mint_data = fetch_account_data_fn(*mint_pubkey)
        .await?
        .ok_or(Into::<ProgramError>::into(TokenAclError::InvalidTokenMint))?;

    let mint_config_pubkey = crate::accounts::MintConfig::find_pda(mint_pubkey).0;
    let gating_program = get_gating_program_from_mint_data(&mint_data);
    let flag_account = crate::accounts::FlagAccount::find_pda(&token_account).0;

    if let Ok(gating_program) = gating_program {
        let mut ix = if idempotent {
            crate::instructions::ThawPermissionlessIdempotentBuilder::new()
                .gating_program(gating_program)
                .authority(*payer_pubkey)
                .mint(*mint_pubkey)
                .token_account(token_account)
                .token_account_owner(*token_account_owner_pubkey)
                .mint_config(mint_config_pubkey)
                .token_program(*token_program_pubkey)
                .flag_account(flag_account)
                .system_program(solana_system_interface::program::ID)
                .instruction_with_data(data)
        } else {
            crate::instructions::ThawPermissionlessBuilder::new()
                .gating_program(gating_program)
                .authority(*payer_pubkey)
                .mint(*mint_pubkey)
                .token_account(token_account)
                .token_account_owner(*token_account_owner_pubkey)
                .mint_config(mint_config_pubkey)
                .token_program(*token_program_pubkey)
                .flag_account(flag_account)
                .system_program(solana_system_interface::program::ID)
                .instruction_with_data(data)
        };

        token_acl_interface::offchain::add_extra_account_metas_for_thaw_with_data(
            &mut ix,
            &gating_program,
            payer_pubkey,
            &token_account,
            mint_pubkey,
            token_account_owner_pubkey,
            &flag_account,
            data,
            |pubkey| {
                let mock_token_account_data = mock_token_account_data.clone();
                async move {
                    if pubkey == token_account {
                        return Ok(Some(mock_token_account_data));
                    }
                    let data = fetch_account_data_fn(pubkey).await.unwrap_or(None);
                    Ok(data)
                }
            },
        )
        .await?;

        instructions.push(ix);
    }

    Ok(instructions)
}
