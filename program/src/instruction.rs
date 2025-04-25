use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub enum MTreeInstruction {
    InsertLeaf(Vec<u8>),
}

#[cfg(feature = "encode")]
pub mod encode {
    use std::{io, vec};

    use crate::tree::find_mtree_pda;
    use borsh::BorshSerialize as _;
    use solana_program::instruction::{AccountMeta, Instruction};
    use solana_program::pubkey::Pubkey;

    use super::MTreeInstruction;

    pub fn make_insert_leaf_instruction(
        program_id: Pubkey,
        sender: Pubkey,
        data: Vec<u8>,
    ) -> Result<Instruction, io::Error> {
        let accounts = vec![
            AccountMeta::new(sender, true),
            AccountMeta::new(find_mtree_pda(&program_id).0, false),
            AccountMeta::new_readonly(solana_program::system_program::ID, false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false),
        ];

        Ok(Instruction {
            program_id,
            accounts,
            data: MTreeInstruction::InsertLeaf(data).try_to_vec()?,
        })
    }
}
