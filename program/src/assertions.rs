use crate::error::MtreeError;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

pub fn assert_signer(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if !account.is_signer {
        msg!(
            "Account \"{}\" [{}] must be a signer",
            account_name,
            account.key,
        );
        Err(MtreeError::ExpectedSignerAccount.into())
    } else {
        Ok(())
    }
}

pub fn assert_system_program(account: &AccountInfo) -> ProgramResult {
    if account.key != &solana_program::system_program::ID {
        msg!(
            "Account \"system_program\" [{}] must be the system program",
            account.key,
        );
        return Err(MtreeError::InvalidSystemProgram.into());
    }

    Ok(())
}
