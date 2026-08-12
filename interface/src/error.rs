use num_traits::FromPrimitive;
use solana_program_error::ProgramError;
use spl_tlv_account_resolution::error::AccountResolutionError;

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum ThawFreezeGateError {
    /// Incorrect account provided
    //#[error("Incorrect account provided")]
    IncorrectAccount,

    //#[error("Missing AccountMeta in instruction")]
    MissingAccountMeta,

    //#[error("ExtraAccountMeta not found or empty")]
    MissingExtraAccountMeta,

    //#[error("Resolution error")]
    ResolutionError(spl_tlv_account_resolution::error::AccountResolutionError),

    ProgramError(ProgramError),

    InvalidTokenMint,
}

impl std::fmt::Display for ThawFreezeGateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<AccountResolutionError> for ThawFreezeGateError {
    fn from(err: AccountResolutionError) -> Self {
        Self::ResolutionError(err)
    }
}

impl From<ProgramError> for ThawFreezeGateError {
    fn from(err: ProgramError) -> Self {
        match err {
            ProgramError::Custom(code) => AccountResolutionError::from_u32(code)
                .map_or(Self::ProgramError(err), Self::ResolutionError),
            _ => Self::ProgramError(err),
        }
    }
}
