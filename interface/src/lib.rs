use solana_pubkey::Pubkey;

pub mod error;
pub mod instruction;
pub mod offchain;
pub mod onchain;

pub const FREEZE_EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"freeze_extra_account_metas";
pub const THAW_EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"thaw_extra_account_metas";

pub fn collect_thaw_extra_account_metas(mint: &Pubkey) -> [&[u8]; 2] {
    [THAW_EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()]
}

pub fn collect_freeze_extra_account_metas(mint: &Pubkey) -> [&[u8]; 2] {
    [FREEZE_EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()]
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
