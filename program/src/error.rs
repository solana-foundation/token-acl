use solana_program_error::ProgramError;



#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EbaltsError {
    InvalidAuthority,
    InvalidSystemProgram,
    InvalidTokenProgram,
    InvalidTokenMint,
    InvalidMintConfig,
    InvalidGatingProgram,
    PermissionlessThawNotEnabled,
    PermissionlessFreezeNotEnabled,
}


impl From<EbaltsError> for ProgramError {
    fn from(e: EbaltsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}