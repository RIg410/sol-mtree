use crate::{error::MtreeError, instruction::MTreeInstruction, processor};
use borsh::BorshDeserialize as _;
use solana_program::entrypoint;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

entrypoint!(process_instruction);

fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = MTreeInstruction::try_from_slice(instruction_data)
        .map_err(|_| MtreeError::InvalidInstruction)?;
    match instruction {
        MTreeInstruction::InsertLeaf(leaf) => {
            msg!("Instruction: InsertLeaf");
            processor::insert_leaf(program_id, accounts, leaf)
        }
    }
}
