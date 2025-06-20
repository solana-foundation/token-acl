

#[repr(u32)]
#[derive(Clone, Debug, Eq, thiserror::Error, num_derive::FromPrimitive, PartialEq)]
pub enum ThawFreezeGateError {
    /// Incorrect account provided
    #[error("Incorrect account provided")]
    IncorrectAccount = 2_110_272_652,
}