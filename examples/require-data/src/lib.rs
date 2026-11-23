//! Example sRFC37 gating program that only allows a permissionless thaw/freeze
//! when the caller forwards a specific payload through Token ACL.
//!
//! It exercises both halves of payload forwarding:
//! - the payload after the 8-byte sRFC37 discriminator must equal
//!   [`EXPECTED_PAYLOAD`], and
//! - the extra account meta list declares a PDA derived from the payload with
//!   a `Seed::InstructionData` seed, so extra-meta resolution (on-chain in
//!   Token ACL and off-chain in the clients) must run on the instruction that
//!   already contains the payload.
//!
//! A real gate would verify e.g. a zero-knowledge proof here instead; the
//! payload is untrusted caller input either way.

use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};
use solana_program_error::ProgramError;
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};
use token_acl_interface::instruction::{
    CanFreezePermissionlessInstruction, CanThawPermissionlessInstruction,
};

pub mod instructions;
pub use instructions::*;

declare_id!("Eba1ts11111111111111111111111111111111111115");

/// The only payload this gate accepts.
pub const EXPECTED_PAYLOAD: [u8; 32] = *b"sRFC37 passthrough test payload!";

/// Seed prefix of the payload-derived PDA declared as an extra account meta.
pub const PAYLOAD_SEED_PREFIX: &[u8] = b"payload";

/// Returns the PDA the extra account meta resolves to for `payload`.
pub fn find_payload_address(payload: &[u8]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PAYLOAD_SEED_PREFIX, payload], &ID)
}

entrypoint!(process_instruction);
fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &'a [u8],
) -> ProgramResult {
    if instruction_data.len() < ArrayDiscriminator::LENGTH {
        return Err(CustomErrors::InvalidInstruction.into());
    }
    let (discriminator, payload) = instruction_data.split_at(ArrayDiscriminator::LENGTH);

    match discriminator {
        InitializeExtraMetas::DISCRIMINATOR_SLICE => {
            InitializeExtraMetas::try_from(accounts)?.process()
        }
        CanThawPermissionlessInstruction::SPL_DISCRIMINATOR_SLICE
        | CanFreezePermissionlessInstruction::SPL_DISCRIMINATOR_SLICE => {
            CanThawFreezePermissionless::try_from(accounts)?.process(payload)
        }
        _ => Err(CustomErrors::InvalidInstruction.into()),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CustomErrors {
    InvalidInstruction = 0x5000,
    /// The forwarded payload is not [`EXPECTED_PAYLOAD`].
    PayloadMismatch = 0x5001,
    /// The payload-derived extra account does not match the payload.
    PayloadAccountMismatch = 0x5002,
    /// The flag account is not the Token ACL flag account set for this call.
    InvalidFlagAccount = 0x5003,
}

impl From<CustomErrors> for ProgramError {
    fn from(e: CustomErrors) -> Self {
        ProgramError::Custom(e as u32)
    }
}
