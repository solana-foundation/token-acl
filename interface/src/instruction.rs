use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};

pub enum EfficientBlockAllowInstruction {
    CanThawPermissionless,
    CanFreezePermissionless,
}

#[derive(SplDiscriminate)]
#[discriminator_hash_input("efficient-allow-block-list-standard:can-thaw-permissionless")]
pub struct CanThawPermissionlessInstruction;

#[derive(SplDiscriminate)]
#[discriminator_hash_input("efficient-allow-block-list-standard:can-freeze-permissionless")]
pub struct CanFreezePermissionlessInstruction;

impl EfficientBlockAllowInstruction {
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < ArrayDiscriminator::LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }
        let (discriminator, _) = data.split_at(ArrayDiscriminator::LENGTH);
        match discriminator {
            CanThawPermissionlessInstruction::SPL_DISCRIMINATOR_SLICE => {
                Ok(Self::CanThawPermissionless)
            }
            CanFreezePermissionlessInstruction::SPL_DISCRIMINATOR_SLICE => {
                Ok(Self::CanFreezePermissionless)
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        match self {
            Self::CanThawPermissionless => {
                CanThawPermissionlessInstruction::SPL_DISCRIMINATOR_SLICE.to_vec()
            }
            Self::CanFreezePermissionless => {
                CanFreezePermissionlessInstruction::SPL_DISCRIMINATOR_SLICE.to_vec()
            }
        }
    }
}

/// Builds the sRFC37 `can-thaw-permissionless` instruction for a gating program.
///
/// Equivalent to [`can_thaw_permissionless_with_data`] with an empty payload.
pub fn can_thaw_permissionless(
    program_id: &Pubkey,
    signer: &Pubkey,
    token_account: &Pubkey,
    mint: &Pubkey,
    token_account_owner: &Pubkey,
    flag_account: &Pubkey,
) -> Instruction {
    can_thaw_permissionless_with_data(
        program_id,
        signer,
        token_account,
        mint,
        token_account_owner,
        flag_account,
        &[],
    )
}

/// Builds the sRFC37 `can-thaw-permissionless` instruction for a gating program,
/// appending `data` verbatim after the 8-byte discriminator.
///
/// `data` is untrusted caller input that Token ACL forwards without
/// interpreting it. An empty `data` produces exactly the same instruction as
/// [`can_thaw_permissionless`].
pub fn can_thaw_permissionless_with_data(
    program_id: &Pubkey,
    signer: &Pubkey,
    token_account: &Pubkey,
    mint: &Pubkey,
    token_account_owner: &Pubkey,
    flag_account: &Pubkey,
    data: &[u8],
) -> Instruction {
    permissionless_instruction(
        EfficientBlockAllowInstruction::CanThawPermissionless,
        program_id,
        signer,
        token_account,
        mint,
        token_account_owner,
        flag_account,
        data,
    )
}

/// Builds the sRFC37 `can-freeze-permissionless` instruction for a gating program.
///
/// Equivalent to [`can_freeze_permissionless_with_data`] with an empty payload.
pub fn can_freeze_permissionless(
    program_id: &Pubkey,
    signer: &Pubkey,
    token_account: &Pubkey,
    mint: &Pubkey,
    token_account_owner: &Pubkey,
    flag_account: &Pubkey,
) -> Instruction {
    can_freeze_permissionless_with_data(
        program_id,
        signer,
        token_account,
        mint,
        token_account_owner,
        flag_account,
        &[],
    )
}

/// Builds the sRFC37 `can-freeze-permissionless` instruction for a gating program,
/// appending `data` verbatim after the 8-byte discriminator.
///
/// `data` is untrusted caller input that Token ACL forwards without
/// interpreting it. An empty `data` produces exactly the same instruction as
/// [`can_freeze_permissionless`].
pub fn can_freeze_permissionless_with_data(
    program_id: &Pubkey,
    signer: &Pubkey,
    token_account: &Pubkey,
    mint: &Pubkey,
    token_account_owner: &Pubkey,
    flag_account: &Pubkey,
    data: &[u8],
) -> Instruction {
    permissionless_instruction(
        EfficientBlockAllowInstruction::CanFreezePermissionless,
        program_id,
        signer,
        token_account,
        mint,
        token_account_owner,
        flag_account,
        data,
    )
}

#[allow(clippy::too_many_arguments)]
fn permissionless_instruction(
    kind: EfficientBlockAllowInstruction,
    program_id: &Pubkey,
    signer: &Pubkey,
    token_account: &Pubkey,
    mint: &Pubkey,
    token_account_owner: &Pubkey,
    flag_account: &Pubkey,
    payload: &[u8],
) -> Instruction {
    let mut data = Vec::with_capacity(ArrayDiscriminator::LENGTH.saturating_add(payload.len()));
    data.extend_from_slice(&kind.pack());
    data.extend_from_slice(payload);

    let accounts = vec![
        AccountMeta::new_readonly(*signer, false),
        AccountMeta::new_readonly(*token_account, false),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new_readonly(*token_account_owner, false),
        AccountMeta::new_readonly(*flag_account, false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_payload_is_byte_identical_to_legacy_builders() {
        let keys: Vec<Pubkey> = (0..6).map(|_| Pubkey::new_unique()).collect();
        let thaw =
            can_thaw_permissionless(&keys[0], &keys[1], &keys[2], &keys[3], &keys[4], &keys[5]);
        assert_eq!(
            thaw.data,
            CanThawPermissionlessInstruction::SPL_DISCRIMINATOR_SLICE
        );
        assert_eq!(
            thaw,
            can_thaw_permissionless_with_data(
                &keys[0],
                &keys[1],
                &keys[2],
                &keys[3],
                &keys[4],
                &keys[5],
                &[]
            )
        );
        let freeze =
            can_freeze_permissionless(&keys[0], &keys[1], &keys[2], &keys[3], &keys[4], &keys[5]);
        assert_eq!(
            freeze.data,
            CanFreezePermissionlessInstruction::SPL_DISCRIMINATOR_SLICE
        );
        assert_eq!(
            freeze,
            can_freeze_permissionless_with_data(
                &keys[0],
                &keys[1],
                &keys[2],
                &keys[3],
                &keys[4],
                &keys[5],
                &[]
            )
        );
    }

    #[test]
    fn discriminators_match_published_values() {
        // Mirrored in clients/js/src/helpers.ts for off-chain extra-meta resolution.
        assert_eq!(
            CanThawPermissionlessInstruction::SPL_DISCRIMINATOR_SLICE,
            &[8, 175, 169, 129, 137, 74, 61, 241]
        );
        assert_eq!(
            CanFreezePermissionlessInstruction::SPL_DISCRIMINATOR_SLICE,
            &[214, 141, 109, 75, 248, 1, 45, 29]
        );
    }

    #[test]
    fn payload_is_appended_after_discriminator() {
        let keys: Vec<Pubkey> = (0..6).map(|_| Pubkey::new_unique()).collect();
        let payload = [9u8, 8, 7, 6, 5];
        let thaw = can_thaw_permissionless_with_data(
            &keys[0], &keys[1], &keys[2], &keys[3], &keys[4], &keys[5], &payload,
        );
        assert_eq!(
            &thaw.data[..ArrayDiscriminator::LENGTH],
            CanThawPermissionlessInstruction::SPL_DISCRIMINATOR_SLICE
        );
        assert_eq!(&thaw.data[ArrayDiscriminator::LENGTH..], &payload);
        assert!(matches!(
            EfficientBlockAllowInstruction::unpack(&thaw.data),
            Ok(EfficientBlockAllowInstruction::CanThawPermissionless)
        ));

        let freeze = can_freeze_permissionless_with_data(
            &keys[0], &keys[1], &keys[2], &keys[3], &keys[4], &keys[5], &payload,
        );
        assert_eq!(
            &freeze.data[..ArrayDiscriminator::LENGTH],
            CanFreezePermissionlessInstruction::SPL_DISCRIMINATOR_SLICE
        );
        assert_eq!(&freeze.data[ArrayDiscriminator::LENGTH..], &payload);
    }
}
