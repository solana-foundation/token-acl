use solana_program::{instruction::{AccountMeta, Instruction}, pubkey::Pubkey};

use crate::instructions::{CreateConfig, ForfeitFreezeAuthority, Freeze, FreezePermissionless, SetAuthority, SetGatingProgram, Thaw, ThawPermissionless, TogglePermissionlessInstructions};
use std::mem::size_of;


#[repr(C)]
pub enum EbaltsInstruction {
    // 0
    /// Initializes a new mint config.
    /// If the authority is the same as the mint freeze authority it will
    /// CPI into token22 program to set the mint config account as the freeze authority.
    /// 
    /// Accounts:
    /// 
    ///   0. `[writable]` - payer
    ///   1. `[signer]` - authority
    ///   2. `[writable]` - mint
    ///   3. `[writable]` - mint config
    ///   4. `[]` - system program
    ///   5. `[]` - token program
    CreateConfig {
        /// The gating program used to check for permissionless calls.
        gating_program: Pubkey,
    },
    // 1
    /// Sets the freeze authority for a given mint config.
    /// 
    /// Accounts:
    /// 
    ///   0. `[signer]` - authority
    ///   1. `[writable]` - mint config
    SetAuthority {
        /// The new freeze authority.
        new_authority: Pubkey,
    },
    // 2
    /// Sets the gating program for a given mint config.
    /// 
    /// Accounts:
    /// 
    ///   0. `[signer]` - authority
    ///   1. `[writable]` - mint config
    SetGatingProgram {
        /// The new gating program.
        new_gating_program: Pubkey,
    },
    // 3
    /// Forfeits the freeze authority from the program back to the one provided by the `MintConfig.freeze_authority`.
    /// 
    /// Accounts:
    /// 
    ///   0. `[signer]` - authority
    ///   1. `[writable]` - mint
    ///   2. `[writable]` - mint config
    ///   3. `[]` - token program
    ForfeitFreezeAuthority {
        /// The new freeze authority to be set for the token mint.
        new_freeze_authority: Pubkey,
    },
    // 4
    /// Thaws a given token account.
    /// 
    /// Accounts:
    /// 
    ///   0. `[signer]` - authority
    ///   1. `[]` - mint
    ///   2. `[writable]` - token account
    ///   3. `[]` - mint config
    ///   4. `[]` - token program
    Thaw,
    // 5
    /// Freezes a given token account.
    /// 
    /// Accounts:
    /// 
    ///   0. `[signer]` - authority
    ///   1. `[]` - mint
    ///   2. `[writable]` - token account
    ///   3. `[]` - mint config
    ///   4. `[]` - token program
    Freeze,
    // 6
    /// Thaws a given token account permissionlessly.
    /// 
    /// Accounts:
    /// 
    ///   0. `[signer]` - authority
    ///   1. `[]` - mint
    ///   2. `[writable]` - token account
    ///   3. `[]` - mint config
    ///   4. `[]` - token program
    ///   5. `5+N` `[]` - remaining accounts as required by extra account metas for permissionless thaw
    ThawPermissionless,
    // 7
    /// Freezes a given token account permissionlessly.
    /// 
    /// Accounts:
    /// 
    ///   0. `[signer]` - authority
    ///   1. `[]` - mint
    ///   2. `[writable]` - token account
    ///   3. `[]` - mint config
    ///   4. `[]` - token program
    ///   5. `5+N` `[]` - remaining accounts as required by extra account metas for permissionless freeze
    FreezePermissionless,
    // 8
    /// Toggles permissionless freeze and thaw instructions.
    /// 
    /// Accounts:
    /// 
    ///   0. `[signer]` - authority
    ///   1. `[]` - mint config
    TogglePermissionlessInstructions {
        freeze_enabled: bool,
        thaw_enabled: bool,
    },
}

impl EbaltsInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::CreateConfig { gating_program } => {
                data.push(CreateConfig::DISCRIMINATOR);
                data.extend_from_slice(&gating_program.to_bytes());
            },
            Self::SetAuthority { new_authority } => {
                data.push(SetAuthority::DISCRIMINATOR);
                data.extend_from_slice(&new_authority.to_bytes());
            },
            Self::SetGatingProgram { new_gating_program } => {
                data.push(SetGatingProgram::DISCRIMINATOR);
                data.extend_from_slice(&new_gating_program.to_bytes());
            },
            Self::ForfeitFreezeAuthority { new_freeze_authority } => {
                data.push(ForfeitFreezeAuthority::DISCRIMINATOR);
                data.extend_from_slice(&new_freeze_authority.to_bytes());
            },
            Self::Thaw => data.push(Thaw::DISCRIMINATOR),
            Self::Freeze => data.push(Freeze::DISCRIMINATOR),
            Self::ThawPermissionless => data.push(ThawPermissionless::DISCRIMINATOR),
            Self::FreezePermissionless => data.push(FreezePermissionless::DISCRIMINATOR),
            Self::TogglePermissionlessInstructions { freeze_enabled, thaw_enabled } => {
                data.push(TogglePermissionlessInstructions::DISCRIMINATOR);
                data.push(*freeze_enabled as u8);
                data.push(*thaw_enabled as u8);
            },
        }
        data
    }
}

pub fn create_config(
    program_id: &Pubkey,
    payer: &Pubkey,
    authority: &Pubkey,
    mint: &Pubkey,
    gating_program: &Pubkey,
    mint_config: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let data = EbaltsInstruction::CreateConfig { gating_program: *gating_program }.pack();

    let accounts = vec![
        AccountMeta::new(*payer, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*mint, false),
        AccountMeta::new(*mint_config, false),
        AccountMeta::new_readonly(solana_system_interface::program::ID, false),
        AccountMeta::new_readonly(*token_program, false),
    ];

    Instruction::new_with_bytes(*program_id, &data, accounts)
}

pub fn set_authority(
    program_id: &Pubkey,
    authority: &Pubkey,
    new_authority: &Pubkey,
    mint_config: &Pubkey,
) -> Instruction {
    let data = EbaltsInstruction::SetAuthority { new_authority: *new_authority }.pack();

    let accounts = vec![
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*mint_config, false),
    ];

    Instruction::new_with_bytes(*program_id, &data, accounts)
}

pub fn set_gating_program(
    program_id: &Pubkey,
    authority: &Pubkey,
    new_gating_program: &Pubkey,
    mint_config: &Pubkey,
) -> Instruction {
    let data = EbaltsInstruction::SetGatingProgram { new_gating_program: *new_gating_program }.pack();

    let accounts = vec![
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*mint_config, false),
    ];

    Instruction::new_with_bytes(*program_id, &data, accounts)
}

pub fn forfeit_freeze_authority(
    program_id: &Pubkey,
    authority: &Pubkey,
    new_freeze_authority: &Pubkey,
    mint: &Pubkey,
    mint_config: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let data = EbaltsInstruction::ForfeitFreezeAuthority { new_freeze_authority: *new_freeze_authority }.pack();

    let accounts = vec![
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*mint, false),
        AccountMeta::new(*mint_config, false),
        AccountMeta::new_readonly(*token_program, false),
    ];

    Instruction::new_with_bytes(*program_id, &data, accounts)
}

pub fn thaw(
    program_id: &Pubkey,
    authority: &Pubkey,
    mint: &Pubkey,
    token_account: &Pubkey,
    mint_config: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let data = EbaltsInstruction::Thaw.pack();

    let accounts = vec![
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new_readonly(*mint_config, false),
        AccountMeta::new_readonly(*token_program, false),
    ];

    Instruction::new_with_bytes(*program_id, &data, accounts)
}

pub fn freeze(
    program_id: &Pubkey,
    authority: &Pubkey,
    mint: &Pubkey,
    token_account: &Pubkey,
    mint_config: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let data = EbaltsInstruction::Freeze.pack();

    let accounts = vec![
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new_readonly(*mint_config, false),
        AccountMeta::new_readonly(*token_program, false),
    ];

    Instruction::new_with_bytes(*program_id, &data, accounts)
}


pub fn thaw_permissionless(
    program_id: &Pubkey,
    signer: &Pubkey,
    mint: &Pubkey,
    mint_config: &Pubkey,
    token_account: &Pubkey,
    token_account_owner: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let data = EbaltsInstruction::ThawPermissionless.pack();

    let accounts = vec![
        AccountMeta::new_readonly(*signer,true),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(*token_account, true),
        AccountMeta::new_readonly(*mint_config, false),
        AccountMeta::new_readonly(*token_account_owner, false),
        AccountMeta::new_readonly(*token_program, false),
    ];

    Instruction::new_with_bytes(*program_id, &data, accounts)
}

pub fn freeze_permissionless(
    program_id: &Pubkey,
    signer: &Pubkey,
    mint: &Pubkey,
    mint_config: &Pubkey,
    token_account: &Pubkey,
    token_account_owner: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let data = EbaltsInstruction::FreezePermissionless.pack();

    let accounts = vec![
        AccountMeta::new_readonly(*signer,true),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(*token_account, true),
        AccountMeta::new_readonly(*mint_config, false),
        AccountMeta::new_readonly(*token_account_owner, false),
        AccountMeta::new_readonly(*token_program, false),
    ];

    Instruction::new_with_bytes(*program_id, &data, accounts)
}

pub fn toggle_permissionless_instructions(
    program_id: &Pubkey,
    authority: &Pubkey,
    mint_config: &Pubkey,
    freeze_enabled: bool,
    thaw_enabled: bool,
) -> Instruction {
    let data = EbaltsInstruction::TogglePermissionlessInstructions { freeze_enabled, thaw_enabled }.pack();

    let accounts = vec![
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*mint_config, false),
    ];

    Instruction::new_with_bytes(*program_id, &data, accounts)
}