use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub enum MTreeInstruction {
    InsertLeaf(Vec<u8>),
}

#[cfg(feature = "encode")]
pub mod encode {
    use std::io;

    use crate::info::{find_info_pda, find_sub_tree_pda};
    use crate::mtree::path::get_path_to_root;
    use crate::mtree::SubTreeId;
    use borsh::BorshSerialize as _;
    use solana_program::instruction::{AccountMeta, Instruction};
    use solana_program::pubkey::Pubkey;

    use super::MTreeInstruction;

    pub fn make_insert_leaf_instruction(
        program_id: Pubkey,
        sender: Pubkey,
        data: Vec<u8>,
        id: SubTreeId,
    ) -> Result<Instruction, io::Error> {
        let path = get_path_to_root(id);
        let mut accounts = Vec::with_capacity(2 + path.len());

        accounts.push(AccountMeta::new(sender, true));
        accounts.push(AccountMeta::new(find_info_pda(&program_id).0, false));
        accounts.push(AccountMeta::new_readonly(
            solana_program::system_program::ID,
            false,
        ));

        for node_id in path {
            accounts.push(AccountMeta::new(
                find_sub_tree_pda(node_id, &program_id).0,
                false,
            ));
        }

        accounts.push(AccountMeta::new_readonly(
            solana_program::sysvar::rent::ID,
            false,
        ));

        Ok(Instruction {
            program_id,
            accounts,
            data: MTreeInstruction::InsertLeaf(data).try_to_vec()?,
        })
    }
}
