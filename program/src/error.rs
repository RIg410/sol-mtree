use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub enum MtreeError {
    #[error("Expected signer account")]
    ExpectedSignerAccount,

    #[error("Invalid system program")]
    InvalidSystemProgram,

    #[error("Invalid instruction")]
    InvalidInstruction,
}

impl PrintProgramError for MtreeError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<MtreeError> for ProgramError {
    fn from(e: MtreeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MtreeError {
    fn type_of() -> &'static str {
        "Mpl Project Name Error"
    }
}
