use solana_pubkey::{pubkey, Pubkey};

pub mod error;
pub mod instruction;
pub mod offchain;
pub mod onchain;

pub const FREEZE_EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"freeze_extra_account_metas";
pub const THAW_EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"thaw_extra_account_metas";
pub const FLAG_ACCOUNT_SEED: &[u8] = b"FLAG_ACCOUNT";

pub const TOKEN_ACL_ID: Pubkey = pubkey!("TACLkU6CiCdkQN2MjoyDkVg2yAH9zkxiHDsiztQ52TP");

pub fn collect_thaw_extra_account_metas(mint: &Pubkey) -> [&[u8]; 2] {
    [THAW_EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()]
}

pub fn collect_freeze_extra_account_metas(mint: &Pubkey) -> [&[u8]; 2] {
    [FREEZE_EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()]
}

pub fn collect_flag_account(token_account: &Pubkey) -> [&[u8]; 2] {
    [FLAG_ACCOUNT_SEED, token_account.as_ref()]
}

pub fn get_thaw_extra_account_metas_address_and_bump_seed(
    mint: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&collect_thaw_extra_account_metas(mint), program_id)
}

pub fn get_freeze_extra_account_metas_address_and_bump_seed(
    mint: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&collect_freeze_extra_account_metas(mint), program_id)
}

pub fn get_thaw_extra_account_metas_address(mint: &Pubkey, program_id: &Pubkey) -> Pubkey {
    get_thaw_extra_account_metas_address_and_bump_seed(mint, program_id).0
}

pub fn get_freeze_extra_account_metas_address(mint: &Pubkey, program_id: &Pubkey) -> Pubkey {
    get_freeze_extra_account_metas_address_and_bump_seed(mint, program_id).0
}

pub fn get_flag_account_address(token_account: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&collect_flag_account(token_account), program_id).0
}
