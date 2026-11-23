//! Payload support for the permissionless thaw/freeze instructions.
//!
//! Token ACL forwards every byte after its 1-byte instruction discriminator to
//! the gating program, appended after the sRFC37 8-byte discriminator. The
//! IDL models this as a trailing optional `data` argument; the generated Rust
//! builders omit it (see `scripts/generate-clients.mjs`), so these helpers
//! append it instead.

use solana_instruction::Instruction;

use crate::instructions::{
    FreezePermissionlessBuilder, FreezePermissionlessIdempotentBuilder, ThawPermissionlessBuilder,
    ThawPermissionlessIdempotentBuilder,
};

/// Appends a gate payload to an already built permissionless thaw/freeze
/// instruction. An empty `data` leaves the instruction unchanged.
pub fn append_permissionless_data(instruction: &mut Instruction, data: &[u8]) {
    instruction.data.extend_from_slice(data);
}

macro_rules! impl_instruction_with_data {
    ($($builder:ty),* $(,)?) => {
        $(
            impl $builder {
                /// Builds the instruction with `data` appended after the 1-byte
                /// discriminator. Token ACL forwards `data` verbatim to the
                /// gating program; an empty slice is equivalent to `instruction()`.
                pub fn instruction_with_data(&self, data: &[u8]) -> Instruction {
                    let mut instruction = self.instruction();
                    append_permissionless_data(&mut instruction, data);
                    instruction
                }
            }
        )*
    };
}

impl_instruction_with_data!(
    ThawPermissionlessBuilder,
    ThawPermissionlessIdempotentBuilder,
    FreezePermissionlessBuilder,
    FreezePermissionlessIdempotentBuilder,
);
