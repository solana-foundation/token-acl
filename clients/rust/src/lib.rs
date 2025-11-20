mod generated;
mod metadata;
use std::future::Future;

pub use generated::*;
pub use metadata::*;

#[cfg(feature = "fetch")]
use solana_client::nonblocking;
use solana_instruction::Instruction;
use solana_program_error::ProgramError;
use solana_program_option::COption;
use solana_pubkey::Pubkey;
use solana_sdk::program_pack::Pack;
use spl_associated_token_account_client::address::get_associated_token_address_with_program_id;
use spl_associated_token_account_client::instruction::{
    create_associated_token_account, create_associated_token_account_idempotent,
};
pub use spl_tlv_account_resolution::state::{AccountDataResult, AccountFetchError};
use spl_token_client::spl_token_2022;
use spl_token_client::spl_token_2022::state::{Account, AccountState};

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
            .flag_account(flag_account)
            .system_program(solana_system_interface::program::ID)
            .instruction()
    };

    if mint_config.gating_program != Pubkey::default() {
        token_acl_interface::offchain::add_extra_account_metas_for_thaw(
            &mut ix,
            &mint_config.gating_program,
            signer_pubkey,
            token_account_pubkey,
            mint_pubkey,
            token_account_owner_pubkey,
            &flag_account,
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
            .system_program(solana_system_interface::program::ID)
            .flag_account(flag_account)
            .instruction()
    };

    if mint_config.gating_program != Pubkey::default() {
        token_acl_interface::offchain::add_extra_account_metas_for_freeze(
            &mut ix,
            &mint_config.gating_program,
            signer_pubkey,
            token_account_pubkey,
            mint_pubkey,
            token_account_owner_pubkey,
            &flag_account,
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
    let fetch_account_data_fn = |pubkey: Pubkey| async move {
        rpc.get_account_data(&pubkey).await.map(|data| Some(data.to_vec())).map_err(Into::<AccountFetchError>::into)
    };
    
    create_ata_and_thaw_permissionless_instructions(
        payer_pubkey,
        mint_pubkey,
        &spl_token_2022::ID,
        token_account_owner_pubkey,
        idempotent,
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
    let token_account = get_associated_token_address_with_program_id(
        &token_account_owner_pubkey,
        &mint_pubkey,
        &spl_token_2022::ID,
    );

    let ix = if idempotent {
        create_associated_token_account_idempotent(
            &payer_pubkey,
            &token_account_owner_pubkey,
            &mint_pubkey,
            &spl_token_2022::ID,
        )
    } else {
        create_associated_token_account(
            &payer_pubkey,
            &token_account_owner_pubkey,
            &mint_pubkey,
            &spl_token_2022::ID,
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

    let mut data = vec![0u8; Account::LEN];
    Account::pack(acc, &mut data)?;

    let mint_data = fetch_account_data_fn(*mint_pubkey).await?.ok_or(Into::<ProgramError>::into(TokenAclError::InvalidTokenMint))?;

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
            .instruction()
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
            .instruction()
    };

        token_acl_interface::offchain::add_extra_account_metas_for_thaw(
            &mut ix,
            &gating_program,
            payer_pubkey,
            &token_account,
            mint_pubkey,
            token_account_owner_pubkey,
            &flag_account,
            |pubkey| {
                let data = data.clone();
                async move {
                    if pubkey == token_account {
                        return Ok(Some(data));
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
