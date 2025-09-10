use solana_program_error::ProgramError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenAclError {
    InvalidAuthority,
    InvalidSystemProgram,
    InvalidTokenProgram,
    InvalidTokenMint,
    InvalidMintConfig,
    InvalidGatingProgram,
    PermissionlessThawNotEnabled,
    PermissionlessFreezeNotEnabled,
    InvalidTokenAccountOwner,
}

impl From<TokenAclError> for ProgramError {
    fn from(e: TokenAclError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
