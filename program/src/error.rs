use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub enum MtreeError {
    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("Expected signer account")]
    ExpectedSignerAccount,

    #[error("Invalid info account")]
    InvalidInfoAccount,

    #[error("Expected writable account")]
    ExpectedWritableAccount,

    #[error("Invalid system program")]
    InvalidSystemProgram,

    #[error("Invalid node account")]
    InvalidNodeAccount,

    #[error("Invalid sub tree account")]
    SubTreeFull,

    #[error("Uninitialized sub tree account")]
    UninitializedSubTree,
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
