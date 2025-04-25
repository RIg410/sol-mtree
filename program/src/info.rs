use crate::mtree::{Hash, SubTreeId};
use borsh::{BorshDeserialize, BorshSerialize};
use core::mem::size_of;
use solana_program::pubkey::Pubkey;

pub fn find_sub_tree_pda(node_id: SubTreeId, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&node_id.to_be_bytes()[..]], program_id)
}

pub const INFO_SEED: &[u8] = b"info";

pub fn find_info_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[INFO_SEED], program_id)
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct MTreeInfo {
    pub node_id: SubTreeId,
    pub root_hash: Hash,
}

impl MTreeInfo {
    pub const LEN: usize = size_of::<SubTreeId>() + size_of::<Hash>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borsh_info_size() {
        let info = MTreeInfo::default();
        let size = info.try_to_vec().unwrap().len();
        assert_eq!(size, MTreeInfo::LEN);
    }
}
