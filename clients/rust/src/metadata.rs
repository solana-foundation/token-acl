use std::str::FromStr;

use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use spl_token_client::spl_token_2022::{
    extension::{BaseStateWithExtensions, PodStateWithExtensions},
    pod::PodMint,
};
use spl_token_metadata_interface::state::{Field, TokenMetadata};
use token_acl_interface::error::ThawFreezeGateError;

pub const TOKEN_ACL_METADATA_KEY: &str = "token_acl";

pub fn set_mint_tacl_metadata_ix(
    mint: &Pubkey,
    metadata_authority: &Pubkey,
    gating_program: &Pubkey,
) -> Instruction {
    spl_token_metadata_interface::instruction::update_field(
        &spl_token_client::spl_token_2022::ID,
        mint,
        metadata_authority,
        Field::Key(TOKEN_ACL_METADATA_KEY.to_string()),
        gating_program.to_string(),
    )
}

pub fn get_gating_program_from_mint_data(data: &[u8]) -> Result<Pubkey, ThawFreezeGateError> {
    let mint = PodStateWithExtensions::<PodMint>::unpack(data)
        .map_err(|_| ThawFreezeGateError::InvalidTokenMint)?;

    let metadata = mint
        .get_variable_len_extension::<TokenMetadata>()
        .map_err(|_| ThawFreezeGateError::InvalidTokenMint)?;

    let gating_program = metadata
        .additional_metadata
        .iter()
        .find(|(key, _)| key == TOKEN_ACL_METADATA_KEY)
        .map(|(_, val)| val)
        .ok_or(ThawFreezeGateError::InvalidTokenMint)?;

    Pubkey::from_str(&gating_program).map_err(|_| ThawFreezeGateError::InvalidTokenMint)
}

#[cfg(feature = "fetch")]
pub fn get_gating_program_from_mint(
    rpc: &solana_client::rpc_client::RpcClient,
    mint_pubkey: &Pubkey,
) -> Result<Pubkey, ThawFreezeGateError> {
    let account = rpc
        .get_account(mint_pubkey)
        .map_err(|_| ThawFreezeGateError::InvalidTokenMint)?;

    get_gating_program_from_mint_data(&account.data)
}
